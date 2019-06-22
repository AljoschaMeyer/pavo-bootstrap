//! Wrappers around foreign types so that they work with the gc.

use gc::{Trace, Finalize, custom_trace};
use gc_derive::{Trace, Finalize};
use im_rc;
use ordered_float;
use ropey;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Rope(#[unsafe_ignore_trace] pub ropey::Rope);

/// A garbage-collectable `ordered_float::NotNan<f64>`.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Trace, Finalize)]
pub struct NotNan(#[unsafe_ignore_trace] pub ordered_float::NotNan<f64>);

impl NotNan {
    pub unsafe fn unchecked_new(val: f64) -> Self {
        NotNan(ordered_float::NotNan::unchecked_new(val))
    }

    pub fn into_inner(self) -> f64 {
        self.0.into_inner()
    }
}

impl Eq for NotNan {}

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

/// A garbage-collectable `im_rc::OrdSet`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrdSet<K: Clone + Ord>(pub im_rc::OrdSet<K>);

impl<K: Trace + Clone + Ord> Finalize for OrdSet<K> {}
unsafe impl<K: Trace + Clone + Ord> Trace for OrdSet<K> {
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
