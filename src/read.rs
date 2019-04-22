//! A parser that turns a string of source code into a pavo valect.

use failure_derive::Fail;
use nom::{
    {line_ending, not_line_ending, multispace1, eof},
    {value, tag, take_while1},
    {do_parse, alt, many0, opt, preceded},
    {delimited, terminated},
    {named, named_args, call, not, map, try_parse},
    IResult, Err, Context, ErrorKind,
    types::CompleteStr,
};

use crate::value::Value;

named!(linecomment(CompleteStr) -> (), do_parse!(
    tag!("#") >>
    many0!(not_line_ending) >>
    opt!(line_ending) >>
    (())
));

named!(ws(CompleteStr) -> (), alt!(
    value!((), linecomment) |
    value!((), multispace1) |
    value!((), tag!(","))
));

named!(ws0(CompleteStr) -> (), do_parse!(
    many0!(ws) >>
    (())
));

named!(lbrace(CompleteStr) -> (), do_parse!(tag!("{") >> ws0 >> (())));
named!(rbrace(CompleteStr) -> (), do_parse!(tag!("}") >> ws0 >> (())));
named!(lbracket(CompleteStr) -> (), do_parse!(tag!("[") >> ws0 >> (())));
named!(rbracket(CompleteStr) -> (), do_parse!(tag!("]") >> ws0 >> (())));
named!(lparen(CompleteStr) -> (), do_parse!(tag!("(") >> ws0 >> (())));
named!(rparen(CompleteStr) -> (), do_parse!(tag!(")") >> ws0 >> (())));

fn is_id_char(c: char) -> bool {
    return c.is_ascii_alphanumeric() || c == '!' || c == '*' || c == '+'
        || c == '-' || c == '_' || c == '?' || c == '~' || c == '<'
        || c == '>' || c == '=';
}

fn id_str(i: CompleteStr) -> IResult<CompleteStr, CompleteStr> {
    let (i, id) = try_parse!(i, take_while1!(is_id_char));

    if id.len() > 255 {
        Err(Err::Error(Context::Code(i, ErrorKind::Custom(0))))
    } else {
        Ok((i, id))
    }
}

named!(kw_str(CompleteStr) -> CompleteStr, preceded!(tag!(":"), id_str));

fn num(i: CompleteStr) -> IResult<CompleteStr, i64> {
    let start = i;
    let (i, has_sign) = try_parse!(i, map!(opt!(alt!(tag!("+") | tag!("-"))), |opt| opt.is_some()));
    let (i, is_hex) = try_parse!(i, map!(opt!(tag!("0x")), |opt| opt.is_some()));
    let (i, _) = if is_hex {
        try_parse!(i, take_while1!(|c: char| c.is_ascii_hexdigit()))
    } else {
        try_parse!(i, take_while1!(|c: char| c.is_ascii_digit()))
    };
    let end = i;
    let (i, _) = try_parse!(i, not!(take_while1!(is_id_char)));
    let (i, _) = try_parse!(i, ws0);

    let raw = if is_hex {
        if has_sign {
            let mut buf = start[..1].to_string();
            buf.push_str(&start[3..end.len() - start.len()]);
            buf
        } else {
            start[2..end.len() - start.len()].to_string()
        }
    } else {
        start[..end.len() - start.len()].to_string()
    };

    match i64::from_str_radix(&raw, if is_hex { 16 } else { 10 }) {
        Ok(n) => Ok((i, n)),
        Err(_) => Err(Err::Error(Context::Code(i, ErrorKind::Custom(1)))),
    }
}

named_args!(app(color: usize)<CompleteStr, Value>, map!(
    delimited!(lparen, many0!(call!(val, color)), rparen),
    |vals| Value::app_from_vec(vals)
));

named_args!(arr(color: usize)<CompleteStr, Value>, map!(
    delimited!(lbracket, many0!(call!(val, color)), rbracket),
    |vals| Value::arr_from_vec(vals)
));

named_args!(map_(color: usize)<CompleteStr, Value>, map!(
    delimited!(lbrace, many0!(do_parse!(
        key: call!(val, color) >>
        val: call!(val, color) >>
        ((key, val))
    )), rbrace),
    |entries| Value::map_from_vec(entries)
));

named_args!(val(color: usize)<CompleteStr, Value>, terminated!(alt!(
    call!(app, color) |
    call!(arr, color) |
    call!(map_, color) |
    map!(num, |n| Value::int(n)) |
    map!(kw_str, |kw| Value::kw_str(kw.0)) |
    map!(id_str, |id| if id.0 == "nil" {
        Value::nil()
    } else if id.0 == "true" {
        Value::bool_(true)
    } else if id.0 == "false" {
        Value::bool_(false)
    } else {
        Value::id_str_color(id.0, color)
    })
), ws0));

named_args!(read_(color: usize)<CompleteStr, Value>, do_parse!(
    ws0 >>
    o: call!(val, color) >>
    eof!() >>
    (o)
));

pub fn read(i: CompleteStr, color: usize) -> Result<Value, ParseError> {
    match read_(i, color) {
        Ok((_, o)) => return Ok(o),
        Err(Err::Incomplete(_)) => unreachable!(),
        Err(Err::Error(cx)) | Err(Err::Failure(cx)) => {
            return Err(ParseError(cx.into_error_kind()));
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Fail)]
#[fail(display = "Parse error of kind {:?}", 0)]
pub struct ParseError(ErrorKind);
