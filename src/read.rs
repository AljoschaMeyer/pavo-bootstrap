//! A parser that turns a string of source code into a pavo value.

use failure_derive::Fail;
use nom::{
    {is_a, one_of, eof},
    {value, tag, take_while1},
    {do_parse, alt, many0, many1, opt, preceded},
    {delimited, terminated},
    {named, map, try_parse, separated_list},
    none_of, many_m_n, take_until,
    IResult, Err, Context, ErrorKind,
    types::CompleteStr,
    hex_digit,
};
use strtod::strtod;

use crate::value::Value;

named!(line_ending(CompleteStr) -> (), value!((), one_of!("\n\r")));
named!(not_line_ending(CompleteStr) -> (), value!((), none_of!("\n\r")));

named!(linecomment(CompleteStr) -> (), do_parse!(
    tag!("#") >>
    many0!(not_line_ending) >>
    opt!(line_ending) >>
    (())
));

named!(ws(CompleteStr) -> (), alt!(
    value!((), linecomment) |
    value!((), is_a!(",\n\t\r "))
));

named!(ws0(CompleteStr) -> (), do_parse!(
    many0!(ws) >>
    (())
));

named!(ws1(CompleteStr) -> (), do_parse!(
    many1!(ws) >>
    (())
));

named!(lbrace(CompleteStr) -> (), do_parse!(tag!("{") >> (())));
named!(rbrace(CompleteStr) -> (), do_parse!(tag!("}") >> (())));
named!(lbracket(CompleteStr) -> (), do_parse!(tag!("[") >> (())));
named!(rbracket(CompleteStr) -> (), do_parse!(tag!("]") >> (())));
named!(lparen(CompleteStr) -> (), do_parse!(tag!("(") >> (())));
named!(rparen(CompleteStr) -> (), do_parse!(tag!(")") >> (())));
named!(at_lbrace(CompleteStr) -> (), do_parse!(tag!("@{") >> (())));
named!(at_lbracket(CompleteStr) -> (), do_parse!(tag!("@[") >> (())));
named!(at_tilde(CompleteStr) -> (), do_parse!(tag!("@~") >> (())));

pub fn is_id_char(c: char) -> bool {
    return c.is_ascii_alphanumeric() || c == '!' || c == '*' || c == '+'
        || c == '-' || c == '_' || c == '?' || c == '.' || c == '%'
        || c == '<' || c == '>' || c == '=' || c == '/' || c == '\\'
        || c == '|' || c == '&';
}

fn id_str(i: CompleteStr) -> IResult<CompleteStr, CompleteStr> {
    let (i, id) = try_parse!(i, take_while1!(is_id_char));

    if id.len() > 255 {
        Err(Err::Failure(Context::Code(i, ErrorKind::Custom(0))))
    } else {
        Ok((i, id))
    }
}

named!(kw_str(CompleteStr) -> CompleteStr, preceded!(tag!(":"), id_str));

fn num(i: CompleteStr) -> IResult<CompleteStr, Value> {
    let start = i;
    let (i, has_sign) = try_parse!(i, map!(opt!(one_of!("+-")), |opt| opt.is_some()));
    let (i, is_hex) = try_parse!(i, map!(opt!(tag!("0x")), |opt| opt.is_some()));
    let (i, _) = if is_hex {
        try_parse!(i, take_while1!(|c: char| c.is_ascii_hexdigit()))
    } else {
        try_parse!(i, take_while1!(|c: char| c.is_ascii_digit()))
    };

    if is_hex {
        let end = i;

        let raw = if has_sign {
                let mut buf = start[..1].to_string();
                buf.push_str(&start[3..start.len() - end.len()]);
                buf
            } else {
                start[2..start.len() - end.len()].to_string()
            };

        match i64::from_str_radix(&raw, 16) {
            Ok(n) => return Ok((i, Value::int(n))),
            Err(_) => return Err(Err::Failure(Context::Code(i, ErrorKind::Custom(1)))),
        }
    } else {
        let (i, is_float) = try_parse!(i, map!(opt!(tag!(".")), |opt| opt.is_some()));

        if is_float {
            let (i, _) = try_parse!(i, take_while1!(|c: char| c.is_ascii_digit()));
            let (i, _) = try_parse!(i, opt!(do_parse!(
                one_of!("eE") >>
                opt!(one_of!("+-")) >>
                take_while1!(|c: char| c.is_ascii_digit()) >>
                (())
            )));
            let end = i;

            let raw = &start[..start.len() - end.len()];
            let f = strtod(raw).unwrap();
            if f.is_finite() {
                return Ok((i, Value::float(f)));
            } else {
                return Err(Err::Failure(Context::Code(i, ErrorKind::Custom(2))));
            }
        } else {
            let end = i;

            let raw = &start[..start.len() - end.len()];

            match i64::from_str_radix(raw, 10) {
                Ok(n) => return Ok((i, Value::int(n))),
                Err(_) => return Err(Err::Failure(Context::Code(i, ErrorKind::Custom(1)))),
            }
        }
    }
}

fn byte(i: CompleteStr) -> IResult<CompleteStr, u8> {
    let start = i;
    let (i, is_hex) = try_parse!(i, map!(opt!(tag!("0x")), |opt| opt.is_some()));
    let (i, _) = if is_hex {
        try_parse!(i, take_while1!(|c: char| c.is_ascii_hexdigit()))
    } else {
        try_parse!(i, take_while1!(|c: char| c.is_ascii_digit()))
    };
    let end = i;

    let raw = if is_hex {
        start[2..start.len() - end.len()].to_string()
    } else {
        start[..start.len() - end.len()].to_string()
    };

    match u8::from_str_radix(&raw, if is_hex { 16 } else { 10 }) {
        Ok(n) => Ok((i, n)),
        Err(_) => Err(Err::Failure(Context::Code(i, ErrorKind::Custom(5)))),
    }
}

fn unicode(i: CompleteStr) -> IResult<CompleteStr, char> {
    let start = i;
    let (i, _) = try_parse!(i, many_m_n!(1, 6, hex_digit));
    let end = i;

    let raw = start[..start.len() - end.len()].to_string();
    let numeric = u32::from_str_radix(&raw, 16).unwrap();

    match std::char::from_u32(numeric) {
        Some(c) => Ok((i, c)),
        None => Err(Err::Failure(Context::Code(i, ErrorKind::Custom(4)))),
    }
}

named!(char_char(CompleteStr) -> char, alt!(
    value!('\\', tag!("\\\\")) |
    value!('\'', tag!("\\'")) |
    value!('\t', tag!("\\t")) |
    value!('\n', tag!("\\n")) |
    delimited!(
        tag!("\\{"),
        unicode,
        tag!("}")
    ) |
    none_of!("\\'")
));

named!(char_str(CompleteStr) -> char, alt!(
    value!('\\', tag!("\\\\")) |
    value!('\"', tag!("\\\"")) |
    value!('\t', tag!("\\t")) |
    value!('\n', tag!("\\n")) |
    delimited!(
        tag!("\\{"),
        unicode,
        tag!("}")
    ) |
    none_of!("\"\\")
));

named!(char_(CompleteStr) -> Value, delimited!(
    tag!("'"),
    map!(char_char, Value::char_),
    do_parse!(tag!("'") >> (()))
));

named!(string(CompleteStr) -> Value, delimited!(
    tag!("\""),
    map!(many0!(char_str), Value::string_from_vec),
    do_parse!(tag!("\"") >> (()))
));

fn raw_string(i: CompleteStr) -> IResult<CompleteStr, Value> {
    let (i, leading_ats) = try_parse!(i, many_m_n!(1, 8, tag!("@")));
    let (i, _) = try_parse!(i, tag!("\""));
    let num_ats = leading_ats.len();
    let mut at_str = "\"".to_string();
    for _ in 0..num_ats {
        at_str.push('@');
    }

    let (i, raw) = try_parse!(i, take_until!(at_str.as_str()));
    let (i, _) = try_parse!(i, tag!(at_str.as_str()));

    return Ok((i, Value::string_from_str(&raw)));
}

named!(bytes(CompleteStr) -> Value, map!(
    delimited!(
        terminated!(at_lbracket, ws0),
        separated_list!(ws1, byte),
        preceded!(ws0, rbracket)
    ),
    |byte_vec| Value::bytes_from_vec(byte_vec)
));

named!(app(CompleteStr) -> Value, map!(
    delimited!(
        terminated!(lparen, ws0),
        separated_list!(ws1, obj),
        preceded!(ws0, rparen)
    ),
    |objs| Value::app_from_vec(objs)
));

named!(arr(CompleteStr) -> Value, map!(
    delimited!(
        terminated!(lbracket, ws0),
        separated_list!(ws1, obj),
        preceded!(ws0, rbracket)
    ),
    |objs| Value::arr_from_vec(objs)
));

named!(map_(CompleteStr) -> Value, map!(
    delimited!(
        terminated!(lbrace, ws0),
        separated_list!(ws1, do_parse!(
            key: obj >>
            ws1 >>
            val: obj >>
            ((key, val))
        )),
        preceded!(ws0, rbrace)
    ),
    |entries| Value::map_from_vec(entries)
));

named!(set(CompleteStr) -> Value, map!(
    delimited!(
        terminated!(at_lbrace, ws0),
        separated_list!(ws1, obj),
        preceded!(ws0, rbrace)
    ),
    |objs| Value::set_from_vec(objs)
));

named!(id(CompleteStr) -> Value, map!(id_str, |id| if id.0 == "nil" {
    Value::nil()
} else if id.0 == "true" {
    Value::bool_(true)
} else if id.0 == "false" {
    Value::bool_(false)
} else {
    Value::id_str(id.0)
}));

named!(quote(CompleteStr) -> Value, do_parse!(
    tag!("$") >>
    inner: obj >>
    (Value::app_from_vec(vec![Value::id_str("quote"), inner]))
));

named!(quasiquote(CompleteStr) -> Value, do_parse!(
    tag!("`") >>
    inner: obj >>
    (Value::app_from_vec(vec![Value::id_str("quasiquote"), inner]))
));

named!(unquote(CompleteStr) -> Value, do_parse!(
    tag!("~") >>
    inner: obj >>
    (Value::app_from_vec(vec![Value::kw_str("unquote"), inner]))
));

named!(unquote_splice(CompleteStr) -> Value, do_parse!(
    at_tilde >>
    inner: obj >>
    (Value::app_from_vec(vec![Value::kw_str("unquote-splice"), inner]))
));

fn fresh_name(i: CompleteStr) -> IResult<CompleteStr, Value> {
    let (i, _) = try_parse!(i, tag!("@"));
    let (i, inner) = try_parse!(i, obj);

    match inner.as_user_id() {
        Some(_) => return Ok((i, Value::app_from_vec(vec![Value::kw_str("fresh-name"), inner]))),
        None => return Err(Err::Failure(Context::Code(i, ErrorKind::Custom(6)))),
    }
}

named!(obj(CompleteStr) -> Value, alt!(
    quote |
    quasiquote |
    unquote |
    unquote_splice |
    app |
    arr |
    map_ |
    set |
    bytes |
    char_ |
    string |
    raw_string |
    num |
    map!(kw_str, |kw| Value::kw_str(kw.0)) |
    fresh_name |
    id
));

named!(read_(CompleteStr) -> Value, do_parse!(
    ws0 >>
    o: obj >>
    ws0 >>
    eof!() >>
    (o)
));

pub fn read(i: CompleteStr) -> Result<Value, ParseError> {
    match read_(i) {
        Ok((_, o)) => return Ok(o),
        Err(Err::Incomplete(_)) => unreachable!(),
        Err(Err::Error(cx)) | Err(Err::Failure(cx)) => {
            return Err(ParseError(cx.into_error_kind()));
        }
    }
}

pub fn parse_id(i: CompleteStr) -> Result<Value, ParseError> {
    match num(i) {
        Ok(..) => return Err(ParseError(ErrorKind::RegexpMatch)),
        _ => {
            match id(i) {
                Ok((_, o)) => return Ok(o),
                Err(Err::Incomplete(_)) => unreachable!(),
                Err(Err::Error(cx)) | Err(Err::Failure(cx)) => {
                    return Err(ParseError(cx.into_error_kind()));
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Fail)]
#[fail(display = "Parse error of kind {:?}", 0)]
pub struct ParseError(ErrorKind);
