use gc_derive::{Trace, Finalize};

use crate::gc_foreign::Vector;

// FIXME this doesn't uphold the time complexity guarantees, next and prev always take O(log n)
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize, PartialOrd, Ord)]
pub struct VectorCursor<T: Clone> {
    index: usize,
    vector: Vector<T>,
}

impl<T: Clone> VectorCursor<T> {
    pub fn new(v: Vector<T>, index: usize) -> VectorCursor<T> {
        VectorCursor {
            index,
            vector: v,
        }
    }

    pub fn next(&mut self) -> Option<T> {
        if self.index == self.vector.0.len() {
            return None;
        } else {
            self.index += 1;
            return Some(self.vector.0[self.index - 1].clone());
        }
    }

    pub fn prev(&mut self) -> Option<T> {
        if self.index == 0 {
            return None;
        } else {
            self.index -= 1;
            return Some(self.vector.0[self.index].clone());
        }
    }
}
