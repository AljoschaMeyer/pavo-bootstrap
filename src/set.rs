// 2-3 tree set

use std::cmp::Ordering::{self, *};

use gc::{Gc, Trace, Finalize, custom_trace};
use gc_derive::{Trace, Finalize};

use crate::value::Value;

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Set(Node, u8 /* height */);

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Node {
    Leaf,
    N2(Gc<(Node, Value, Node, usize /* count */)>),
    N3(Gc<(Node, Value, Node, Value, Node, usize /* count */)>),
}
use self::Node::*;

fn n2(l: Node, k: Value, r: Node) -> Node {
    let c = l.count() + 1 + r.count();
    N2(Gc::new((l, k, r, c)))
}

fn n3(l: Node, lk: Value, m: Node, rk: Value, r: Node) -> Node {
    let c = l.count() + 1 + m.count() + 1 + r.count();
    N3(Gc::new((l, lk, m, rk, r, c)))
}

impl Set {
    pub fn new() -> Self {
        Set(Leaf, 0)
    }

    pub fn singleton(k: Value) -> Self {
        Set(n2(Leaf, k, Leaf), 1)
    }

    pub fn count(&self) -> usize {
        self.0.count()
    }

    pub fn contains(&self, kx: &Value) -> bool {
        if self.is_empty() {
            false
        } else {
            self.0.contains(kx)
        }
    }

    pub fn insert(&self, kx: Value) -> Self {
        if self.is_empty() {
            Self::singleton(kx)
        } else {
            match self.0.insert(kx) {
                Insert::Done(done_n) => Set(
                    done_n, self.1
                ),
                Insert::Up(l, k, r) => Set(
                    n2(l.clone(), k, r.clone()),
                    self.1 + 1,
                ),
            }
        }
    }

    pub fn remove(&self, kx: &Value) -> Self {
        if self.is_empty() {
            Self::new()
        } else {
            match self.0.remove(kx) {
                Remove::Done(done_n) => Set(
                    done_n, self.1
                ),
                Remove::Up(up_n) => Set(up_n, self.1 - 1),
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub fn iter(&self) -> Iter {
        Iter::new(&self.0)
    }

    pub fn cursor_min(&self) -> Option<Cursor> {
        if self.is_empty() {
            None
        } else {
            Some(Cursor::new_min(&self.0))
        }
    }

    pub fn cursor_max(&self) -> Option<Cursor> {
        if self.is_empty() {
            None
        } else {
            Some(Cursor::new_max(&self.0))
        }
    }

    pub fn split(&self, kx: &Value) -> (Set, Option<Value>, Set) {
        match &self.0 {
            Leaf => (Set::new(), None, Set::new()),
            N2(n) => {
                let (ref l, ref k, ref r, _) = &(**n);
                match kx.cmp(k) {
                    Less => {
                        let (ll, lm, lr) = Set(l.clone(), self.1 - 1).split(kx);
                        return (
                            ll,
                            lm.clone(),
                            join(&lr.0, lr.1, k.clone(), r, self.1 - 1),
                        );
                    }
                    Equal => (
                        Set(l.clone(), self.1 - 1),
                        Some(k.clone()),
                        Set(r.clone(), self.1 - 1),
                    ),
                    Greater => {
                        let (rl, rm, rr) = Set(r.clone(), self.1 - 1).split(kx);
                        return (
                            join(l, self.1 - 1, k.clone(), &rl.0, rl.1),
                            rm.clone(),
                            rr,
                        );
                    }
                }
            }
            N3(n) => {
                let (ref l, ref lk, ref m, ref rk, ref r, _) = &(**n);
                match kx.cmp(lk) {
                    Less => {
                        let (ll, lm, lr) = Set(l.clone(), self.1 - 1).split(kx);
                        let tmp = join(&lr.0, lr.1, lk.clone(), m, self.1 - 1);
                        return (
                            ll,
                            lm.clone(),
                            join(
                                &tmp.0, tmp.1,
                                rk.clone(),
                                r, self.1 - 1,
                            ),
                        );
                    }
                    Equal => (
                        Set(l.clone(), self.1 - 1),
                        Some(lk.clone()),
                        Set(n2(m.clone(), rk.clone(), r.clone()), self.1),
                    ),
                    Greater => match kx.cmp(rk) {
                        Less => {
                            let (ml, mm, mr) = Set(m.clone(), self.1 - 1).split(kx);
                            return (
                                join(l, self.1 - 1, lk.clone(), &ml.0, ml.1),
                                mm.clone(),
                                join(&mr.0, mr.1, rk.clone(), r, self.1 - 1),
                            );
                        }
                        Equal => (
                            Set(n2(l.clone(), lk.clone(), m.clone()), self.1),
                            Some(rk.clone()),
                            Set(r.clone(), self.1 - 1),
                        ),
                        Greater => {
                            let (rl, rm, rr) = Set(r.clone(), self.1 - 1).split(kx);
                            let tmp = join(m, self.1 - 1, rk.clone(), &rl.0, rl.1);
                            return (
                                join(
                                    l, self.1 - 1,
                                    lk.clone(),
                                    &tmp.0, tmp.1,
                                ),
                                rm.clone(),
                                rr,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn is_subset(&self, other: &Set) -> bool {
        unimplemented!();
        // if other.count() > self.count() {
        //     return false;
        // } else if self.is_empty() {
        //     return true;
        // } else {
        //     for k in other.iter() {
        //         match self.get(&k) {
        //             None => return false,
        //             Some(actual_v) => if v != *actual_v {
        //                 return false;
        //             }
        //         }
        //     }
        //     return true;
        // }
    }

    // prefers items in self
    pub fn union(&self, other: &Set) -> Set {
        if self.is_empty() {
            return other.clone();
        } else if other.is_empty() {
            return self.clone();
        } else {
            let other_root = other.root();
            let (lm, x, rm) = self.split(other_root);
            let nl = other.left().union(&lm);
            let nr = other.right().union(&rm);
            let nroot = match &x {
                None => other_root,
                Some(k) => k,
            };
            return join(&nl.0, nl.1, nroot.clone(), &nr.0, nr.1);
        }
    }

    pub fn intersection(&self, other: &Set) -> Set {
        if self.is_empty() || other.is_empty() {
            Self::new()
        } else {
            let other_root = other.root();
            let (lm, x, rm) = self.split(other_root);
            let nl = other.left().intersection(&lm);
            let nr = other.right().intersection(&rm);
            match &x {
                Some(k) => return join(&nl.0, nl.1, k.clone(), &nr.0, nr.1),
                None => return join2(&nl.0, nl.1, &nr.0, nr.1),
            }
        }
    }

    pub fn difference(&self, other: &Set) -> Set {
        if self.is_empty() || other.is_empty() {
            return self.clone();
        } else {
            let other_root = other.root();
            let (lm, x, rm) = self.split(other_root);
            let nl = lm.difference(&other.left());
            let nr = rm.difference(&other.right());
            return join2(&nl.0, nl.1, &nr.0, nr.1);
        }
    }

    pub fn symmetric_difference(&self, other: &Set) -> Set {
        if self.is_empty() {
            return other.clone();
        } else if other.is_empty() {
            return self.clone();
        } else {
            let other_root = other.root();
            let (lm, x, rm) = self.split(other_root);
            let nl = lm.symmetric_difference(&other.left());
            let nr = rm.symmetric_difference(&other.right());
            match &x {
                Some(_) => return join2(&nl.0, nl.1, &nr.0, nr.1),
                None => return join(&nl.0, nl.1, other_root.clone(), &nr.0, nr.1),
            }
        }
    }

    fn root(&self) -> &Value {
        match &self.0 {
            Leaf => unreachable!(),
            N2(n) => &n.1,
            N3(n) => &n.1,
        }
    }

    fn left(&self) -> Self {
        match &self.0 {
            Leaf => unreachable!(),
            N2(n) => Set(n.0.clone(), self.1 - 1),
            N3(n) => Set(n.0.clone(), self.1 - 1),
        }
    }

    fn right(&self) -> Self {
        match &self.0 {
            Leaf => unreachable!(),
            N2(n) => Set(n.2.clone(), self.1 - 1),
            N3(n) => Set(n2(n.2.clone(), n.3.clone(), n.4.clone()), self.1),
        }
    }
}

fn join(lesser: &Node, lh: u8, k: Value, greater: &Node, gh: u8) -> Set {
    if lesser.is_empty() {
        return Set(greater.clone(), gh).insert(k);
    } else if greater.is_empty() {
        return Set(lesser.clone(), lh).insert(k);
    } else {
        match lh.cmp(&gh) {
            Less => match join_lesser_smaller(lesser, k, greater, gh - lh) {
                Insert::Done(done_n) => Set(done_n, gh),
                Insert::Up(l, k, r) => Set(
                    n2(
                        l.clone(),
                        /**/ k,
                        r.clone(),
                    ),
                    gh + 1,
                ),
            }
            Equal => Set(
                n2(
                    lesser.clone(),
                    /**/ k,
                    greater.clone(),
                ),
                gh + 1,
            ),
            Greater => match join_greater_smaller(lesser, k, greater, lh - gh) {
                Insert::Done(done_n) => Set(done_n, lh),
                Insert::Up(l, k, r) => Set(
                    n2(
                        l.clone(),
                        /**/ k,
                        r.clone(),
                    ),
                    lh + 1,
                ),
            }
        }
    }
}

fn join2(lesser: &Node, lh: u8, greater: &Node, gh: u8) -> Set {
    if lesser.is_empty() {
        return Set(greater.clone(), gh);
    } else {
        let max = lesser.get_max();
        let nl = Set(lesser.clone(), lh).remove(max);
        return join(&nl.0, nl.1, max.clone(), greater, gh);
    }
}

impl Node {
    fn count(&self) -> usize {
        match self {
            Leaf => 0,
            N2(n) => n.3,
            N3(n) => n.5,
        }
    }

    fn is_empty(&self) -> bool {
        self.count() == 0
    }

    fn contains(&self, kx: &Value) -> bool {
        match self {
            Leaf => false,
            N2(n) => {
                let (ref l, ref k, ref r, _) = &(**n);
                match kx.cmp(k) {
                   Less => l.contains(kx),
                   Equal => true,
                   Greater => r.contains(kx),
               }
            }
            N3(n) => {
                let (ref l, ref lk, ref m, ref rk, ref r, _) = &(**n);
                match kx.cmp(lk) {
                    Less => l.contains(kx),
                    Equal => true,
                    Greater => match kx.cmp(rk) {
                        Less => m.contains(kx),
                        Equal => true,
                        Greater => r.contains(kx),
                    }
                }
            }
        }
    }

    fn get_min(&self) -> &Value {
        match self {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref l, ref k, _, _) = &(**n);
                if l.is_leaf() {
                    k
                } else {
                    l.get_min()
                }
            }
            N3(n) => {
                let (ref l, ref lk, _, _, _, _) = &(**n);
                if l.is_leaf() {
                    lk
                } else {
                    l.get_min()
                }
            }
        }
    }

    fn get_max(&self) -> &Value {
        match self {
            Leaf => unreachable!(),
            N2(n) => {
                let (_, ref k, ref r, _) = &(**n);
                if r.is_leaf() {
                    k
                } else {
                    r.get_max()
                }
            }
            N3(n) => {
                let (_, _,  _, ref rk, ref r, _) = &(**n);
                if r.is_leaf() {
                    rk
                } else {
                    r.get_max()
                }
            }
        }
    }

    fn insert(&self, kx: Value) -> Insert {
        match self {
            Leaf => Insert::Up(Leaf, kx, Leaf),
            N2(n) => {
                let (ref l, ref k, ref r, _) = &(**n);
                match kx.cmp(k) {
                    Less => n2_handle_insert_l(l.insert(kx), k, r),
                    Equal => Insert::Done(n2(
                        l.clone(),
                        /**/ kx,
                        r.clone(),
                    )),
                    Greater => n2_handle_insert_r(l, k, r.insert(kx))
                }
            }
            N3(n) => {
                let (ref l, ref lk, ref m, ref rk, ref r, _) = &(**n);
                match kx.cmp(lk) {
                    Less => n3_handle_insert_l(l.insert(kx), lk, m, rk, r),
                    Equal => Insert::Done(n3(
                        l.clone(),
                        /**/ kx,
                        m.clone(),
                        /**/ rk.clone(),
                        r.clone(),
                    )),
                    Greater => match kx.cmp(rk) {
                        Less => n3_handle_insert_m(l, lk, m.insert(kx), rk, r),
                        Equal => Insert::Done(n3(
                            l.clone(),
                            /**/ lk.clone(),
                            m.clone(),
                            /**/ kx,
                            r.clone(),
                        )),
                        Greater => n3_handle_insert_r(l, lk, m, rk, r.insert(kx)),
                    }
                }
            }
        }
    }

    fn remove(&self, kx: &Value) -> Remove {
        match self {
            Leaf => Remove::Done(Leaf),
            N2(n) => {
                let (ref l, ref k, ref r, _) = &(**n);
                match kx.cmp(k) {
                    Less => n2_handle_remove_l(l.remove(kx), k, r),
                    Equal => if r.is_leaf() {
                        Remove::Up(Leaf)
                    } else {
                        let new_k = l.get_max();
                        n2_handle_remove_l(l.remove_max(), new_k, r)
                    }
                    Greater => n2_handle_remove_r(l, k, r.remove(kx)),
                }
            }
            N3(n) => {
                let (ref l, ref lk, ref m, ref rk, ref r, _) = &(**n);
                match kx.cmp(lk) {
                    Less => n3_handle_remove_l(l.remove(kx), lk, m, rk, r),
                    Equal => if m.is_leaf() {
                        Remove::Done(n2(Leaf, rk.clone(), Leaf))
                    } else {
                        let new_k = l.get_max();
                        n3_handle_remove_l(l.remove_max(), new_k,  m, rk, r)
                    }
                    Greater => match kx.cmp(rk) {
                        Less => n3_handle_remove_m(l, lk, m.remove(kx), rk, r),
                        Equal => if r.is_leaf() {
                            Remove::Done(n2(Leaf, lk.clone(), Leaf))
                        } else {
                            let new_k = m.get_max();
                            n3_handle_remove_m(l, lk, m.remove_max(), new_k, r)
                        }
                        Greater => n3_handle_remove_r(l, lk, m, rk, r.remove(kx)),
                    }
                }
            }
        }
    }

    fn remove_min(&self) -> Remove {
        let min_k = self.get_min();
        return self.remove(min_k);
    }

    fn remove_max(&self) -> Remove {
        let max_k = self.get_max();
        return self.remove(max_k);
    }

    fn is_leaf(&self) -> bool {
        match self {
            Leaf => true,
            _ => false,
        }
    }

    // appends the leftmost path from self (inclusive) to a leaf (exclusive) of positions
    fn leftmost_positions(&self, out: &mut Vec<Position>) {
        match self {
            Leaf => {}
            N2(n) => {
                out.push(N2Left(n.clone()));
                n.0.leftmost_positions(out);
            }
            N3(n) => {
                out.push(N3Left(n.clone()));
                n.0.leftmost_positions(out);
            }
        }
    }

    // appends the rightmost path from self (inclusive) to a leaf (exclusive) of positions
    // (places the cursor *before* the rightmost element)
    fn rightmost_positions(&self, out: &mut Vec<Position>) {
        match self {
            Leaf => {}
            N2(n) => {
                if n.2.is_leaf() {
                    out.push(N2KV(n.clone()));
                } else {
                    out.push(N2Right(n.clone()));
                    n.2.rightmost_positions(out);
                }
            }
            N3(n) => {
                if n.4.is_leaf() {
                    out.push(N3RKV(n.clone()));
                } else {
                    out.push(N3Right(n.clone()));
                    n.4.rightmost_positions(out);
                }
            }
        }
    }
}

fn n2_handle_insert_l(insert_l: Insert, k: &Value, r: &Node) -> Insert {
    match insert_l {
        Insert::Done(done_n) => Insert::Done(n2(
            done_n,
            /**/ k.clone(),
            r.clone(),
        )),
        Insert::Up(up_l, up_k, up_r) => Insert::Done(n3(
            up_l,
            /**/ up_k,
            up_r,
            /**/ k.clone(),
            r.clone(),
        )),
    }
}

fn n2_handle_insert_r(l: &Node, k: &Value, insert_r: Insert) -> Insert {
    match insert_r {
        Insert::Done(done_n) => Insert::Done(n2(
            l.clone(),
            /**/ k.clone(),
            done_n,
        )),
        Insert::Up(up_l, up_k, up_r) => Insert::Done(n3(
            l.clone(),
            /**/ k.clone(),
            up_l,
            /**/ up_k,
            up_r,
        )),
    }
}

fn n3_handle_insert_l(
    insert_l: Insert, lk: &Value, m: &Node, rk: &Value, r: &Node
) -> Insert {
    match insert_l {
        Insert::Done(done_n) => Insert::Done(n3(
            done_n,
            /**/ lk.clone(),
            m.clone(),
            /**/ rk.clone(),
            r.clone(),
        )),
        Insert::Up(up_l, up_k, up_r) => Insert::Up(
                n2(up_l, up_k, up_r),
                /**/ lk.clone(),
                n2(m.clone(), rk.clone(), r.clone()),
            ),
    }
}

fn n3_handle_insert_m(
    l: &Node, lk: &Value, insert_m: Insert, rk: &Value, r: &Node
) -> Insert {
    match insert_m {
        Insert::Done(done_n) => Insert::Done(n3(
            l.clone(),
            /**/ lk.clone(),
            done_n,
            /**/ rk.clone(),
            r.clone(),
        )),
        Insert::Up(up_l, up_k, up_r) => Insert::Up(
            n2(l.clone(), lk.clone(), up_l),
            /**/ up_k.clone(),
            n2(up_r.clone(), rk.clone(), r.clone()),
        ),
    }
}

fn n3_handle_insert_r(
    l: &Node, lk: &Value, m: &Node, rk: &Value, insert_r: Insert
) -> Insert {
    match insert_r {
        Insert::Done(done_n) => Insert::Done(n3(
            l.clone(),
            /**/ lk.clone(),
            m.clone(),
            /**/ rk.clone(),
            done_n,
        )),
        Insert::Up(up_l, up_k, up_r) => Insert::Up(
            n2(l.clone(), lk.clone(), m.clone()),
            /**/ rk.clone(),
            n2(up_l, up_k, up_r),
        ),
    }
}

fn n2_handle_remove_l(remove_l: Remove, k: &Value, r: &Node) -> Remove {
    match remove_l {
        Remove::Done(done_n) => Remove::Done(n2(
            done_n,
            /**/ k.clone(),
            r.clone(),
        )),
        Remove::Up(up_n) => match r {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref rl, ref rk, ref rr, _) = &(**n);
                Remove::Up(n3(
                    up_n,
                    /**/ k.clone(),
                    rl.clone(),
                    /**/ rk.clone(),
                    rr.clone(),
                ))
            }
            N3(n) => {
                let (ref rl, ref rlk, ref rm, ref rrk, ref rr, _) = &(**n);
                Remove::Done(n2(
                    n2(
                        up_n,
                        /**/ k.clone(),
                        rl.clone(),
                    ),
                    /**/ rlk.clone(),
                    n2(
                        rm.clone(),
                        /**/ rrk.clone(),
                        rr.clone(),
                    ),
                ))
            }
        }
    }
}

fn n2_handle_remove_r(l: &Node, k: &Value, remove_r: Remove) -> Remove {
    match remove_r {
        Remove::Done(done_n) => Remove::Done(n2(
            l.clone(),
            /**/ k.clone(),
            done_n,
        )),
        Remove::Up(up_n) => match l {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref ll, ref lk, ref lr, _) = &(**n);
                Remove::Up(n3(
                    ll.clone(),
                    /**/ lk.clone(),
                    lr.clone(),
                    /**/ k.clone(),
                    up_n,
                ))
            }
            N3(n) => {
                let (ref ll, ref llk, ref lm, ref lrk, ref lr, _) = &(**n);
                Remove::Done(n2(
                    n2(
                        ll.clone(),
                        /**/ llk.clone(),
                        lm.clone(),
                    ),
                    /**/ lrk.clone(),
                    n2(
                        lr.clone(),
                        /**/ k.clone(),
                        up_n,
                    ),
                ))
            }
        }
    }
}

fn n3_handle_remove_l(
    remove_l: Remove, lk: &Value, m: &Node, rk: &Value, r: &Node
) -> Remove {
    match remove_l {
        Remove::Done(done_n) => Remove::Done(n3(
            done_n,
            /**/ lk.clone(),
            m.clone(),
            /**/ rk.clone(),
            r.clone(),
        )),
        Remove::Up(up_n) => match m {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref ml, ref mk, ref mr, _) = &(**n);
                Remove::Done(n2(
                    n3(
                        up_n,
                        /**/ lk.clone(),
                        ml.clone(),
                        /**/ mk.clone(),
                        mr.clone(),
                    ),
                    /**/ rk.clone(),
                    r.clone()
                ))
            }
            N3(n) => {
                let (ref ml, ref mlk, ref mm, ref mrk, ref mr, _) = &(**n);
                Remove::Done(n3(
                    n2(
                        up_n,
                        /**/ lk.clone(),
                        ml.clone(),
                    ),
                    /**/ mlk.clone(),
                    n2(mm.clone(),
                    /**/ mrk.clone(),
                    mr.clone(),),
                    /**/ rk.clone(),
                    r.clone(),
                ))
            }
        }
    }
}

fn n3_handle_remove_m(
    l: &Node, lk: &Value, remove_m: Remove, rk: &Value, r: &Node
) -> Remove {
    match remove_m {
        Remove::Done(done_n) => Remove::Done(n3(
            l.clone(),
            /**/ lk.clone(),
            done_n,
            /**/ rk.clone(),
            r.clone(),
        )),
        Remove::Up(up_n) => match r {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref rl, ref rk_, ref rr, _) = &(**n);
                Remove::Done(n2(
                    l.clone(),
                    /**/ lk.clone(),
                    n3(
                        up_n,
                        /**/ rk.clone(),
                        rl.clone(),
                        /**/ rk_.clone(),
                        rr.clone(),
                    ),
                ))
            }
            N3(n) => {
                let (ref rl, ref rlk, ref rm, ref rrk, ref rr, _) = &(**n);
                Remove::Done(n3(
                    l.clone(),
                    /**/ lk.clone(),
                    n2(
                        up_n,
                        /**/ rk.clone(),
                        rl.clone(),
                    ),
                    /**/ rlk.clone(),
                    n2(
                        rm.clone(),
                        /**/ rrk.clone(),
                        rr.clone(),
                    ),
                ))
            }
        }
    }
}

fn n3_handle_remove_r(
    l: &Node, lk: &Value, m: &Node, rk: &Value, remove_r: Remove
) -> Remove {
    match remove_r {
        Remove::Done(done_n) => Remove::Done(n3(
            l.clone(),
            /**/ lk.clone(),
            m.clone(),
            /**/ rk.clone(),
            done_n,
        )),
        Remove::Up(up_n) => match m {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref ml, ref mk, ref mr, _) = &(**n);
                Remove::Done(n2(
                    l.clone(),
                    /**/ lk.clone(),
                    n3(
                        ml.clone(),
                        /**/ mk.clone(),
                        mr.clone(),
                        /**/ rk.clone(),
                        up_n,
                    ),
                ))
            }
            N3(n) => {
                let (ref ml, ref mlk, ref mm, ref mrk, ref mr, _) = &(**n);
                Remove::Done(n3(
                    l.clone(),
                    /**/ lk.clone(),
                    n2(
                        ml.clone(),
                        /**/ mlk.clone(),
                        mm.clone(),
                    ),
                    /**/ mrk.clone(),
                    n2(
                        mr.clone(),
                        /**/ rk.clone(),
                        up_n,
                    ),
                ))
            }
        }
    }
}

// traverse left spine of greater node for h_diff, then merge
fn join_lesser_smaller(lesser: &Node, k: Value, greater: &Node, h_diff: u8) -> Insert {
    if h_diff == 0 {
        Insert::Up(lesser.clone(), k, greater.clone())
    } else {
        match greater {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref gl, ref gk, ref gr, _) = &(**n);
                n2_handle_insert_l(
                    join_lesser_smaller(lesser, k, gl, h_diff - 1), gk, gr
                )
            }
            N3(n) => {
                let (ref gl, ref glk, ref gm, ref grk, ref gr, _) = &(**n);
                n3_handle_insert_l(
                    join_lesser_smaller(
                        lesser, k, gl, h_diff - 1
                    ),
                    glk, gm, grk, gr,
                )
            }
        }
    }
}

fn join_greater_smaller(lesser: &Node, k: Value, greater: &Node, h_diff: u8) -> Insert {
    if h_diff == 0 {
        Insert::Up(lesser.clone(), k, greater.clone())
    } else {
        match lesser {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref ll, ref lk, ref lr, _) = &(**n);
                n2_handle_insert_r(
                    ll, lk, join_greater_smaller(lr, k, greater, h_diff - 1)
                )
            }
            N3(n) => {
                let (ref ll, ref llk, ref lm, lrk, lr, _) = &(**n);
                n3_handle_insert_r(
                    ll, llk, lm, lrk,
                    join_greater_smaller(
                        lr, k, greater, h_diff - 1
                    ),
                )
            }
        }
    }
}

enum Insert {
    Done(Node),
    Up(Node, Value, Node),
}

enum Remove {
    Done(Node),
    Up(Node),
}

#[derive(Debug, Clone)]
pub struct Cursor(Vec<Position>);

#[derive(Debug, Clone)]
enum Position {
    N2Left(Gc<(Node, Value, Node, usize /* count */)>),
    N2KV(Gc<(Node, Value, Node, usize /* count */)>),
    N2Right(Gc<(Node, Value, Node, usize /* count */)>),
    N2Post(Gc<(Node, Value, Node, usize /* count */)>),
    N3Left(Gc<(Node, Value, Node, Value, Node, usize /* count */)>),
    N3LKV(Gc<(Node, Value, Node, Value, Node, usize /* count */)>),
    N3Middle(Gc<(Node, Value, Node, Value, Node, usize /* count */)>),
    N3RKV(Gc<(Node, Value, Node, Value, Node, usize /* count */)>),
    N3Right(Gc<(Node, Value, Node, Value, Node, usize /* count */)>),
    N3Post(Gc<(Node, Value, Node, Value, Node, usize /* count */)>),
}
use self::Position::*;

impl Position {
    fn is_post(&self) -> bool {
        match self {
            N2Post(_) | N3Post(_) => true,
            _ => false,
        }
    }

    fn is_pre(&self) -> bool {
        match self {
            N2Left(_) | N3Left(_) => true,
            _ => false,
        }
    }

    // true if it changed, false if not
    fn step_next(positions: &mut Vec<Position>) -> bool {
        let len = positions.len();
        let must_post = len == 1 || positions[len - 2].is_post();
        let p = positions[len - 1].clone();

        match p {
            N2Left(n) => {
                if n.0.is_leaf() {
                    positions[len - 1] = N2KV(n);
                    return Position::step_next(positions);
                } else {
                    positions[len - 1] = N2KV(n);
                    return true;
                }
            }
            N2KV(n) => {
                if n.2.is_leaf() {
                    positions[len - 1] = N2Right(n);
                    return true;
                } else {
                    positions[len - 1] = N2Right(n.clone());
                    n.2.leftmost_positions(positions);
                    return true;
                }
            },
            N2Right(n) => {
                if must_post {
                    positions[len - 1] = N2Post(n);
                    return Position::step_next(positions);
                } else {
                    positions.pop();
                    return Position::step_next(positions);
                }
            }
            N2Post(_) => return false,
            N3Left(n) => {
                if n.0.is_leaf() {
                    positions[len - 1] = N3Middle(n.clone());
                    n.2.leftmost_positions(positions);
                    return true;
                } else {
                    positions[len - 1] = N3LKV(n);
                    return true;
                }
            }
            N3LKV(n) => {
                if n.2.is_leaf() {
                    positions[len - 1] = N3RKV(n);
                    return true;
                } else {
                    positions[len - 1] = N3Middle(n.clone());
                    n.2.leftmost_positions(positions);
                    return true;
                }
            },
            N3Middle(n) => {
                if n.2.is_leaf() {
                    positions[len - 1] = N3RKV(n);
                    return Position::step_next(positions);
                } else {
                    positions[len - 1] = N3RKV(n);
                    return true;
                }
            }
            N3RKV(n) => {
                if n.4.is_leaf() {
                    positions[len - 1] = N3Right(n);
                    return true;
                } else {
                    positions[len - 1] = N3Right(n.clone());
                    n.4.leftmost_positions(positions);
                    return true;
                }
            },
            N3Right(n) => {
                if must_post {
                    positions[len - 1] = N3Post(n);
                    return Position::step_next(positions);
                } else {
                    positions.pop();
                    return Position::step_next(positions);
                }
            }
            N3Post(_) => return false,
        };
    }

    fn step_prev(positions: &mut Vec<Position>) -> bool {
        let len = positions.len();
        let must_stay = len == 1 || positions[len - 2].is_post();
        let p = positions[len - 1].clone();

        match p {
            N2Left(_) => {
                if positions.iter().rev().all(|p| p.is_pre()) {
                    return false;
                } else {
                    positions.pop();
                    return Position::step_prev(positions);
                }
            }
            N2KV(n) => {
                if n.0.is_leaf() {
                    positions[len - 1] = N2Left(n);
                    Position::step_prev(positions);
                    return true;
                } else {
                    positions[len - 1] = N2Left(n.clone());
                    n.0.rightmost_positions(positions);
                    return true;
                }
            },
            N2Right(n) => {
                if n.2.is_leaf() {
                    positions[len - 1] = N2KV(n);
                    return Position::step_prev(positions);
                } else {
                    positions[len - 1] = N2KV(n);
                    return true;
                }
            }
            N2Post(n) => {
                if n.2.is_leaf() {
                    positions[len - 1] = N2KV(n);
                    return true;
                } else {
                    positions[len - 1] = N2Right(n.clone());
                    n.2.rightmost_positions(positions);
                    return true;
                }
            }
            N3Left(_) => {
                if positions.iter().rev().all(|p| p.is_pre()) {
                    return false;
                } else {
                    positions.pop();
                    return Position::step_prev(positions);
                }
            }
            N3LKV(n) => {
                if n.0.is_leaf() {
                    positions[len - 1] = N3Left(n);
                    Position::step_prev(positions);
                    return true;
                } else {
                    positions[len - 1] = N3Left(n.clone());
                    n.0.rightmost_positions(positions);
                    return true;
                }
            },
            N3Middle(n) => {
                if n.2.is_leaf() {
                    positions[len - 1] = N3LKV(n);
                    return Position::step_prev(positions);
                } else {
                    positions[len - 1] = N3LKV(n);
                    return true;
                }
            }
            N3RKV(n) => {
                if n.2.is_leaf() {
                    positions[len - 1] = N3LKV(n);
                    return true;
                } else {
                    positions[len - 1] = N3Middle(n.clone());
                    n.2.rightmost_positions(positions);
                    return true;
                }
            },
            N3Right(n) => {
                if n.4.is_leaf() {
                    positions[len - 1] = N3RKV(n);
                    return Position::step_prev(positions);
                } else {
                    positions[len - 1] = N3RKV(n);
                    return true;
                }
            }
            N3Post(n) => {
                if n.4.is_leaf() {
                    positions[len - 1] = N3RKV(n);
                    return true;
                } else {
                    positions[len - 1] = N3Right(n.clone());
                    n.4.rightmost_positions(positions);
                    return true;
                }
            }
        };
    }
}

impl Cursor {
    pub fn new_min(n: &Node) -> Self {
        let mut buf = Vec::new();
        n.leftmost_positions(&mut buf);
        return Cursor(buf);
    }

    pub fn new_max(n: &Node) -> Self {
        let mut buf = Vec::new();
        n.rightmost_positions(&mut buf);
        return Cursor(buf);
    }

    // TODO other creation functions

    pub fn current(&mut self) -> Option<Value> {
        let len = self.0.len();
        match &self.0[len - 1] {
            N2Left(n) => {
                self.0[len - 1] = N2KV(n.clone());
                return self.current();
            }
            N2KV(n) => return Some(n.1.clone()),
            N2Right(n) => {
                self.0[len - 1] = N2Post(n.clone());
                return self.current();
            }
            N3Left(n) => {
                self.0[len - 1] = N3LKV(n.clone());
                return self.current();
            }
            N3LKV(n) => return Some(n.1.clone()),
            N3Middle(n) => {
                self.0[len - 1] = N3RKV(n.clone());
                return self.current();
            }
            N3RKV(n) => return Some(n.3.clone()),
            N3Right(n) => {
                self.0[len - 1] = N3Post(n.clone());
                return self.current();
            }
            N2Post(_) | N3Post(_) => if len == 1 {
                return None;
            } else {
                let _ = self.0.pop();
                return self.current();
            }
        }
    }

    // true if it moved, false, if it was already at the end
    pub fn next(&mut self) -> bool {
        Position::step_next(&mut self.0)
    }

    // true if it moved, false, if it was already at the start
    pub fn prev(&mut self) -> bool {
        Position::step_prev(&mut self.0)
    }
}

impl PartialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        match (self.is_empty(), other.is_empty()) {
            (true, true) => true,
            (true, false) | (false, true) => false,
            (false, false) => self.0.eq(&other.0),
        }
    }
}
impl Eq for Set {}

impl PartialOrd for Set {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Set {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.is_empty(), other.is_empty()) {
            (true, true) => Equal,
            (true, false) => Less,
            (false, true) => Greater,
            (false, false) => self.0.cmp(&other.0),
        }
    }
}

impl PartialEq for Node {
    // Neither arg may be a leaf.
    fn eq(&self, other: &Self) -> bool {
        let mut ca = Cursor::new_min(&self);
        let mut cb = Cursor::new_min(&other);

        loop {
            match (ca.current(), cb.current()) {
                (None, None) => return true,
                (None, Some(_)) => return false,
                (Some(_), None) => return false,
                (Some(ka), Some(kb)) => {
                    if ka == kb {
                        continue
                    } else {
                        return false;
                    }
                }
            }
        }
    }
}
impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    // Neither arg may be a leaf.
    fn cmp(&self, other: &Self) -> Ordering {
        let mut ca = Cursor::new_min(&self);
        let mut cb = Cursor::new_min(&other);

        loop {
            match (ca.current(), cb.current()) {
                (None, None) => return Equal,
                (None, Some(_)) => return Less,
                (Some(_), None) => return Greater,
                (Some(ka), Some(kb)) => {
                    let compare_keys = ka.cmp(&kb);
                    match compare_keys {
                        Equal => {
                            ca.next();
                            cb.next();
                        }
                        _ => return compare_keys,
                    }
                }
            }
        }
    }
}

pub struct Iter(Option<Cursor>);

impl Iter {
    fn new(n: &Node) -> Self {
        if n.count() == 0 {
            Iter(None)
        } else {
            Iter(Some(Cursor::new_min(n)))
        }
    }
}

impl std::iter::Iterator for Iter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.as_mut() {
            None => None,
            Some(c) => {
                match c.current() {
                    None => None,
                    Some(current) => {
                        c.next();
                        Some(current)
                    }
                }
            }
        }
    }
}

//////////////////////////////////////// debug /testing stuff

pub fn map_to_vec(m: &Set, out: &mut Vec<Value>) {
    node_to_vec(&m.0, out)
}

fn node_to_vec(n: &Node, out: &mut Vec<Value>) {
    match n {
        Leaf => {},
        N2(n) => {
            let (ref l, ref k, ref r, _) = &(**n);
            node_to_vec(l, out);
            out.push(k.clone());
            node_to_vec(r, out);
        }
        N3(n) => {
            let (ref l, ref lk, ref m, ref rk, ref r, _) = &(**n);
            node_to_vec(l, out);
            out.push(lk.clone());
            node_to_vec(m, out);
            out.push(rk.clone());
            node_to_vec(r, out);
        }
    }
}


use std::collections::BTreeSet;

// fn fuzzy(data: &[u8]) {
//     // Foo
//     let mut control = BTreeSet::new();
//     let mut m = Set::new();
//
//     for b in data {
//         // m = m.insert(Value::int(*b as i64), Foo);
//         // control.insert(Value::int(*b as i64), Foo);
//
//         match *b {
//             0...63 => {
//                 m = m.insert(Value::int((b & 0b0011_1111) as i64), Foo);
//                 control.insert(Value::int((b & 0b0011_1111) as i64), Foo);
//                 println!("insert {:?}", b & 0b0011_1111);
//             }
//             64...127 => {
//                 m = m.remove(&Value::int((b & 0b0011_1111) as i64));
//                 control.remove(&Value::int((b & 0b0011_1111) as i64));
//                 println!("remove {:?}", b & 0b0011_1111);
//             }
//             128...191 => {
//                 let key = Value::int((b & 0b0011_1111) as i64);
//                 let (l, k, _) = m.split(&key);
//                 println!("split-l {:?}", b & 0b0011_1111);
//                 println!("splitl: ({:?}, {:?}, _)", l, k);
//
//                 // m = l;
//                 match k {
//                     None => m = l,
//                     Some((k, v)) => m = l.insert(k.clone(), v.clone()),
//                 }
//
//                 let mut new_control = BTreeSet::new();
//                 for (k, v) in control.iter() {
//                     // if k < &key {
//                     //     new_control.insert(k.clone(), v.clone());
//                     // }
//                     if k <= &key {
//                         new_control.insert(k.clone(), v.clone());
//                     }
//                 }
//                 control = new_control;
//             }
//             192...255 => {
//                 let key = Value::int((b & 0b0011_1111) as i64);
//                 let (_, k, r) = m.split(&key);
//                 println!("{:?}", m);
//                 println!("split-r {:?}", b & 0b0011_1111);
//                 println!("splitr: (_, {:?}, {:?})", k, r);
//
//                 // m = r;
//                 match k {
//                     None => m = r,
//                     Some((k, v)) => m = r.insert(k.clone(), v.clone()),
//                 }
//
//                 let mut new_control = BTreeSet::new();
//                 for (k, v) in control.iter() {
//                     if k >= &key {
//                         new_control.insert(k.clone(), v.clone());
//                     }
//                     // if k > &key {
//                     //     new_control.insert(k.clone(), v.clone());
//                     // }
//                 }
//                 control = new_control;
//             }
//         }
//     }
//
//     let mut out = vec![];
//     map_to_vec(&m, &mut out);
//     let out_control: Vec<Value> = control.into_iter().collect();
//
//     if out != out_control {
//         println!("{:?}", "-----");
//         println!("{:?}", out_control);
//         println!("{:?}", out);
//     }
//
//     assert!(out == out_control);
// }

// fn fuzzy_cursor(data: &[u8]) {
//     let mut control = BTreeSet::new();
//     let mut m = Set::new();
//     let half = data.len() / 2;
//
//     for b in &data[..half] {
//         match *b {
//             0...63 => {
//                 m = m.insert(Value::int((b & 0b0011_1111) as i64), Foo);
//                 control.insert(Value::int((b & 0b0011_1111) as i64), Foo);
//             }
//             64...127 => {
//                 m = m.remove(&Value::int((b & 0b0011_1111) as i64));
//                 control.remove(&Value::int((b & 0b0011_1111) as i64));
//             }
//             128...191 => {
//                 let key = Value::int((b & 0b0011_1111) as i64);
//                 let (l, k, _) = m.split(&key);
//
//                 match k {
//                     None => m = l,
//                     Some((k, v)) => m = l.insert(k.clone(), v.clone()),
//                 }
//
//                 let mut new_control = BTreeSet::new();
//                 for (k, v) in control.iter() {
//                     if k <= &key {
//                         new_control.insert(k.clone(), v.clone());
//                     }
//                 }
//                 control = new_control;
//             }
//             192...255 => {
//                 let key = Value::int((b & 0b0011_1111) as i64);
//                 let (_, k, r) = m.split(&key);
//
//                 match k {
//                     None => m = r,
//                     Some((k, v)) => m = r.insert(k.clone(), v.clone()),
//                 }
//
//                 let mut new_control = BTreeSet::new();
//                 for (k, v) in control.iter() {
//                     if k >= &key {
//                         new_control.insert(k.clone(), v.clone());
//                     }
//                 }
//                 control = new_control;
//             }
//         }
//     }
//
//     let out_control: Vec<Value> = control.into_iter().collect();
//     let len = out_control.len();
//     if len == 0 {
//         return;
//     } else {
//         let (mut cursor, mut control_index) = if data[0] % 2 == 0 {
//             (m.cursor_min().unwrap(), 0)
//         } else {
//             (m.cursor_max().unwrap(), len - 1)
//         };
//         let mut skip = false;
//
//         println!("Initial: ({:?}, {:?})\n===", out_control, control_index);
//
//         for b in &data[half..] {
//             println!("control_index: {:?}", control_index);
//             println!("{:?}", cursor);
//             println!("---");
//             if skip {
//                 assert!(control_index == len || control_index == 0)
//             } else {
//                 match cursor.current() {
//                     None => assert!(control_index == len),
//                     Some((k, v)) => assert!((k, v) == out_control[control_index]),
//                 }
//             }
//
//             if b % 2 == 0 {
//                 skip = !cursor.next();
//                 if control_index != len {
//                     control_index += 1;
//                 }
//             } else {
//                 skip = !cursor.prev();
//                 if control_index != 0 {
//                     control_index -= 1;
//                 }
//             }
//         }
//     }
// }
//
// fn fuzzy_bulk(data: &[u8]) {
//     let mut control = BTreeSet::new();
//     let mut control2 = BTreeSet::new();
//     let mut m = Set::new();
//     let mut n = Set::new();
//     let half = data.len() / 2;
//
//     if data.len() == 0 {
//         return;
//     }
//
//     for b in &data[..half] {
//         match *b {
//             0...127 => {
//                 m = m.insert(Value::int((b & 0b0111_1111) as i64), Foo);
//                 control.insert(Value::int((b & 0b0111_1111) as i64), Foo);
//             }
//             128...255 => {
//                 m = m.remove(&Value::int((b & 0b0111_1111) as i64));
//                 control.remove(&Value::int((b & 0b0111_1111) as i64));
//             }
//         }
//     }
//
//     for b in &data[half..] {
//         match *b {
//             0...127 => {
//                 n = n.insert(Value::int((b & 0b0111_1111) as i64), Foo);
//                 control2.insert(Value::int((b & 0b0111_1111) as i64), Foo);
//             }
//             128...255 => {
//                 n = n.remove(&Value::int((b & 0b0111_1111) as i64));
//                 control2.remove(&Value::int((b & 0b0111_1111) as i64));
//             }
//         }
//     }
//
//     let mut out = vec![];
//     let out_control: Vec<Value>;
//
//     match data[0] {
//         _ => {
//             let union_ = m.union(&n);
//             map_to_vec(&union_, &mut out);
//
//             let mut tmp = control2.clone();
//             for (k, v) in control.into_iter() {
//                 tmp.insert(k, v);
//             }
//             out_control = tmp.into_iter().collect();
//         }
//     }
//
//     if out != out_control {
//         println!("{:?}", out_control);
//         println!("{:?}", out);
//     }
//
//     assert!(out == out_control);
// }
//
// #[test]
// fn test_fuzzy_bulk() {
//     fuzzy_bulk(&[0, 0, 0, 1]);
// }

// #[test]
// fn test_fuzzy_cursor() {
//     fuzzy_cursor(&[0x1f,0x0,0x1,0x32,0x0,0x1d,0xff,0xff]);
//     fuzzy(&[0x10,0x1,0x0,0x23]);
//     fuzzy(&[0xca,0x31,0xd1,0x0,0x6b]);
//     fuzzy(&[0x3b,0x1,0x0,0x1,0x10]);
//     fuzzy(&[0x2a,0x2d,0xa,0x1,0x0,0x80]);
//     fuzzy(&[0x1,0xa,0xa]);
// }
