// persistent LLRB tree map, adapted from:
// https://hackage.haskell.org/package/llrbtree-0.0.2/docs/src/Data-RBTree-LL.html

use std::cmp::Ordering::{self, *};

use gc::{Gc, Trace, Finalize, custom_trace};
use gc_derive::{Trace, Finalize};

use crate::value::Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Foo;

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Map(Node, u8 /* height */);

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Node {
    Leaf,
    N2(Gc<(Node, Value, Foo, Node, usize /* count */)>),
    N3(Gc<(Node, Value, Foo, Node, Value, Foo, Node, usize /* count */)>),
}
use self::Node::*;

fn n2(l: Node, k: Value, v: Foo, r: Node) -> Node {
    let c = l.count() + 1 + r.count();
    N2(Gc::new((l, k, v, r, c)))
}

fn n3(l: Node, lk: Value, lv: Foo, m: Node, rk: Value, rv: Foo, r: Node) -> Node {
    let c = l.count() + 1 + m.count() + 1 + r.count();
    N3(Gc::new((l, lk, lv, m, rk, rv, r, c)))
}

impl Map {
    pub fn new() -> Self {
        Map(Leaf, 0)
    }

    pub fn singleton(k: Value, v: Foo) -> Self {
        Map(n2(Leaf, k, v, Leaf), 1)
    }

    pub fn count(&self) -> usize {
        self.0.count()
    }

    pub fn get(&self, kx: &Value) -> Option<&Foo> {
        if self.is_empty() {
            None
        } else {
            self.0.get(kx)
        }
    }

    pub fn contains(&self, kx: &Value) -> bool {
        match self.get(kx) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn insert(&self, kx: Value, vx: Foo) -> Self {
        if self.is_empty() {
            Self::singleton(kx, vx)
        } else {
            match self.0.insert(kx, vx) {
                Insert::Done(done_n) => Map(
                    done_n, self.1
                ),
                Insert::Up(l, k, v, r) => Map(
                    n2(l.clone(), k, v, r.clone()),
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
                Remove::Done(done_n) => Map(
                    done_n, self.1
                ),
                Remove::Up(up_n) => Map(up_n, self.1 - 1),
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

    pub fn split(&self, kx: &Value) -> (Map, Option<(Value, Foo)>, Map) {
        match &self.0 {
            Leaf => (Map::new(), None, Map::new()),
            N2(n) => {
                let (ref l, ref k, ref v, ref r, _) = &(**n);
                match kx.cmp(k) {
                    Less => {
                        let (ll, lm, lr) = Map(l.clone(), self.1 - 1).split(kx);
                        return (
                            ll,
                            lm.clone(),
                            join(&lr.0, lr.1, k.clone(), v.clone(), r, self.1 - 1),
                        );
                    }
                    Equal => (
                        Map(l.clone(), self.1 - 1),
                        Some((k.clone(), v.clone())),
                        Map(r.clone(), self.1 - 1),
                    ),
                    Greater => {
                        let (rl, rm, rr) = Map(r.clone(), self.1 - 1).split(kx);
                        return (
                            join(l, self.1 - 1, k.clone(), v.clone(), &rl.0, rl.1),
                            rm.clone(),
                            rr,
                        );
                    }
                }
            }
            N3(n) => {
                let (ref l, ref lk, ref lv, ref m, ref rk, ref rv, ref r, _) = &(**n);
                match kx.cmp(lk) {
                    Less => {
                        let (ll, lm, lr) = Map(l.clone(), self.1 - 1).split(kx);
                        let tmp = join(&lr.0, lr.1, lk.clone(), lv.clone(), m, self.1 - 1);
                        return (
                            ll,
                            lm.clone(),
                            join(
                                &tmp.0, tmp.1,
                                rk.clone(), rv.clone(),
                                r, self.1 - 1,
                            ),
                        );
                    }
                    Equal => (
                        Map(l.clone(), self.1 - 1),
                        Some((lk.clone(), lv.clone())),
                        Map(n2(m.clone(), rk.clone(), rv.clone(), r.clone()), self.1),
                    ),
                    Greater => match kx.cmp(rk) {
                        Less => {
                            let (ml, mm, mr) = Map(m.clone(), self.1 - 1).split(kx);
                            return (
                                join(l, self.1 - 1, lk.clone(), lv.clone(), &ml.0, ml.1),
                                mm.clone(),
                                join(&mr.0, mr.1, rk.clone(), rv.clone(), r, self.1 - 1),
                            );
                        }
                        Equal => (
                            Map(n2(l.clone(), lk.clone(), lv.clone(), m.clone()), self.1),
                            Some((rk.clone(), rv.clone())),
                            Map(r.clone(), self.1 - 1),
                        ),
                        Greater => {
                            let (rl, rm, rr) = Map(r.clone(), self.1 - 1).split(kx);
                            let tmp = join(m, self.1 - 1, rk.clone(), rv.clone(), &rl.0, rl.1);
                            return (
                                join(
                                    l, self.1 - 1,
                                    lk.clone(), lv.clone(),
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

    pub fn is_submap(&self, other: &Map) -> bool {
        if other.count() > self.count() {
            return false;
        } else if self.is_empty() {
            return true;
        } else {
            for (k, v) in other.iter() {
                match self.get(&k) {
                    None => return false,
                    Some(actual_v) => if v != *actual_v {
                        return false;
                    }
                }
            }
            return true;
        }
    }

    // Insert entry unless there already is an entry of this key.
    pub fn tentative_insert(&self, k: Value, v: Foo) -> Map {
        if self.contains(&k) {
            self.clone()
        } else {
            self.insert(k, v)
        }
    }

    // prefers items in self
    pub fn union(&self, other: &Map) -> Map {
        if self.is_empty() {
            return other.clone();
        } else if other.is_empty() {
            return self.clone();
        } else {
            let other_root = other.root();
            let (lm, x, rm) = self.split(other_root.0);
            let nl = other.left().union(&lm);
            let nr = other.right().union(&rm);
            let nroot = match &x {
                None => other_root,
                Some((k, v)) => (k, v),
            };
            return join(&nl.0, nl.1, nroot.0.clone(), nroot.1.clone(), &nr.0, nr.1);
        }
    }

    pub fn intersection(&self, other: &Map) -> Map {
        if self.is_empty() || other.is_empty() {
            Self::new()
        } else {
            let other_root = other.root();
            let (lm, x, rm) = self.split(other_root.0);
            let nl = other.left().intersection(&lm);
            let nr = other.right().intersection(&rm);
            match &x {
                Some((k, v)) => return join(&nl.0, nl.1, k.clone(), v.clone(), &nr.0, nr.1),
                None => return join2(&nl.0, nl.1, &nr.0, nr.1),
            }
        }
    }

    pub fn difference(&self, other: &Map) -> Map {
        if self.is_empty() || other.is_empty() {
            return self.clone();
        } else {
            let other_root = other.root();
            let (lm, x, rm) = self.split(other_root.0);
            let nl = lm.difference(&other.left());
            let nr = rm.difference(&other.right());
            return join2(&nl.0, nl.1, &nr.0, nr.1);
        }
    }

    pub fn symmetric_difference(&self, other: &Map) -> Map {
        if self.is_empty() {
            return other.clone();
        } else if other.is_empty() {
            return self.clone();
        } else {
            let other_root = other.root();
            let (lm, x, rm) = self.split(other_root.0);
            let nl = lm.symmetric_difference(&other.left());
            let nr = rm.symmetric_difference(&other.right());
            match &x {
                Some((k, v)) => return join2(&nl.0, nl.1, &nr.0, nr.1),
                None => return join(&nl.0, nl.1, other_root.0.clone(), other_root.1.clone(), &nr.0, nr.1),
            }
        }
    }

    fn root(&self) -> (&Value, &Foo) {
        match &self.0 {
            Leaf => unreachable!(),
            N2(n) => (&n.1, &n.2),
            N3(n) => (&n.1, &n.2),
        }
    }

    fn left(&self) -> Self {
        match &self.0 {
            Leaf => unreachable!(),
            N2(n) => Map(n.0.clone(), self.1 - 1),
            N3(n) => Map(n.0.clone(), self.1 - 1),
        }
    }

    fn right(&self) -> Self {
        match &self.0 {
            Leaf => unreachable!(),
            N2(n) => Map(n.3.clone(), self.1 - 1),
            N3(n) => Map(n2(n.3.clone(), n.4.clone(), n.5.clone(), n.6.clone()), self.1),
        }
    }
}

fn join(lesser: &Node, lh: u8, k: Value, v: Foo, greater: &Node, gh: u8) -> Map {
    if lesser.is_empty() {
        return Map(greater.clone(), gh).insert(k, v);
    } else if greater.is_empty() {
        return Map(lesser.clone(), lh).insert(k, v);
    } else {
        match lh.cmp(&gh) {
            Less => match join_lesser_smaller(lesser, k, v, greater, gh - lh) {
                Insert::Done(done_n) => Map(done_n, gh),
                Insert::Up(l, k, v, r) => Map(
                    n2(
                        l.clone(),
                        /**/ k, v,
                        r.clone(),
                    ),
                    gh + 1,
                ),
            }
            Equal => Map(
                n2(
                    lesser.clone(),
                    /**/ k, v,
                    greater.clone(),
                ),
                gh + 1,
            ),
            Greater => match join_greater_smaller(lesser, k, v, greater, lh - gh) {
                Insert::Done(done_n) => Map(done_n, lh),
                Insert::Up(l, k, v, r) => Map(
                    n2(
                        l.clone(),
                        /**/ k, v,
                        r.clone(),
                    ),
                    lh + 1,
                ),
            }
        }
    }
}

fn join2(lesser: &Node, lh: u8, greater: &Node, gh: u8) -> Map {
    if lesser.is_empty() {
        return Map(greater.clone(), gh);
    } else {
        let max = lesser.get_max();
        let nl = Map(lesser.clone(), lh).remove(max.0);
        return join(&nl.0, nl.1, max.0.clone(), max.1.clone(), greater, gh);
    }
}

impl Node {
    fn count(&self) -> usize {
        match self {
            Leaf => 0,
            N2(n) => n.4,
            N3(n) => n.7,
        }
    }

    fn is_empty(&self) -> bool {
        self.count() == 0
    }

    fn get(&self, kx: &Value) -> Option<&Foo> {
        match self {
            Leaf => None,
            N2(n) => {
                let (ref l, ref k, ref v, ref r, _) = &(**n);
                match kx.cmp(k) {
                   Less => l.get(kx),
                   Equal => Some(v),
                   Greater => r.get(kx),
               }
            }
            N3(n) => {
                let (ref l, ref lk, ref lv, ref m, ref rk, ref rv, ref r, _) = &(**n);
                match kx.cmp(lk) {
                    Less => l.get(kx),
                    Equal => Some(lv),
                    Greater => match kx.cmp(rk) {
                        Less => m.get(kx),
                        Equal => Some(rv),
                        Greater => r.get(kx),
                    }
                }
            }
        }
    }

    fn get_min(&self) -> (&Value, &Foo) {
        match self {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref l, ref k, ref v, _, _) = &(**n);
                if l.is_leaf() {
                    (k, v)
                } else {
                    l.get_min()
                }
            }
            N3(n) => {
                let (ref l, ref lk, ref lv, _, _, _, _, _) = &(**n);
                if l.is_leaf() {
                    (lk, lv)
                } else {
                    l.get_min()
                }
            }
        }
    }

    fn get_max(&self) -> (&Value, &Foo) {
        match self {
            Leaf => unreachable!(),
            N2(n) => {
                let (_, ref k, ref v, ref r, _) = &(**n);
                if r.is_leaf() {
                    (k, v)
                } else {
                    r.get_max()
                }
            }
            N3(n) => {
                let (_, _, _, _, ref rk, ref rv, ref r, _) = &(**n);
                if r.is_leaf() {
                    (rk, rv)
                } else {
                    r.get_max()
                }
            }
        }
    }

    fn find_lt(&self, kx: &Value) -> Option<&Foo> {
        unimplemented!();
    }

    fn insert(&self, kx: Value, vx: Foo) -> Insert {
        match self {
            Leaf => Insert::Up(Leaf, kx, vx, Leaf),
            N2(n) => {
                let (ref l, ref k, ref v, ref r, _) = &(**n);
                match kx.cmp(k) {
                    Less => n2_handle_insert_l(l.insert(kx, vx), k, v, r),
                    Equal => Insert::Done(n2(
                        l.clone(),
                        /**/ kx, vx,
                        r.clone(),
                    )),
                    Greater => n2_handle_insert_r(l, k, v, r.insert(kx, vx))
                }
            }
            N3(n) => {
                let (ref l, ref lk, ref lv, ref m, ref rk, ref rv, ref r, _) = &(**n);
                match kx.cmp(lk) {
                    Less => n3_handle_insert_l(l.insert(kx, vx), lk, lv, m, rk, rv, r),
                    Equal => Insert::Done(n3(
                        l.clone(),
                        /**/ kx, vx,
                        m.clone(),
                        /**/ rk.clone(), rv.clone(),
                        r.clone(),
                    )),
                    Greater => match kx.cmp(rk) {
                        Less => n3_handle_insert_m(l, lk, lv, m.insert(kx, vx), rk, rv, r),
                        Equal => Insert::Done(n3(
                            l.clone(),
                            /**/ lk.clone(), lv.clone(),
                            m.clone(),
                            /**/ kx, vx,
                            r.clone(),
                        )),
                        Greater => n3_handle_insert_r(l, lk, lv, m, rk, rv, r.insert(kx, vx)),
                    }
                }
            }
        }
    }

    fn remove(&self, kx: &Value) -> Remove {
        match self {
            Leaf => Remove::Done(Leaf),
            N2(n) => {
                let (ref l, ref k, ref v, ref r, _) = &(**n);
                match kx.cmp(k) {
                    Less => n2_handle_remove_l(l.remove(kx), k, v, r),
                    Equal => if r.is_leaf() {
                        Remove::Up(Leaf)
                    } else {
                        let (new_k, new_v) = l.get_max();
                        n2_handle_remove_l(l.remove_max(), new_k, new_v, r)
                    }
                    Greater => n2_handle_remove_r(l, k, v, r.remove(kx)),
                }
            }
            N3(n) => {
                let (ref l, ref lk, ref lv, ref m, ref rk, ref rv, ref r, _) = &(**n);
                match kx.cmp(lk) {
                    Less => n3_handle_remove_l(l.remove(kx), lk, lv, m, rk, rv, r),
                    Equal => if m.is_leaf() {
                        Remove::Done(n2(Leaf, rk.clone(), rv.clone(), Leaf))
                    } else {
                        let (new_k, new_v) = l.get_max();
                        n3_handle_remove_l(l.remove_max(), new_k, new_v, m, rk, rv, r)
                    }
                    Greater => match kx.cmp(rk) {
                        Less => n3_handle_remove_m(l, lk, lv, m.remove(kx), rk, rv, r),
                        Equal => if r.is_leaf() {
                            Remove::Done(n2(Leaf, lk.clone(), lv.clone(), Leaf))
                        } else {
                            let (new_k, new_v) = m.get_max();
                            n3_handle_remove_m(l, lk, lv, m.remove_max(), new_k, new_v, r)
                        }
                        Greater => n3_handle_remove_r(l, lk, lv, m, rk, rv, r.remove(kx)),
                    }
                }
            }
        }
    }

    fn remove_min(&self) -> Remove {
        let (min_k, _) = self.get_min();
        return self.remove(min_k);
    }

    fn remove_max(&self) -> Remove {
        let (max_k, _) = self.get_max();
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
                if n.3.is_leaf() {
                    out.push(N2KV(n.clone()));
                } else {
                    out.push(N2Right(n.clone()));
                    n.3.rightmost_positions(out);
                }
            }
            N3(n) => {
                if n.6.is_leaf() {
                    out.push(N3RKV(n.clone()));
                } else {
                    out.push(N3Right(n.clone()));
                    n.6.rightmost_positions(out);
                }
            }
        }
    }
}

fn n2_handle_insert_l(insert_l: Insert, k: &Value, v: &Foo, r: &Node) -> Insert {
    match insert_l {
        Insert::Done(done_n) => Insert::Done(n2(
            done_n,
            /**/ k.clone(), v.clone(),
            r.clone(),
        )),
        Insert::Up(up_l, up_k, up_v, up_r) => Insert::Done(n3(
            up_l,
            /**/ up_k, up_v,
            up_r,
            /**/ k.clone(), v.clone(),
            r.clone(),
        )),
    }
}

fn n2_handle_insert_r(l: &Node, k: &Value, v: &Foo, insert_r: Insert) -> Insert {
    match insert_r {
        Insert::Done(done_n) => Insert::Done(n2(
            l.clone(),
            /**/ k.clone(), v.clone(),
            done_n,
        )),
        Insert::Up(up_l, up_k, up_v, up_r) => Insert::Done(n3(
            l.clone(),
            /**/ k.clone(), v.clone(),
            up_l,
            /**/ up_k, up_v,
            up_r,
        )),
    }
}

fn n3_handle_insert_l(
    insert_l: Insert, lk: &Value, lv: &Foo, m: &Node, rk: &Value, rv: &Foo, r: &Node
) -> Insert {
    match insert_l {
        Insert::Done(done_n) => Insert::Done(n3(
            done_n,
            /**/ lk.clone(), lv.clone(),
            m.clone(),
            /**/ rk.clone(), rv.clone(),
            r.clone(),
        )),
        Insert::Up(up_l, up_k, up_v, up_r) => Insert::Up(
                n2(up_l, up_k, up_v, up_r),
                /**/ lk.clone(), lv.clone(),
                n2(m.clone(), rk.clone(), rv.clone(), r.clone()),
            ),
    }
}

fn n3_handle_insert_m(
    l: &Node, lk: &Value, lv: &Foo, insert_m: Insert, rk: &Value, rv: &Foo, r: &Node
) -> Insert {
    match insert_m {
        Insert::Done(done_n) => Insert::Done(n3(
            l.clone(),
            /**/ lk.clone(), lv.clone(),
            done_n,
            /**/ rk.clone(), rv.clone(),
            r.clone(),
        )),
        Insert::Up(up_l, up_k, up_v, up_r) => Insert::Up(
            n2(l.clone(), lk.clone(), lv.clone(), up_l),
            /**/ up_k.clone(), up_v.clone(),
            n2(up_r.clone(), rk.clone(), rv.clone(), r.clone()),
        ),
    }
}

fn n3_handle_insert_r(
    l: &Node, lk: &Value, lv: &Foo, m: &Node, rk: &Value, rv: &Foo, insert_r: Insert
) -> Insert {
    match insert_r {
        Insert::Done(done_n) => Insert::Done(n3(
            l.clone(),
            /**/ lk.clone(), lv.clone(),
            m.clone(),
            /**/ rk.clone(), rv.clone(),
            done_n,
        )),
        Insert::Up(up_l, up_k, up_v, up_r) => Insert::Up(
            n2(l.clone(), lk.clone(), lv.clone(), m.clone()),
            /**/ rk.clone(), rv.clone(),
            n2(up_l, up_k, up_v, up_r),
        ),
    }
}

fn n2_handle_remove_l(remove_l: Remove, k: &Value, v: &Foo, r: &Node) -> Remove {
    match remove_l {
        Remove::Done(done_n) => Remove::Done(n2(
            done_n,
            /**/ k.clone(), v.clone(),
            r.clone(),
        )),
        Remove::Up(up_n) => match r {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref rl, ref rk, ref rv, ref rr, _) = &(**n);
                Remove::Up(n3(
                    up_n,
                    /**/ k.clone(), v.clone(),
                    rl.clone(),
                    /**/ rk.clone(), rv.clone(),
                    rr.clone(),
                ))
            }
            N3(n) => {
                let (ref rl, ref rlk, ref rlv, ref rm, ref rrk, ref rrv, ref rr, _) = &(**n);
                Remove::Done(n2(
                    n2(
                        up_n,
                        /**/ k.clone(), v.clone(),
                        rl.clone(),
                    ),
                    /**/ rlk.clone(), rlv.clone(),
                    n2(
                        rm.clone(),
                        /**/ rrk.clone(), rrv.clone(),
                        rr.clone(),
                    ),
                ))
            }
        }
    }
}

fn n2_handle_remove_r(l: &Node, k: &Value, v: &Foo, remove_r: Remove) -> Remove {
    match remove_r {
        Remove::Done(done_n) => Remove::Done(n2(
            l.clone(),
            /**/ k.clone(), v.clone(),
            done_n,
        )),
        Remove::Up(up_n) => match l {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref ll, ref lk, ref lv, ref lr, _) = &(**n);
                Remove::Up(n3(
                    ll.clone(),
                    /**/ lk.clone(), lv.clone(),
                    lr.clone(),
                    /**/ k.clone(), v.clone(),
                    up_n,
                ))
            }
            N3(n) => {
                let (ref ll, ref llk, ref llv, ref lm, ref lrk, ref lrv, ref lr, _) = &(**n);
                Remove::Done(n2(
                    n2(
                        ll.clone(),
                        /**/ llk.clone(), llv.clone(),
                        lm.clone(),
                    ),
                    /**/ lrk.clone(), lrv.clone(),
                    n2(
                        lr.clone(),
                        /**/ k.clone(), v.clone(),
                        up_n,
                    ),
                ))
            }
        }
    }
}

fn n3_handle_remove_l(
    remove_l: Remove, lk: &Value, lv: &Foo, m: &Node, rk: &Value, rv: &Foo, r: &Node
) -> Remove {
    match remove_l {
        Remove::Done(done_n) => Remove::Done(n3(
            done_n,
            /**/ lk.clone(), lv.clone(),
            m.clone(),
            /**/ rk.clone(), rv.clone(),
            r.clone(),
        )),
        Remove::Up(up_n) => match m {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref ml, ref mk, ref mv, ref mr, _) = &(**n);
                Remove::Done(n2(
                    n3(
                        up_n,
                        /**/ lk.clone(), lv.clone(),
                        ml.clone(),
                        /**/ mk.clone(), mv.clone(),
                        mr.clone(),
                    ),
                    /**/ rk.clone(), rv.clone(),
                    r.clone()
                ))
            }
            N3(n) => {
                let (ref ml, ref mlk, ref mlv, ref mm, ref mrk, ref mrv, ref mr, _) = &(**n);
                Remove::Done(n3(
                    n2(
                        up_n,
                        /**/ lk.clone(), lv.clone(),
                        ml.clone(),
                    ),
                    /**/ mlk.clone(), mlv.clone(),
                    n2(mm.clone(),
                    /**/ mrk.clone(), mrv.clone(),
                    mr.clone(),),
                    /**/ rk.clone(), rv.clone(),
                    r.clone(),
                ))
            }
        }
    }
}

fn n3_handle_remove_m(
    l: &Node, lk: &Value, lv: &Foo, remove_m: Remove, rk: &Value, rv: &Foo, r: &Node
) -> Remove {
    match remove_m {
        Remove::Done(done_n) => Remove::Done(n3(
            l.clone(),
            /**/ lk.clone(), lv.clone(),
            done_n,
            /**/ rk.clone(), rv.clone(),
            r.clone(),
        )),
        Remove::Up(up_n) => match r {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref rl, ref rk_, ref rv_, ref rr, _) = &(**n);
                Remove::Done(n2(
                    l.clone(),
                    /**/ lk.clone(), lv.clone(),
                    n3(
                        up_n,
                        /**/ rk.clone(), rv.clone(),
                        rl.clone(),
                        /**/ rk_.clone(), rv_.clone(),
                        rr.clone(),
                    ),
                ))
            }
            N3(n) => {
                let (ref rl, ref rlk, ref rlv, ref rm, ref rrk, ref rrv, ref rr, _) = &(**n);
                Remove::Done(n3(
                    l.clone(),
                    /**/ lk.clone(), lv.clone(),
                    n2(
                        up_n,
                        /**/ rk.clone(), rv.clone(),
                        rl.clone(),
                    ),
                    /**/ rlk.clone(), rlv.clone(),
                    n2(
                        rm.clone(),
                        /**/ rrk.clone(), rrv.clone(),
                        rr.clone(),
                    ),
                ))
            }
        }
    }
}

fn n3_handle_remove_r(
    l: &Node, lk: &Value, lv: &Foo, m: &Node, rk: &Value, rv: &Foo, remove_r: Remove
) -> Remove {
    match remove_r {
        Remove::Done(done_n) => Remove::Done(n3(
            l.clone(),
            /**/ lk.clone(), lv.clone(),
            m.clone(),
            /**/ rk.clone(), rv.clone(),
            done_n,
        )),
        Remove::Up(up_n) => match m {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref ml, ref mk, ref mv, ref mr, _) = &(**n);
                Remove::Done(n2(
                    l.clone(),
                    /**/ lk.clone(), lv.clone(),
                    n3(
                        ml.clone(),
                        /**/ mk.clone(), mv.clone(),
                        mr.clone(),
                        /**/ rk.clone(), rv.clone(),
                        up_n,
                    ),
                ))
            }
            N3(n) => {
                let (ref ml, ref mlk, ref mlv, ref mm, ref mrk, ref mrv, ref mr, _) = &(**n);
                Remove::Done(n3(
                    l.clone(),
                    /**/ lk.clone(), lv.clone(),
                    n2(
                        ml.clone(),
                        /**/ mlk.clone(), mlv.clone(),
                        mm.clone(),
                    ),
                    /**/ mrk.clone(), mrv.clone(),
                    n2(
                        mr.clone(),
                        /**/ rk.clone(), rv.clone(),
                        up_n,
                    ),
                ))
            }
        }
    }
}

// traverse left spine of greater node for h_diff, then merge
fn join_lesser_smaller(lesser: &Node, k: Value, v: Foo, greater: &Node, h_diff: u8) -> Insert {
    if h_diff == 0 {
        Insert::Up(lesser.clone(), k, v, greater.clone())
    } else {
        match greater {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref gl, ref gk, ref gv, ref gr, _) = &(**n);
                n2_handle_insert_l(
                    join_lesser_smaller(lesser, k, v, gl, h_diff - 1), gk, gv, gr
                )
            }
            N3(n) => {
                let (ref gl, ref glk, ref glv, ref gm, ref grk, ref grv, ref gr, _) = &(**n);
                n3_handle_insert_l(
                    join_lesser_smaller(
                        lesser, k, v, gl, h_diff - 1
                    ),
                    glk, glv, gm, grk, grv, gr,
                )
            }
        }
    }
}

fn join_greater_smaller(lesser: &Node, k: Value, v: Foo, greater: &Node, h_diff: u8) -> Insert {
    if h_diff == 0 {
        Insert::Up(lesser.clone(), k, v, greater.clone())
    } else {
        match lesser {
            Leaf => unreachable!(),
            N2(n) => {
                let (ref ll, ref lk, ref lv, ref lr, _) = &(**n);
                n2_handle_insert_r(
                    ll, lk, lv, join_greater_smaller(lr, k, v, greater, h_diff - 1)
                )
            }
            N3(n) => {
                let (ref ll, ref llk, ref llv, ref lm, lrk, lrv, lr, _) = &(**n);
                n3_handle_insert_r(
                    ll, llk, llv, lm, lrk, lrv,
                    join_greater_smaller(
                        lr, k, v, greater, h_diff - 1
                    ),
                )
            }
        }
    }
}

enum Insert {
    Done(Node),
    Up(Node, Value, Foo, Node),
}

enum Remove {
    Done(Node),
    Up(Node),
}

#[derive(Debug, Clone)]
pub struct Cursor(Vec<Position>);

#[derive(Debug, Clone)]
enum Position {
    N2Left(Gc<(Node, Value, Foo, Node, usize /* count */)>),
    N2KV(Gc<(Node, Value, Foo, Node, usize /* count */)>),
    N2Right(Gc<(Node, Value, Foo, Node, usize /* count */)>),
    N2Post(Gc<(Node, Value, Foo, Node, usize /* count */)>),
    N3Left(Gc<(Node, Value, Foo, Node, Value, Foo, Node, usize /* count */)>),
    N3LKV(Gc<(Node, Value, Foo, Node, Value, Foo, Node, usize /* count */)>),
    N3Middle(Gc<(Node, Value, Foo, Node, Value, Foo, Node, usize /* count */)>),
    N3RKV(Gc<(Node, Value, Foo, Node, Value, Foo, Node, usize /* count */)>),
    N3Right(Gc<(Node, Value, Foo, Node, Value, Foo, Node, usize /* count */)>),
    N3Post(Gc<(Node, Value, Foo, Node, Value, Foo, Node, usize /* count */)>),
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
                if n.3.is_leaf() {
                    positions[len - 1] = N2Right(n);
                    return true;
                } else {
                    positions[len - 1] = N2Right(n.clone());
                    n.3.leftmost_positions(positions);
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
                    n.3.leftmost_positions(positions);
                    return true;
                } else {
                    positions[len - 1] = N3LKV(n);
                    return true;
                }
            }
            N3LKV(n) => {
                if n.3.is_leaf() {
                    positions[len - 1] = N3RKV(n);
                    return true;
                } else {
                    positions[len - 1] = N3Middle(n.clone());
                    n.3.leftmost_positions(positions);
                    return true;
                }
            },
            N3Middle(n) => {
                if n.3.is_leaf() {
                    positions[len - 1] = N3RKV(n);
                    return Position::step_next(positions);
                } else {
                    positions[len - 1] = N3RKV(n);
                    return true;
                }
            }
            N3RKV(n) => {
                if n.6.is_leaf() {
                    positions[len - 1] = N3Right(n);
                    return true;
                } else {
                    positions[len - 1] = N3Right(n.clone());
                    n.6.leftmost_positions(positions);
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
                if n.3.is_leaf() {
                    positions[len - 1] = N2KV(n);
                    return Position::step_prev(positions);
                } else {
                    positions[len - 1] = N2KV(n);
                    return true;
                }
            }
            N2Post(n) => {
                if n.3.is_leaf() {
                    positions[len - 1] = N2KV(n);
                    return true;
                } else {
                    positions[len - 1] = N2Right(n.clone());
                    n.3.rightmost_positions(positions);
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
                if n.3.is_leaf() {
                    positions[len - 1] = N3LKV(n);
                    return Position::step_prev(positions);
                } else {
                    positions[len - 1] = N3LKV(n);
                    return true;
                }
            }
            N3RKV(n) => {
                if n.3.is_leaf() {
                    positions[len - 1] = N3LKV(n);
                    return true;
                } else {
                    positions[len - 1] = N3Middle(n.clone());
                    n.3.rightmost_positions(positions);
                    return true;
                }
            },
            N3Right(n) => {
                if n.6.is_leaf() {
                    positions[len - 1] = N3RKV(n);
                    return Position::step_prev(positions);
                } else {
                    positions[len - 1] = N3RKV(n);
                    return true;
                }
            }
            N3Post(n) => {
                if n.6.is_leaf() {
                    positions[len - 1] = N3RKV(n);
                    return true;
                } else {
                    positions[len - 1] = N3Right(n.clone());
                    n.6.rightmost_positions(positions);
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

    pub fn current(&mut self) -> Option<(Value, Foo)> {
        let len = self.0.len();
        match &self.0[len - 1] {
            N2Left(n) => {
                self.0[len - 1] = N2KV(n.clone());
                return self.current();
            }
            N2KV(n) => return Some((n.1.clone(), n.2.clone())),
            N2Right(n) => {
                self.0[len - 1] = N2Post(n.clone());
                return self.current();
            }
            N3Left(n) => {
                self.0[len - 1] = N3LKV(n.clone());
                return self.current();
            }
            N3LKV(n) => return Some((n.1.clone(), n.2.clone())),
            N3Middle(n) => {
                self.0[len - 1] = N3RKV(n.clone());
                return self.current();
            }
            N3RKV(n) => return Some((n.4.clone(), n.5.clone())),
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

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        match (self.is_empty(), other.is_empty()) {
            (true, true) => true,
            (true, false) | (false, true) => false,
            (false, false) => self.0.eq(&other.0),
        }
    }
}
impl Eq for Map {}

impl PartialOrd for Map {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Map {
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
                (Some((ka, va)), Some((kb, vb))) => {
                    if ka == kb && va == vb {
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
                (Some((ka, va)), Some((kb, vb))) => {
                    let compare_keys = ka.cmp(&kb);
                    match compare_keys {
                        Equal => {
                            let compare_vals = va.cmp(&vb);
                            match compare_vals {
                                Equal => {
                                    ca.next();
                                    cb.next();
                                }
                                _ => return compare_vals,
                            }
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
    type Item = (Value, Foo);

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

pub fn map_to_vec(m: &Map, out: &mut Vec<(Value, Foo)>) {
    node_to_vec(&m.0, out)
}

fn node_to_vec(n: &Node, out: &mut Vec<(Value, Foo)>) {
    match n {
        Leaf => {},
        N2(n) => {
            let (ref l, ref k, ref v, ref r, _) = &(**n);
            node_to_vec(l, out);
            out.push((k.clone(), v.clone()));
            node_to_vec(r, out);
        }
        N3(n) => {
            let (ref l, ref lk, ref lv, ref m, ref rk, ref rv, ref r, _) = &(**n);
            node_to_vec(l, out);
            out.push((lk.clone(), lv.clone()));
            node_to_vec(m, out);
            out.push((rk.clone(), rv.clone()));
            node_to_vec(r, out);
        }
    }
}


use std::collections::BTreeMap;

// fn fuzzy(data: &[u8]) {
//     // Foo
//     let mut control = BTreeMap::new();
//     let mut m = Map::new();
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
//                 let mut new_control = BTreeMap::new();
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
//                 let mut new_control = BTreeMap::new();
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
//     let out_control: Vec<(Value, Foo)> = control.into_iter().collect();
//
//     if out != out_control {
//         println!("{:?}", "-----");
//         println!("{:?}", out_control);
//         println!("{:?}", out);
//     }
//
//     assert!(out == out_control);
// }

fn fuzzy_cursor(data: &[u8]) {
    let mut control = BTreeMap::new();
    let mut m = Map::new();
    let half = data.len() / 2;

    for b in &data[..half] {
        match *b {
            0...63 => {
                m = m.insert(Value::int((b & 0b0011_1111) as i64), Foo);
                control.insert(Value::int((b & 0b0011_1111) as i64), Foo);
            }
            64...127 => {
                m = m.remove(&Value::int((b & 0b0011_1111) as i64));
                control.remove(&Value::int((b & 0b0011_1111) as i64));
            }
            128...191 => {
                let key = Value::int((b & 0b0011_1111) as i64);
                let (l, k, _) = m.split(&key);

                match k {
                    None => m = l,
                    Some((k, v)) => m = l.insert(k.clone(), v.clone()),
                }

                let mut new_control = BTreeMap::new();
                for (k, v) in control.iter() {
                    if k <= &key {
                        new_control.insert(k.clone(), v.clone());
                    }
                }
                control = new_control;
            }
            192...255 => {
                let key = Value::int((b & 0b0011_1111) as i64);
                let (_, k, r) = m.split(&key);

                match k {
                    None => m = r,
                    Some((k, v)) => m = r.insert(k.clone(), v.clone()),
                }

                let mut new_control = BTreeMap::new();
                for (k, v) in control.iter() {
                    if k >= &key {
                        new_control.insert(k.clone(), v.clone());
                    }
                }
                control = new_control;
            }
        }
    }

    let out_control: Vec<(Value, Foo)> = control.into_iter().collect();
    let len = out_control.len();
    if len == 0 {
        return;
    } else {
        let (mut cursor, mut control_index) = if data[0] % 2 == 0 {
            (m.cursor_min().unwrap(), 0)
        } else {
            (m.cursor_max().unwrap(), len - 1)
        };
        let mut skip = false;

        println!("Initial: ({:?}, {:?})\n===", out_control, control_index);

        for b in &data[half..] {
            println!("control_index: {:?}", control_index);
            println!("{:?}", cursor);
            println!("---");
            if skip {
                assert!(control_index == len || control_index == 0)
            } else {
                match cursor.current() {
                    None => assert!(control_index == len),
                    Some((k, v)) => assert!((k, v) == out_control[control_index]),
                }
            }

            if b % 2 == 0 {
                skip = !cursor.next();
                if control_index != len {
                    control_index += 1;
                }
            } else {
                skip = !cursor.prev();
                if control_index != 0 {
                    control_index -= 1;
                }
            }
        }
    }
}

fn fuzzy_bulk(data: &[u8]) {
    let mut control = BTreeMap::new();
    let mut control2 = BTreeMap::new();
    let mut m = Map::new();
    let mut n = Map::new();
    let half = data.len() / 2;

    if data.len() == 0 {
        return;
    }

    for b in &data[..half] {
        match *b {
            0...127 => {
                m = m.insert(Value::int((b & 0b0111_1111) as i64), Foo);
                control.insert(Value::int((b & 0b0111_1111) as i64), Foo);
            }
            128...255 => {
                m = m.remove(&Value::int((b & 0b0111_1111) as i64));
                control.remove(&Value::int((b & 0b0111_1111) as i64));
            }
        }
    }

    for b in &data[half..] {
        match *b {
            0...127 => {
                n = n.insert(Value::int((b & 0b0111_1111) as i64), Foo);
                control2.insert(Value::int((b & 0b0111_1111) as i64), Foo);
            }
            128...255 => {
                n = n.remove(&Value::int((b & 0b0111_1111) as i64));
                control2.remove(&Value::int((b & 0b0111_1111) as i64));
            }
        }
    }

    let mut out = vec![];
    let out_control: Vec<(Value, Foo)>;

    match data[0] {
        _ => {
            let union_ = m.union(&n);
            map_to_vec(&union_, &mut out);

            let mut tmp = control2.clone();
            for (k, v) in control.into_iter() {
                tmp.insert(k, v);
            }
            out_control = tmp.into_iter().collect();
        }
    }

    if out != out_control {
        println!("{:?}", out_control);
        println!("{:?}", out);
    }

    assert!(out == out_control);
}

#[test]
fn test_fuzzy_bulk() {
    fuzzy_bulk(&[0, 0, 0, 1]);
}

// #[test]
// fn test_fuzzy_cursor() {
//     fuzzy_cursor(&[0x1f,0x0,0x1,0x32,0x0,0x1d,0xff,0xff]);
//     fuzzy(&[0x10,0x1,0x0,0x23]);
//     fuzzy(&[0xca,0x31,0xd1,0x0,0x6b]);
//     fuzzy(&[0x3b,0x1,0x0,0x1,0x10]);
//     fuzzy(&[0x2a,0x2d,0xa,0x1,0x0,0x80]);
//     fuzzy(&[0x1,0xa,0xa]);
// }
