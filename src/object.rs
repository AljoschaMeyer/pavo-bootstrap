//! Definition of the objects that the language manipulates at runtime.

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt,
};

use gc::{Gc, GcCell};
use gc_derive::{Trace, Finalize};
use im_rc::{
    Vector as ImVector,
    // OrdMap as ImOrdMap,
};

use crate::gc_foreign::{Vector, OrdMap};
use crate::context::Context;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Value {
    Atomic(Atomic),
    Id(Id),
    Arr(Vector<Object>),
    App(Vector<Object>),
    // Map(OrdMap<Object, Object>),
    Fun(Fun),
}

/// The atomic values are those that do not contain other objects and that use synactic equality.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Atomic {
    Nil,
    Bool(bool),
    Int(i64),
    Keyword(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Id {
    pub chars: String,
    // Macro hygiene is implemented by coloring identifiers. Two ids can only compare as equal
    // if they have the same color. When read, all Ids start out with the color 0, recoloring
    // happens during macro expansion.
    pub color: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Fun {
    Closure(Closure),
    Builtin(Builtin),
}

/// Runtime representation of a value produced by a lambda special form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Closure {
    // funs: Gc<BTreeMap<Id, (Id /* arg */, bool /*arg mutable*/, Object /*body*/)>>,
    // entry: Id,
    // env: Gc<Environment>
}

/// A function that is provided by pavo (as opposed to a programmer-defined closure).
#[derive(Trace, Finalize)]
pub struct Builtin {
    #[unsafe_ignore_trace]
    fun: fn(Object, &mut Context) -> Result<Object, Object>,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Object(pub Value, ());

impl Object {
    pub fn nil() -> Object {
        Object(Value::Atomic(Atomic::Nil), ())
    }

    pub fn bool_(b: bool) -> Object {
        Object(Value::Atomic(Atomic::Bool(b)), ())
    }

    pub fn int(n: i64) -> Object {
        Object(Value::Atomic(Atomic::Int(n)), ())
    }

    pub fn id(id: Id) -> Object {
        Object(Value::Id(id), ())
    }

    pub fn id_str(id: &str) -> Object {
        Object(Value::Id(Id {
            chars: id.to_string(),
            color: 0,
        }), ())
    }

    pub fn kw(kw: String) -> Object {
        Object(Value::Atomic(Atomic::Keyword(kw)), ())
    }

    pub fn kw_str(kw: &str) -> Object {
        Object::kw(kw.to_string())
    }

    fn arr(objs: Vector<Object>) -> Object {
        Object(Value::Arr(objs), ())
    }

    pub fn arr_from_vec(objs: Vec<Object>) -> Object {
        Object::arr(Vector(ImVector::from(&objs)))
    }

    fn app(objs: Vector<Object>) -> Object {
        Object(Value::App(objs), ())
    }

    pub fn app_from_vec(objs: Vec<Object>) -> Object {
        Object::app(Vector(ImVector::from(objs)))
    }

    // pub fn map(objs: OrdMap<Object, Object>) -> Object {
    //     Object(Value::Map(objs), ())
    // }
    //
    // pub fn map_from_vec(objs: Vec<(Object, Object)>) -> Object {
    //     Object::map(OrdMap(ImOrdMap::from(objs)))
    // }

    pub fn closure(c: Closure) -> Object {
        Object(Value::Fun(Fun::Closure(c)), ())
    }

    pub fn builtin(b: Builtin) -> Object {
        Object(Value::Fun(Fun::Builtin(b)), ())
    }

    pub fn is_truthy(&self) -> bool {
        match self.0 {
            Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(false)) => false,
            _ => true,
        }
    }
}
