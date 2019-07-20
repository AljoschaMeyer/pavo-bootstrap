use std::env::{current_dir, set_current_dir};
use std::path::PathBuf;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

use im_rc::{OrdMap as ImOrdMap, Vector as ImVector};
use nom::types::CompleteStr;

use crate::builtins;
use crate::gc_foreign::{OrdMap, Vector};
use crate::read::read;
use crate::value::{Value, Atomic, NUM_BUILTIN_OPAQUES};

/// Global state tracked throughout the execution.
///
/// This includes both semantically relevant data (e.g. id counters, the event loop) and metadata
/// for programmer feedback (error locations, callstack, macro expansion information).
pub struct Context {
    symbol_id: u64,
    fun_id: u64,
    cell_id: u64,
    level: usize,
    require_cache: RequireCache,
}

impl Context {
    pub fn new() -> Context {
        Context {
            symbol_id: NUM_BUILTIN_OPAQUES,
            fun_id: 0,
            cell_id: 0,
            level: 0, // no semantic effect, only for debugging information
            require_cache: RequireCache::new(),
        }
    }

    pub fn default() -> Context {
        Context::new()
    }

    pub fn next_symbol_id(&mut self) -> u64 {
        let old = self.symbol_id;
        self.symbol_id = self.symbol_id.checked_add(1).expect("symbol id counter overflow");
        return old;
    }

    pub fn next_fun_id(&mut self) -> u64 {
        let old = self.fun_id;
        self.fun_id = self.fun_id.checked_add(1).expect("function id counter overflow");
        return old;
    }

    pub fn next_cell_id(&mut self) -> u64 {
        let old = self.cell_id;
        self.cell_id = self.cell_id.checked_add(1).expect("cell id counter overflow");
        return old;
    }

    pub fn inc_level(&mut self) {
        self.level += 1;
    }

    pub fn dec_level(&mut self) {
        self.level -= 1;
    }

    pub fn require(
        &mut self,
        v: &Value,
        expand_opts: &ImOrdMap<Value, Value>,
        eval_opts: &ImOrdMap<Value, Value>
    ) -> Result<Value, Value>{
        match v {
            Value::Atomic(Atomic::String(s)) => {
                let old_dir = current_dir().unwrap();
                let s: String = s.0.clone().into();
                let path = PathBuf::from(s).canonicalize().unwrap();
                let newdir = path.parent().unwrap();

                match self.require_cache.evaled.get(&(path.clone(), expand_opts.clone(), eval_opts.clone())) {
                    Some(yay) => return yay.clone(),
                    None => {
                        match self.require_cache.expanded.get(&(path.clone(), expand_opts.clone())) {
                            Some(Ok(yay)) => {
                                set_current_dir(&newdir).unwrap();
                                match builtins::eval(Vector(ImVector::from(vec![
                                    yay.clone(),
                                    Value::map(OrdMap(expand_opts.clone())),
                                    ])), self) {
                                    Err(err) => {
                                        self.require_cache.evaled.insert((path.clone(), expand_opts.clone(), eval_opts.clone()), Err(err.clone()));
                                        set_current_dir(&old_dir).unwrap();
                                        return Err(err);
                                    }
                                    Ok(expanded) => {
                                        self.require_cache.evaled.insert((path.clone(), expand_opts.clone(), eval_opts.clone()), Ok(expanded.clone()));
                                        let ret = self.require(v, expand_opts, eval_opts);
                                        set_current_dir(&old_dir).unwrap();
                                        return ret;
                                    }
                                }
                            }
                            Some(Err(nope)) => {
                                set_current_dir(&old_dir).unwrap();
                                return Err(nope.clone());
                            }
                            None => {
                                match self.require_cache.read.get(&path) {
                                    Some(Ok(yay)) => {
                                        set_current_dir(&newdir).unwrap();
                                        match builtins::expand(Vector(ImVector::from(vec![
                                            yay.clone(),
                                            Value::map(OrdMap(expand_opts.clone())),
                                            ])), self) {
                                            Err(err) => {
                                                self.require_cache.expanded.insert((path.clone(), expand_opts.clone()), Err(err.clone()));
                                                set_current_dir(&old_dir).unwrap();
                                                return Err(err);
                                            }
                                            Ok(expanded) => {
                                                self.require_cache.expanded.insert((path.clone(), expand_opts.clone()), Ok(expanded.clone()));
                                                let ret = self.require(v, expand_opts, eval_opts);
                                                set_current_dir(&old_dir).unwrap();
                                                return ret;
                                            }
                                        }
                                    }
                                    Some(Err(nope)) => {
                                        set_current_dir(&old_dir).unwrap();
                                        return Err(nope.clone());
                                    }
                                    None => {
                                        match load_file(&path) {
                                            Err(err) => {
                                                self.require_cache.read.insert(path.clone(), Err(err.clone()));
                                                set_current_dir(&old_dir).unwrap();
                                                return Err(err);
                                            }
                                            Ok(src) => {
                                                match read(CompleteStr(&src)) {
                                                    Err(_) => {
                                                        let err = require_error();
                                                        self.require_cache.read.insert(path.clone(), Err(err.clone()));
                                                        set_current_dir(&old_dir).unwrap();
                                                        return Err(err);
                                                    }
                                                    Ok(code) => {
                                                        self.require_cache.read.insert(path.clone(), Ok(code));
                                                        let ret = self.require(v, expand_opts, eval_opts);
                                                        set_current_dir(&old_dir).unwrap();
                                                        return ret;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            _ => return Err(require_error()),
        }
    }
}

pub struct RequireCache {
    read: BTreeMap<PathBuf, Result<Value, Value>>,
    expanded: BTreeMap<(PathBuf, ImOrdMap<Value, Value>), Result<Value, Value>>,
    evaled: BTreeMap<(PathBuf, ImOrdMap<Value, Value>, ImOrdMap<Value, Value>), Result<Value, Value>>,
}

impl RequireCache {
    fn new() -> RequireCache {
        return RequireCache {
            read: BTreeMap::new(),
            expanded: BTreeMap::new(),
            evaled: BTreeMap::new(),
        };
    }
}

fn require_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-require")),
        ])))
}

fn load_file(p: &PathBuf) -> Result<String, Value> {
    match File::open(p) {
        Err(_) => return Err(require_error()),
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Err(_) => return Err(require_error()),
                Ok(_) => return Ok(contents),
            }
        }
    }
}
