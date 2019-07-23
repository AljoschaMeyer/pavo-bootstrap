#![feature(reverse_bits)]
#![feature(euclidean_division)]
#![feature(copysign)]

use std::collections::HashMap;
use std::io::{self, Read};
use std::fs::File;
use std::env::set_current_dir;

use nom::types::CompleteStr;
use im_rc::OrdMap as ImOrdMap;
use structopt::StructOpt;

mod builtins;
mod check;
mod compile;
mod context;
mod env;
mod expand;
mod gc_foreign;
mod macros;
mod special_forms;
mod value;
mod read;
mod vm;
mod opaques;

use compile::StaticError;
use context::Context;
use expand::ExpandError;
use value::{Id, Value};
use read::{read, ParseError};

#[derive(StructOpt)]
struct Cli {
    /// The pavo file to run.
    #[structopt(parse(from_os_str))]
    entrypoint: std::path::PathBuf,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ExecuteError {
    Parse(ParseError),
    E(E),
}

impl From<ParseError> for ExecuteError {
    fn from(err: ParseError) -> Self {
        ExecuteError::Parse(err)
    }
}

impl From<E> for ExecuteError {
    fn from(err: E) -> Self {
        ExecuteError::E(err)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum E {
    Expand(ExpandError),
    Static(StaticError),
    Eval(Value),
}

impl From<StaticError> for E {
    fn from(err: StaticError) -> Self {
        E::Static(err)
    }
}

impl From<ExpandError> for E {
    fn from(err: ExpandError) -> Self {
        E::Expand(err)
    }
}

impl From<Value> for E {
    fn from(err: Value) -> Self {
        E::Eval(err)
    }
}

pub fn exval(
    v: &Value,
    m_env: &HashMap<Id, (Value, bool)>,
    macros: &ImOrdMap<Id, Value>,
    env: &HashMap<Id, (Value, bool)>,
    cx: &mut Context,
) -> Result<Value, E> {
    let expanded = expand::expand(v, m_env, macros, cx)?;
    let c = compile::compile(&expanded, env)?;
    c.compute(gc_foreign::Vector(im_rc::Vector::new()), cx).map_err(|nay| E::Eval(nay))
}

pub fn execute(src: &str) -> Result<Value, ExecuteError> {
    let mut default_cx = Context::default();
    let default_env = env::default();
    let default_macros = macros::default();

    let v = read(CompleteStr(src))?;
    let yay = exval(&v, &default_env, &default_macros, &default_env, &mut default_cx)?;
    return Ok(yay);
}

fn main() -> Result<(), io::Error> {
    let args = Cli::from_args();

    let mut file = File::open(&args.entrypoint)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    match args.entrypoint.parent() {
        Some(parent) => {
            set_current_dir(parent)?;
        }
        None => {
            panic!("Argument must point to a source code file inside a directory");
        }
    }

    match execute(&contents) {
        Ok(yay) => {
            let mut buf = String::new();
            value::debug_print(&yay, 0, 2, &mut buf);
            println!("{}", buf);
            return Ok(());
        }
        Err(err) => {
            match err {
                ExecuteError::E(E::Eval(err)) => {
                    let mut buf = String::new();
                    value::debug_print(&err, 0, 2, &mut buf);
                    panic!("Thrown:\n{}", buf);
                }
                _ => panic!("{:?}", err),
            }
        }
    }
}
