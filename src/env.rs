use gc::GcCell;
use gc_derive::{Trace, Finalize};
use im_rc::OrdMap as ImOrdMap;

use crate::builtins;
use crate::context::Context;
use crate::gc_foreign::OrdMap;
use crate::value::{Value, Id, Builtin};

/// An environment that maps identifiers to mutable cells of objects.
///
/// All bindings are mutable, enforcement of pavo's mutability semantics happens at a different
/// layer (the syntactic checks).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Env(pub OrdMap<Id, GcCell<Value>>);

impl Env {
    // Construct a default pavo top-level environment.
    pub fn default(cx: &mut Context) -> Env {
        let mut m = ImOrdMap::new();

        m.insert(Id::user("apply"), GcCell::new(Value::apply(cx)));

        env_add(&mut m, "typeof", builtins::typeof_, cx);
        env_add(&mut m, "nil?", builtins::is_nil, cx);
        env_add(&mut m, "bool?", builtins::is_bool, cx);

        Env(OrdMap(m))
    }

    // Update the binding for the given id. Panics if the id hasn't been bound before.
    pub fn set(&self, id: &Id, v: Value) {
        *(self.0).0.get(id).unwrap().borrow_mut() = v;
    }

    pub fn update(&self, id: Id, v: Value) -> Env {
        Env(OrdMap((self.0).0.update(id, GcCell::new(v))))
    }

    pub fn get(&self, id: &Id) -> Option<Value> {
        (self.0).0.get(id).map(|inner| inner.borrow().clone())
    }
}

fn env_add(
    m: &mut ImOrdMap<Id, GcCell<Value>>,
    name: &str,
    b: fn(Value, &mut Context) -> Result<Value, Value>,
    cx: &mut Context,
) {
    m.insert(
        Id::user(name),
        GcCell::new(Value::builtin(Builtin(b), cx))
    );
}
