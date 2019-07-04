use gc_derive::{Trace, Finalize};

use crate::gc_foreign::OrdSet;

// FIXME this doesn't uphold the time complexity guarantees, next and prev always take O(log n)
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub struct SetCursor<T: Ord + Clone> {
    index: Option<T>, // Some(t): just before elem t, None: just beyond the greatest element
    set: OrdSet<T>,
}

pub fn set_find_greater<T: Ord + Clone>(s: &OrdSet<T>, at: &T) -> Option<T> {
    if s.0.contains(at) {
        Some(at.clone())
    } else {
        set_find_strict_greater(s, at)
    }
}

pub fn set_find_lesser<T: Ord + Clone>(s: &OrdSet<T>, at: &T) -> Option<T> {
    if s.0.contains(at) {
        Some(at.clone())
    } else {
        set_find_strict_lesser(s, at)
    }
}

pub fn set_find_strict_greater<T: Ord + Clone>(s: &OrdSet<T>, at: &T) -> Option<T> {
    let (_, greater) = s.0.clone().split(at);
    greater.get_min().map(Clone::clone)
}

pub fn set_find_strict_lesser<T: Ord + Clone>(s: &OrdSet<T>, at: &T) -> Option<T> {
    let (less, _) = s.0.clone().split(at);
    less.get_max().map(Clone::clone)
}

impl<T: Clone + Ord> SetCursor<T> {
    pub fn new_min(v: OrdSet<T>) -> SetCursor<T> {
        SetCursor {
            index: v.0.get_min().map(Clone::clone),
            set: v,
        }
    }

    pub fn new_max(v: OrdSet<T>) -> SetCursor<T> {
        SetCursor {
            index: None,
            set: v,
        }
    }

    pub fn new_less_strict(v: OrdSet<T>, at: &T) -> SetCursor<T> {
        match set_find_strict_lesser(&v, at) {
            None => SetCursor {
                index: v.0.get_min().map(Clone::clone),
                set: v,
            },
            Some(lesser) => {
                let mut c = SetCursor {
                    index: Some(lesser.clone()),
                    set: v,
                };
                c.next();
                c
            }
        }
    }

    pub fn new_greater_strict(v: OrdSet<T>, at: &T) -> SetCursor<T> {
        match set_find_strict_greater(&v, at) {
            None => SetCursor {
                index: None,
                set: v,
            },
            Some(greater) => {
                SetCursor {
                    index: Some(greater.clone()),
                    set: v,
                }
            }
        }
    }

    pub fn new_less(v: OrdSet<T>, at: &T) -> SetCursor<T> {
        match set_find_lesser(&v, at) {
            None => SetCursor {
                index: v.0.get_min().map(Clone::clone),
                set: v,
            },
            Some(lesser) => {
                let mut c = SetCursor {
                    index: Some(lesser.clone()),
                    set: v,
                };
                c.next();
                c
            }
        }
    }

    pub fn new_greater(v: OrdSet<T>, at: &T) -> SetCursor<T> {
        match set_find_greater(&v, at) {
            None => SetCursor {
                index: None,
                set: v,
            },
            Some(greater) => {
                SetCursor {
                    index: Some(greater.clone()),
                    set: v,
                }
            }
        }
    }

    pub fn next(&mut self) -> Option<T> {
        match &self.index {
            Some(index) => {
                let ret = index.clone();
                self.index = set_find_strict_greater(&self.set, &ret);
                return Some(ret);
            }
            None => return None,
        }
    }

    pub fn prev(&mut self) -> Option<T> {
        match &self.index {
            None => {
                match self.set.0.get_max().map(Clone::clone) {
                    Some(max) => {
                        self.index = Some(max.clone());
                        return Some(max);
                    }
                    None => return None,
                }
            }
            Some(index) => {
                match set_find_strict_lesser(&self.set, &index) {
                    Some(previous) => {
                        self.index = Some(previous.clone());
                        return Some(previous);
                    }
                    None => return None,
                }
            }
        }
    }
}
