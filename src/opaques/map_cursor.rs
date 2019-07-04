use gc_derive::{Trace, Finalize};

use crate::gc_foreign::OrdMap;

// FIXME this doesn't uphold the time complexity guarantees, next and prev always take O(log n)
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub struct MapCursor<K: Ord + Clone, V: Clone> {
    index: Option<K>, // Some(k): just before key k, None: just beyond the greatest key
    map: OrdMap<K, V>,
}

pub fn map_find_greater<K: Ord + Clone, V: Clone>(m: &OrdMap<K, V>, at: &K) -> Option<K> {
    if m.0.contains_key(at) {
        Some(at.clone())
    } else {
        map_find_strict_greater(m, at)
    }
}

pub fn map_find_lesser<K: Ord + Clone, V: Clone>(m: &OrdMap<K, V>, at: &K) -> Option<K> {
    if m.0.contains_key(at) {
        Some(at.clone())
    } else {
        map_find_strict_lesser(m, at)
    }
}

pub fn map_find_strict_greater<K: Ord + Clone, V: Clone>(m: &OrdMap<K, V>, at: &K) -> Option<K> {
    let (_, greater) = m.0.clone().split(at);
    greater.get_min().map(|(k, _)| k.clone())
}

pub fn map_find_strict_lesser<K: Ord + Clone, V: Clone>(m: &OrdMap<K, V>, at: &K) -> Option<K> {
    let (less, _) = m.0.clone().split(at);
    less.get_max().map(|(k, _)| k.clone())
}

impl<K: Ord + Clone, V: Clone> MapCursor<K, V> {
    pub fn new_min(v: OrdMap<K, V>) -> MapCursor<K, V> {
        MapCursor {
            index: v.0.get_min().map(|(k, _)| k.clone()),
            map: v,
        }
    }

    pub fn new_max(v: OrdMap<K, V>) -> MapCursor<K, V> {
        MapCursor {
            index: None,
            map: v,
        }
    }

    pub fn new_less_strict(v: OrdMap<K, V>, at: &K) -> MapCursor<K, V> {
        match map_find_strict_lesser(&v, at) {
            None => MapCursor {
                index: v.0.get_min().map(|(k, _)| k.clone()),
                map: v,
            },
            Some(lesser) => {
                let mut c = MapCursor {
                    index: Some(lesser.clone()),
                    map: v,
                };
                c.next();
                c
            }
        }
    }

    pub fn new_greater_strict(v: OrdMap<K, V>, at: &K) -> MapCursor<K, V> {
        match map_find_strict_greater(&v, at) {
            None => MapCursor {
                index: None,
                map: v,
            },
            Some(greater) => {
                MapCursor {
                    index: Some(greater.clone()),
                    map: v,
                }
            }
        }
    }

    pub fn new_less(v: OrdMap<K, V>, at: &K) -> MapCursor<K, V> {
        match map_find_lesser(&v, at) {
            None => MapCursor {
                index: v.0.get_min().map(|(k, _)| k.clone()),
                map: v,
            },
            Some(lesser) => {
                let mut c = MapCursor {
                    index: Some(lesser.clone()),
                    map: v,
                };
                c.next();
                c
            }
        }
    }

    pub fn new_greater(v: OrdMap<K, V>, at: &K) -> MapCursor<K, V> {
        match map_find_greater(&v, at) {
            None => MapCursor {
                index: None,
                map: v,
            },
            Some(greater) => {
                MapCursor {
                    index: Some(greater.clone()),
                    map: v,
                }
            }
        }
    }

    pub fn next(&mut self) -> Option<(K, &V)> {
        match &self.index {
            Some(index) => {
                let ret = index.clone();
                self.index = map_find_strict_greater(&self.map, &ret);
                return Some((ret.clone(), self.map.0.get(&ret).unwrap()));
            }
            None => return None,
        }
    }

    pub fn prev(&mut self) -> Option<(K, &V)> {
        match &self.index {
            None => {
                match self.map.0.get_max().map(|(k, _)| k.clone()) {
                    Some(max) => {
                        self.index = Some(max.clone());
                        return Some((max.clone(), self.map.0.get(&max).unwrap()));
                    }
                    None => return None,
                }
            }
            Some(index) => {
                match map_find_strict_lesser(&self.map, &index) {
                    Some(previous) => {
                        self.index = Some(previous.clone());
                        return Some((previous.clone(), self.map.0.get(&previous).unwrap()));
                    }
                    None => return None,
                }
            }
        }
    }
}
