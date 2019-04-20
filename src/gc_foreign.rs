//! Wrappers around foreign types so that they work with the gc.

use im_rc;
use gc::{Trace, Finalize, custom_trace};

/// A garbage-collectable `im_rc::Vector`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vector<T: Clone>(pub im_rc::Vector<T>);

impl<T: Trace + Clone> Finalize for Vector<T> {}
unsafe impl<T: Trace + Clone> Trace for Vector<T> {
    custom_trace!(this, {
        for e in this.0.iter() {
            mark(e);
        }
    });
}

/// A garbage-collectable `im_rc::OrdMap`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrdMap<K: Clone + Ord, V: Clone>(pub im_rc::OrdMap<K, V>);

impl<K: Trace + Clone + Ord, V: Trace + Clone> Finalize for OrdMap<K, V> {}
unsafe impl<K: Trace + Clone + Ord, V: Trace + Clone> Trace for OrdMap<K, V> {
    custom_trace!(this, {
        for e in this.0.iter() {
            mark(e);
        }
    });
}
