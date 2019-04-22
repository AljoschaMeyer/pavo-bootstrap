use gc::GcCell;
use gc_derive::{Trace, Finalize};
use im_rc::OrdMap as ImOrdMap;

use crate::gc_foreign::OrdMap;
use crate::value::{Value, Id};

/// An environment that maps identifiers to mutable cells of objects.
///
/// All bindings are mutable, enforcement of pavo's mutability semantics happens at a different
/// layer (the syntactic checks).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Env(pub OrdMap<Id, GcCell<Value>>);

impl Env {
    // Update the binding for the given id. Panics if the id hasn't been bound before.
    fn update(&self, id: &Id, v: Value) {
        *(self.0).0.get(id).unwrap().borrow_mut() = v;
    }

    // Construct a default pavo top-level environment where the ids have the given color.
    pub fn default(color: usize) -> Env {
        let mut m = ImOrdMap::new();
        let mut id = 0;

        // env_add(&mut m, "nil?", color, builtins::is_nil, &mut id);

        Env(OrdMap(m))
    }
}

// fn env_add(
//     m: &mut ImOrdMap<Id, GcCell<Value>>,
//     name: &str,
//     color: usize,
//     b: fn(Value, &mut Context) -> Result<Value, Value>,
//     id: &mut usize
// ) {
//     m.insert(
//         Id {
//             chars: name.to_string(),
//             color,
//         },
//         GcCell::new(Value::builtin(Builtin {
//             fun: b,
//             id: *id
//         }))
//     );
//     *id += 1;
// }
