//! Definition of the objects that the language manipulates at runtime.

use std::{
    cmp::Ordering,
    fmt,
    num::FpCategory,
};

use gc_derive::{Trace, Finalize};
use im_rc::{
    Vector as ImVector,
    OrdSet as ImOrdSet,
    OrdMap as ImOrdMap,
};

use crate::gc_foreign::{Vector, OrdSet, OrdMap, NotNan, Rope};
use crate::context::Context;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Value {
    Atomic(Atomic),
    Id(Id),
    Arr(Vector<Value>),
    App(Vector<Value>),
    Set(OrdSet<Value>),
    Map(OrdMap<Value, Value>),
    Fun(Fun, u64),
}

impl Value {
    pub fn nil() -> Value {
        Value::Atomic(Atomic::Nil)
    }

    pub fn bool_(b: bool) -> Value {
        Value::Atomic(Atomic::Bool(b))
    }

    pub fn int(n: i64) -> Value {
        Value::Atomic(Atomic::Int(n))
    }

    // Panics if given an infinity or NaN, converts -0.0 to 0.0.
    pub fn float(n: f64) -> Value {
        let n = match n.classify() {
            FpCategory::Nan | FpCategory::Infinite => panic!("Floats must not be NaN or infinite"),
            FpCategory::Zero => 0.0,
            _ => n,
        };

        Value::Atomic(Atomic::Float(unsafe { NotNan::unchecked_new(n) }))
    }

    pub fn id(id: Id) -> Value {
        Value::Id(id)
    }

    pub fn id_str(id: &str) -> Value {
        Value::Id(Id::User(id.to_string()))
    }

    pub fn kw(kw: String) -> Value {
        Value::Atomic(Atomic::Keyword(kw))
    }

    pub fn kw_str(kw: &str) -> Value {
        Value::kw(kw.to_string())
    }

    fn arr(objs: Vector<Value>) -> Value {
        Value::Arr(objs)
    }

    pub fn arr_from_vec(objs: Vec<Value>) -> Value {
        Value::arr(Vector(ImVector::from(&objs)))
    }

    fn app(objs: Vector<Value>) -> Value {
        Value::App(objs)
    }

    pub fn app_from_vec(objs: Vec<Value>) -> Value {
        Value::app(Vector(ImVector::from(objs)))
    }

    pub fn set(objs: OrdSet<Value>) -> Value {
        Value::Set(objs)
    }

    pub fn set_from_vec(objs: Vec<Value>) -> Value {
        Value::set(OrdSet(ImOrdSet::from(objs)))
    }

    pub fn map(objs: OrdMap<Value, Value>) -> Value {
        Value::Map(objs)
    }

    pub fn map_from_vec(objs: Vec<(Value, Value)>) -> Value {
        Value::map(OrdMap(ImOrdMap::from(objs)))
    }

    pub fn closure(c: Closure, cx: &mut Context) -> Value {
        Value::Fun(Fun::Closure(c), cx.next_fun_id())
    }

    pub fn builtin(b: Builtin, cx: &mut Context) -> Value {
        Value::Fun(Fun::Builtin(b), cx.next_fun_id())
    }

    pub fn as_id(&self) -> Option<&Id> {
        match self {
            Value::Id(id) => Some(id),
            _ => None,
        }
    }

    pub fn as_user_id(&self) -> Option<&str> {
        match self {
            Value::Id(Id::User(id)) => Some(id),
            _ => None,
        }
    }

    pub fn as_atomic(&self) -> Option<&Atomic> {
        match self {
            Value::Atomic(atomic) => Some(atomic),
            _ => None,
        }
    }

    pub fn as_kw(&self) -> Option<&str> {
        self.as_atomic().and_then(|atomic| match atomic {
            Atomic::Keyword(kw) => Some(kw.as_str()),
            _ => None,
        })
    }

    pub fn is_kw(&self, expected: &str) -> bool {
        match self.as_kw() {
            Some(kw) => kw == expected,
            None => false,
        }
    }

    pub fn as_app(&self) -> Option<&Vector<Value>> {
        match self {
            Value::App(app) => Some(app),
            _ => None,
        }
    }
}

/// The atomic values are those that do not contain other objects and that use synactic equality.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Atomic {
    Nil,
    Bool(bool),
    Int(i64),
    Float(NotNan),
    Char(char),
    String(Rope),
    Bytes(Vector<u8>),
    Keyword(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Id {
    User(String),
    Symbol(u64),
}
//
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
// pub struct Id(String);
//
// impl Id {
//     pub fn get_chars(&self) -> &str {
//         &self.0
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Fun {
    // TODO ids!
    Closure(Closure),
    Builtin(Builtin),
    Apply, // the builtin function `apply` requires special interpretation logic
}

/// Runtime representation of a value produced by a lambda special form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Closure {
    // funs: Gc<BTreeMap<Id, (Id /* arg */, bool /*arg mutable*/, Value /*body*/)>>,
    // entry: Id,
    // env: Gc<Environment>
}

/// A function that is provided by pavo (as opposed to a programmer-defined closure).
#[derive(Trace, Finalize)]
pub struct Builtin {
    #[unsafe_ignore_trace]
    fun: fn(Value, &mut Context) -> Result<Value, Value>,
    // Each builtin is assigned an id that is distinct from the id of all other builtins.
    // Ids are used for comparisons.
    id: usize,
}

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Builtin {{ fun: {:?}, id: {:?} }}", self.fun as usize, self.id)
    }
}

impl Clone for Builtin {
    fn clone(&self) -> Self {
        Builtin { fun: self.fun.clone(), id: self.id }
    }
}

impl PartialEq for Builtin {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Builtin {}

impl Ord for Builtin {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Builtin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
