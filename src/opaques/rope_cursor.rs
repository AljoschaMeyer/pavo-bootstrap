use gc_derive::{Trace, Finalize};

use crate::gc_foreign::Rope;

// FIXME this doesn't uphold the time complexity guarantees, next and prev always take O(log n)
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub struct RopeCursor {
    index: usize,
    rope: Rope,
}

impl RopeCursor {
    pub fn new(rope: Rope, index: usize) -> RopeCursor {
        RopeCursor {
            index,
            rope,
        }
    }

    pub fn next_byte(&mut self) -> Option<u8> {
        if self.index == self.rope.0.len_bytes() {
            return None;
        } else {
            self.index += 1;
            return Some(self.rope.0.byte(self.index - 1));
        }
    }

    pub fn prev_byte(&mut self) -> Option<u8> {
        if self.index == 0 {
            return None;
        } else {
            self.index -= 1;
            return Some(self.rope.0.byte(self.index));
        }
    }

    pub fn next_char(&mut self) -> Option<char> {
        if self.index == self.rope.0.len_chars() {
            return None;
        } else {
            self.index += 1;
            return Some(self.rope.0.char(self.index - 1));
        }
    }

    pub fn prev_char(&mut self) -> Option<char> {
        if self.index == 0 {
            return None;
        } else {
            self.index -= 1;
            return Some(self.rope.0.char(self.index));
        }
    }
}
