// balanced rope, using 2-3 trees for balancing
// *very* naive (leafs are single items)

use std::cmp::Ordering::{self, *};

use gc::{Gc, Trace, Finalize, custom_trace};
use gc_derive::{Trace, Finalize};

use crate::value::Value;

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Arr {
    Empty,
    NonEmpty(Node, u8 /* height */),
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Node {
    Leaf(Value),
    N2(Gc<(Node, usize /*left count*/, Node, usize /* count */)>),
    N3(Gc<(Node, usize /*left count*/, Node, usize /*left + middle count*/, Node, usize /* count */)>),
}
use self::Node::*;

fn n2(l: Node, r: Node) -> Node {
    let lc = l.count();
    let c = lc + r.count();
    N2(Gc::new((l, lc, r, c)))
}

fn n3(l: Node, m: Node, r: Node) -> Node {
    let lc = l.count();
    let mc = m.count();
    let c = lc + mc + r.count();
    N3(Gc::new((l, lc, m, lc + mc, r, c)))
}

impl Arr {
    pub fn new() -> Self {
        Arr::Empty
    }

    pub fn singleton(k: Value) -> Self {
        Arr::NonEmpty(Leaf(k), 1)
    }

    pub fn count(&self) -> usize {
        match self {
            Arr::Empty => 0,
            Arr::NonEmpty(n, _) => n.count(),
        }
    }

    pub fn insert(&self, at: usize, kx: Value) -> Self {
        match self {
            Arr::Empty => Self::singleton(kx),
            Arr::NonEmpty(n, h) => match n.insert(at, kx) {
                Insert::Done(done_n) => Arr::NonEmpty(done_n, *h),
                Insert::Up(l, r) => Arr::NonEmpty(
                    n2(l.clone(), r.clone()),
                    h + 1,
                ),
            }
        }
    }

    pub fn remove(&self, at: usize) -> Self {
        match self {
            Arr::Empty => Self::new(),
            Arr::NonEmpty(n, h) => match n.remove(at) {
                Remove::Empty => Self::new(),
                Remove::Done(done_n) => Arr::NonEmpty(
                    done_n, *h
                ),
                Remove::Up(up_n) => Arr::NonEmpty(up_n, h - 1),
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub fn concat(&self, other: &Arr) -> Arr {
        match (self, other) {
            (Arr::Empty, _) => other.clone(),
            (_, Arr::Empty) => self.clone(),
            (Arr::NonEmpty(self_n, self_h), Arr::NonEmpty(other_n, other_h)) => {
                join(self_n, *self_h, other_n, *other_h)
            }
        }
    }

    pub fn split(&self, at: usize) -> (Arr, Arr) {
        match self {
            Arr::Empty => (Self::new(), Self::new()),
            Arr::NonEmpty(n, h) => match n {
                Leaf(v) => match at {
                    0 => (Self::new(), self.clone()),
                    1 => (self.clone(), Self::new()),
                    _ => panic!("insert: index out of bounds"),
                }
                N2(n) => {
                    let (ref l, ref lc, ref r, _) = &(**n);
                    if at < *lc {
                        let (ll, lr) = Arr::NonEmpty(l.clone(), h - 1).split(at);
                        return (ll, lr.concat(&Arr::NonEmpty(r.clone(), h - 1)));
                    } else {
                        let (rl, rr) = Arr::NonEmpty(r.clone(), h - 1).split(at - lc);
                        return (Arr::NonEmpty(l.clone(), h - 1).concat(&rl), rr);
                    }
                }
                N3(n) => {
                    let (ref l, ref lc, ref m, ref lmc, ref r, _) = &(**n);
                    if at < *lc {
                        let (ll, lr) = Arr::NonEmpty(l.clone(), h - 1).split(at);
                        return (
                            ll,
                            lr.concat(&Arr::NonEmpty(m.clone(), h - 1))
                              .concat(&Arr::NonEmpty(r.clone(), h - 1)),
                        );
                    } else if at < *lmc {
                        let (ml, mr) = Arr::NonEmpty(m.clone(), h - 1).split(at - *lc);
                        return (
                            Arr::NonEmpty(l.clone(), h - 1).concat(&ml),
                            mr.concat(&Arr::NonEmpty(r.clone(), h - 1)),
                        );
                    } else {
                        let (rl, rr) = Arr::NonEmpty(r.clone(), h - 1).split(at - *lmc);
                        return (
                            Arr::NonEmpty(l.clone(), h - 1)
                              .concat(&Arr::NonEmpty(m.clone(), h - 1))
                              .concat(&rl),
                            rr,
                        );
                    }
                }
            }
        }
    }

    pub fn cursor_at(&self, at: usize) -> ActualCursor {
        match self {
            Arr::Empty => ActualCursor::Empty,
            Arr::NonEmpty(n, _) => match n {
                Leaf(v) => ActualCursor::Singleton(v.clone()),
                _ => ActualCursor::Cursor(Cursor::new(n, at)),
            }
        }
    }

    pub fn cursor_start(&self) -> ActualCursor {
        match self {
            Arr::Empty => ActualCursor::Empty,
            Arr::NonEmpty(n, _) => match n {
                Leaf(v) => ActualCursor::Singleton(v.clone()),
                _ => ActualCursor::Cursor(Cursor::new_start(n)),
            }
        }
    }

    pub fn cursor_end(&self) -> ActualCursor {
        match self {
            Arr::Empty => ActualCursor::Empty,
            Arr::NonEmpty(n, _) => match n {
                Leaf(v) => ActualCursor::Singleton(v.clone()),
                _ => ActualCursor::Cursor(Cursor::new_end(n)),
            }
        }
    }
}

pub enum ActualCursor {
    Empty,
    Singleton(Value),
    Cursor(Cursor),
}

impl Node {
    fn count(&self) -> usize {
        match self {
            Leaf(_) => 1,
            N2(n) => n.3,
            N3(n) => n.5,
        }
    }

    fn is_empty(&self) -> bool {
        self.count() == 0
    }

    fn get_start(&self) -> &Value {
        match self {
            Leaf(yay) => yay,
            N2(n) => {
                let (ref l, _, _, _) = &(**n);
                l.get_start()
            }
            N3(n) => {
                let (ref l, _, _, _, _, _) = &(**n);
                l.get_start()
            }
        }
    }

    fn get_end(&self) -> &Value {
        match self {
            Leaf(yay) => yay,
            N2(n) => {
                let (_, _, ref r, _) = &(**n);
                r.get_end()
            }
            N3(n) => {
                let (_, _,  _, _, ref r, _) = &(**n);
                r.get_end()
            }
        }
    }

    fn insert(&self, at: usize, kx: Value) -> Insert {
        match self {
            Leaf(v) => match at {
                0 => Insert::Up(Leaf(kx.clone()), Leaf(v.clone())),
                1 => Insert::Up(Leaf(v.clone()), Leaf(kx.clone())),
                _ => panic!("insert: index out of bounds"),
            }
            N2(n) => {
                let (ref l, ref lc, ref r, _) = &(**n);
                if at < *lc {
                    n2_handle_insert_l(l.insert(at, kx), r)
                } else {
                    n2_handle_insert_r(l, r.insert(at - lc, kx))
                }
            }
            N3(n) => {
                let (ref l, ref lc, ref m, ref lmc, ref r, _) = &(**n);
                if at < *lc {
                    n3_handle_insert_l(l.insert(at, kx), m, r)
                } else if at < *lmc {
                    n3_handle_insert_m(l, m.insert(at - lc, kx), r)
                } else {
                    n3_handle_insert_r(l, m, r.insert(at - lmc, kx))
                }
            }
        }
    }

    fn remove(&self, at: usize) -> Remove {
        match self {
            Leaf(_) => Remove::Empty,
            N2(n) => {
                let (ref l, ref lc, ref r, _) = &(**n);
                if at < *lc {
                    n2_handle_remove_l(l.remove(at), r)
                } else {
                    n2_handle_remove_r(l, r.remove(at - lc))
                }
            }
            N3(n) => {
                let (ref l, ref lc, ref m, ref lmc, ref r, _) = &(**n);
                if at < *lc {
                    n3_handle_remove_l(l.remove(at), m, r)
                } else if at < *lmc {
                    n3_handle_remove_m(l, m.remove(at - lc), r)
                } else {
                    n3_handle_remove_r(l, m, r.remove(at - lmc))
                }
            }
        }
    }

    fn remove_start(&self) -> Remove {
        return self.remove(0);
    }

    fn remove_end(&self) -> Remove {
        let end = self.count() - 1;
        return self.remove(end);
    }

    fn is_leaf(&self) -> bool {
        match self {
            Leaf(_) => true,
            _ => false,
        }
    }

    // appends the leftmost path from self (inclusive) to a leaf (exclusive) of positions
    fn positions_start(&self, out: &mut Vec<Position>) {
        match self {
            Leaf(_) => {}
            N2(n) => {
                out.push(N2Left(n.clone()));
                n.0.positions_start(out);
            }
            N3(n) => {
                out.push(N3Left(n.clone()));
                n.0.positions_start(out);
            }
        }
    }

    // appends the rightmost path from self (inclusive) to a leaf (exclusive) of positions
    // (places the cursor *before* the rightmost element)
    fn positions_end(&self, out: &mut Vec<Position>) {
        match self {
            Leaf(_) => {}
            N2(n) => {
                out.push(N2Right(n.clone()));
                n.2.positions_end(out);
            }
            N3(n) => {
                out.push(N3Right(n.clone()));
                n.4.positions_end(out);
            }
        }
    }
}

fn join(lesser: &Node, lh: u8, greater: &Node, gh: u8) -> Arr {
    if lesser.is_empty() {
        return Arr::NonEmpty(greater.clone(), gh);
    } else if greater.is_empty() {
        return Arr::NonEmpty(lesser.clone(), lh);
    } else {
        match lh.cmp(&gh) {
            Less => match join_lesser_smaller(lesser, greater, gh - lh) {
                Insert::Done(done_n) => Arr::NonEmpty(done_n, gh),
                Insert::Up(l, r) => Arr::NonEmpty(
                    n2(
                        l.clone(),
                        r.clone(),
                    ),
                    gh + 1,
                ),
            }
            Equal => Arr::NonEmpty(
                n2(
                    lesser.clone(),
                    greater.clone(),
                ),
                gh + 1,
            ),
            Greater => match join_greater_smaller(lesser, greater, lh - gh) {
                Insert::Done(done_n) => Arr::NonEmpty(done_n, lh),
                Insert::Up(l, r) => Arr::NonEmpty(
                    n2(
                        l.clone(),
                        r.clone(),
                    ),
                    lh + 1,
                ),
            }
        }
    }
}

fn n2_handle_insert_l(insert_l: Insert, r: &Node) -> Insert {
    match insert_l {
        Insert::Done(done_n) => Insert::Done(n2(
            done_n,
            r.clone(),
        )),
        Insert::Up(up_l, up_r) => Insert::Done(n3(
            up_l,
            up_r,
            r.clone(),
        )),
    }
}

fn n2_handle_insert_r(l: &Node, insert_r: Insert) -> Insert {
    match insert_r {
        Insert::Done(done_n) => Insert::Done(n2(
            l.clone(),
            done_n,
        )),
        Insert::Up(up_l, up_r) => Insert::Done(n3(
            l.clone(),
            up_l,
            up_r,
        )),
    }
}

fn n3_handle_insert_l(
    insert_l: Insert, m: &Node, r: &Node
) -> Insert {
    match insert_l {
        Insert::Done(done_n) => Insert::Done(n3(
            done_n,
            m.clone(),
            r.clone(),
        )),
        Insert::Up(up_l, up_r) => Insert::Up(
                n2(up_l, up_r),
                n2(m.clone(), r.clone()),
            ),
    }
}

fn n3_handle_insert_m(
    l: &Node, insert_m: Insert, r: &Node
) -> Insert {
    match insert_m {
        Insert::Done(done_n) => Insert::Done(n3(
            l.clone(),
            done_n,
            r.clone(),
        )),
        Insert::Up(up_l, up_r) => Insert::Up(
            n2(l.clone(), up_l),
            n2(up_r.clone(), r.clone()),
        ),
    }
}

fn n3_handle_insert_r(
    l: &Node, m: &Node, insert_r: Insert
) -> Insert {
    match insert_r {
        Insert::Done(done_n) => Insert::Done(n3(
            l.clone(),
            m.clone(),
            done_n,
        )),
        Insert::Up(up_l, up_r) => Insert::Up(
            n2(l.clone(), m.clone()),
            n2(up_l, up_r),
        ),
    }
}

fn n2_handle_remove_l(remove_l: Remove, r: &Node) -> Remove {
    match remove_l {
        Remove::Empty => Remove::Up(r.clone()),
        Remove::Done(done_n) => Remove::Done(n2(
            done_n,
            r.clone(),
        )),
        Remove::Up(up_n) => match r {
            Leaf(_) => unreachable!(),
            N2(n) => {
                let (ref rl, _, ref rr, _) = &(**n);
                Remove::Up(n3(
                    up_n,
                    rl.clone(),
                    rr.clone(),
                ))
            }
            N3(n) => {
                let (ref rl, _, ref rm, _, ref rr, _) = &(**n);
                Remove::Done(n2(
                    n2(
                        up_n,
                        rl.clone(),
                    ),
                    n2(
                        rm.clone(),
                        rr.clone(),
                    ),
                ))
            }
        }
    }
}

fn n2_handle_remove_r(l: &Node, remove_r: Remove) -> Remove {
    match remove_r {
        Remove::Empty => Remove::Up(l.clone()),
        Remove::Done(done_n) => Remove::Done(n2(
            l.clone(),
            done_n,
        )),
        Remove::Up(up_n) => match l {
            Leaf(_) => unreachable!(),
            N2(n) => {
                let (ref ll, _, ref lr, _) = &(**n);
                Remove::Up(n3(
                    ll.clone(),
                    lr.clone(),
                    up_n,
                ))
            }
            N3(n) => {
                let (ref ll, _, ref lm, _, ref lr, _) = &(**n);
                Remove::Done(n2(
                    n2(
                        ll.clone(),
                        lm.clone(),
                    ),
                    n2(
                        lr.clone(),
                        up_n,
                    ),
                ))
            }
        }
    }
}

fn n3_handle_remove_l(
    remove_l: Remove, m: &Node, r: &Node
) -> Remove {
    match remove_l {
        Remove::Empty => Remove::Done(n2(
            m.clone(),
            r.clone(),
        )),
        Remove::Done(done_n) => Remove::Done(n3(
            done_n,
            m.clone(),
            r.clone(),
        )),
        Remove::Up(up_n) => match m {
            Leaf(_) => unreachable!(),
            N2(n) => {
                let (ref ml, _, ref mr, _) = &(**n);
                Remove::Done(n2(
                    n3(
                        up_n,
                        ml.clone(),
                        mr.clone(),
                    ),
                    r.clone()
                ))
            }
            N3(n) => {
                let (ref ml, _, ref mm, _, ref mr, _) = &(**n);
                Remove::Done(n3(
                    n2(
                        up_n,
                        ml.clone(),
                    ),
                    n2(mm.clone(),
                    mr.clone(),),
                    r.clone(),
                ))
            }
        }
    }
}

fn n3_handle_remove_m(l: &Node, remove_m: Remove, r: &Node) -> Remove {
    match remove_m {
        Remove::Empty => Remove::Done(n2(
            l.clone(),
            r.clone(),
        )),
        Remove::Done(done_n) => Remove::Done(n3(
            l.clone(),
            done_n,
            r.clone(),
        )),
        Remove::Up(up_n) => match r {
            Leaf(_) => unreachable!(),
            N2(n) => {
                let (ref rl, _, ref rr, _) = &(**n);
                Remove::Done(n2(
                    l.clone(),
                    n3(
                        up_n,
                        rl.clone(),
                        rr.clone(),
                    ),
                ))
            }
            N3(n) => {
                let (ref rl, _, ref rm, _, ref rr, _) = &(**n);
                Remove::Done(n3(
                    l.clone(),
                    n2(
                        up_n,
                        rl.clone(),
                    ),
                    n2(
                        rm.clone(),
                        rr.clone(),
                    ),
                ))
            }
        }
    }
}

fn n3_handle_remove_r(l: &Node, m: &Node, remove_r: Remove) -> Remove {
    match remove_r {
        Remove::Empty => Remove::Done(n2(
            l.clone(),
            m.clone(),
        )),
        Remove::Done(done_n) => Remove::Done(n3(
            l.clone(),
            m.clone(),
            done_n,
        )),
        Remove::Up(up_n) => match m {
            Leaf(_) => unreachable!(),
            N2(n) => {
                let (ref ml, _, ref mr, _) = &(**n);
                Remove::Done(n2(
                    l.clone(),
                    n3(
                        ml.clone(),
                        mr.clone(),
                        up_n,
                    ),
                ))
            }
            N3(n) => {
                let (ref ml, _, ref mm, _, ref mr, _) = &(**n);
                Remove::Done(n3(
                    l.clone(),
                    n2(
                        ml.clone(),
                        mm.clone(),
                    ),
                    n2(
                        mr.clone(),
                        up_n,
                    ),
                ))
            }
        }
    }
}

// traverse left spine of greater node for h_diff, then merge
fn join_lesser_smaller(lesser: &Node, greater: &Node, h_diff: u8) -> Insert {
    if h_diff == 0 {
        Insert::Up(lesser.clone(), greater.clone())
    } else {
        match greater {
            Leaf(_) => unreachable!(),
            N2(n) => {
                let (ref gl, _, ref gr, _) = &(**n);
                n2_handle_insert_l(
                    join_lesser_smaller(lesser, gl, h_diff - 1), gr
                )
            }
            N3(n) => {
                let (ref gl, _, ref gm, _, ref gr, _) = &(**n);
                n3_handle_insert_l(
                    join_lesser_smaller(
                        lesser, gl, h_diff - 1
                    ),
                    gm, gr,
                )
            }
        }
    }
}

fn join_greater_smaller(lesser: &Node, greater: &Node, h_diff: u8) -> Insert {
    if h_diff == 0 {
        Insert::Up(lesser.clone(), greater.clone())
    } else {
        match lesser {
            Leaf(_) => unreachable!(),
            N2(n) => {
                let (ref ll, _, ref lr, _) = &(**n);
                n2_handle_insert_r(
                    ll, join_greater_smaller(lr, greater, h_diff - 1)
                )
            }
            N3(n) => {
                let (ref ll, _, ref lm, _, lr, _) = &(**n);
                n3_handle_insert_r(
                    ll, lm,
                    join_greater_smaller(
                        lr, greater, h_diff - 1
                    ),
                )
            }
        }
    }
}

enum Insert {
    Done(Node),
    Up(Node, Node),
}

enum Remove {
    Empty,
    Done(Node),
    Up(Node),
}

#[derive(Debug, Clone)]
pub struct Cursor(Vec<Position>);

#[derive(Debug, Clone)]
enum Position {
    N2Left(Gc<(Node, usize, Node, usize /* count */)>),
    N2Right(Gc<(Node, usize, Node, usize /* count */)>),
    N2Post(Gc<(Node, usize, Node, usize /* count */)>),
    N3Left(Gc<(Node, usize, Node, usize, Node, usize /* count */)>),
    N3Middle(Gc<(Node, usize, Node, usize, Node, usize /* count */)>),
    N3Right(Gc<(Node, usize, Node, usize, Node, usize /* count */)>),
    N3Post(Gc<(Node, usize, Node, usize, Node, usize /* count */)>),
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
                if n.2.is_leaf() {
                    positions[len - 1] = N2Right(n);
                    return true;
                } else {
                    positions[len - 1] = N2Right(n.clone());
                    n.2.positions_start(positions);
                    return true;
                }
            }
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
                if n.2.is_leaf() {
                    positions[len - 1] = N3Middle(n);
                    return true;
                } else {
                    positions[len - 1] = N3Middle(n.clone());
                    n.2.positions_start(positions);
                    return true;
                }
            }
            N3Middle(n) => {
                if n.4.is_leaf() {
                    positions[len - 1] = N3Right(n);
                    return true;
                } else {
                    positions[len - 1] = N3Right(n.clone());
                    n.4.positions_start(positions);
                    return true;
                }
            }
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
            N2Right(n) => {
                if n.0.is_leaf() {
                    positions[len - 1] = N2Left(n);
                    return true;
                } else {
                    positions[len - 1] = N2Left(n.clone());
                    n.0.positions_end(positions);
                    return true;
                }
            }
            N2Post(n) => {
                if n.2.is_leaf() {
                    positions[len - 1] = N2Right(n);
                    return true;
                } else {
                    positions[len - 1] = N2Right(n.clone());
                    n.2.positions_end(positions);
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
            N3Middle(n) => {
                if n.0.is_leaf() {
                    positions[len - 1] = N3Left(n);
                    return true;
                } else {
                    positions[len - 1] = N3Left(n.clone());
                    n.0.positions_end(positions);
                    return true;
                }
            }
            N3Right(n) => {
                if n.2.is_leaf() {
                    positions[len - 1] = N3Middle(n);
                    return true;
                } else {
                    positions[len - 1] = N3Middle(n.clone());
                    n.2.positions_end(positions);
                    return true;
                }
            }
            N3Post(n) => {
                if n.4.is_leaf() {
                    positions[len - 1] = N3Right(n);
                    return true;
                } else {
                    positions[len - 1] = N3Right(n.clone());
                    n.4.positions_end(positions);
                    return true;
                }
            }
        };
    }
}

impl Cursor {
    pub fn new(n: &Node, at: usize) -> Self {
        unimplemented!();
        // let mut buf = Vec::new();
        // n.positions_start(&mut buf);
        // return Cursor(buf);
    }

    pub fn new_start(n: &Node) -> Self {
        let mut buf = Vec::new();
        n.positions_start(&mut buf);
        return Cursor(buf);
    }

    pub fn new_end(n: &Node) -> Self {
        let mut buf = Vec::new();
        n.positions_end(&mut buf);
        return Cursor(buf);
    }

    // TODO other creation functions

    pub fn current(&mut self) -> Option<Value> {
        let len = self.0.len();
        match &self.0[len - 1] {
            N2Left(n) => {
                let (ref l, _, _, _) = &(**n);
                match l {
                    Node::Leaf(v) => Some(v.clone()),
                    _ => unreachable!(),
                }
            }
            N2Right(n) => {
                let (_, _, ref r, _) = &(**n);
                match r {
                    Node::Leaf(v) => Some(v.clone()),
                    _ => unreachable!(),
                }
            }
            N3Left(n) => {
                let (ref l, _, _, _, _, _) = &(**n);
                match l {
                    Node::Leaf(v) => Some(v.clone()),
                    _ => unreachable!(),
                }
            }
            N3Middle(n) => {
                let (_, _, ref m, _, _, _) = &(**n);
                match m {
                    Node::Leaf(v) => Some(v.clone()),
                    _ => unreachable!(),
                }
            }
            N3Right(n) => {
                let (_, _, _, _, ref r, _) = &(**n);
                match r {
                    Node::Leaf(v) => Some(v.clone()),
                    _ => unreachable!(),
                }
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

impl PartialEq for Arr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Arr::Empty, Arr::Empty) => true,
            (Arr::Empty, Arr::NonEmpty(_, _)) => false,
            (Arr::NonEmpty(_, _), Arr::Empty) => false,
            (Arr::NonEmpty(self_n, _), Arr::NonEmpty(other_n, _)) => self_n.eq(other_n),
        }
    }
}
impl Eq for Arr {}

impl PartialOrd for Arr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Arr {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Arr::Empty, Arr::Empty) => Equal,
            (Arr::Empty, Arr::NonEmpty(_, _)) => Less,
            (Arr::NonEmpty(_, _), Arr::Empty) => Greater,
            (Arr::NonEmpty(self_n, _), Arr::NonEmpty(other_n, _)) => self_n.cmp(other_n),
        }
    }
}

impl PartialEq for Node {
    // Neither arg may be a leaf.
    fn eq(&self, other: &Self) -> bool {
        let mut ca = Cursor::new(&self, 0);
        let mut cb = Cursor::new(&other, 0);

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
        let mut ca = Cursor::new(&self, 0);
        let mut cb = Cursor::new(&other, 0);

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
            Iter(Some(Cursor::new(n, 0)))
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

//////////////////////////////////////// debug/testing stuff

pub fn arr_to_vec(m: &Arr, out: &mut Vec<Value>) {
    match m {
        Arr::Empty => {}
        Arr::NonEmpty(n, _) => node_to_vec(n, out),
    }
}

fn node_to_vec(n: &Node, out: &mut Vec<Value>) {
    match n {
        Leaf(v) => out.push(v.clone()),
        N2(n) => {
            let (ref l, _, ref r, _) = &(**n);
            node_to_vec(l, out);
            node_to_vec(r, out);
        }
        N3(n) => {
            let (ref l, _, ref m, _, ref r, _) = &(**n);
            node_to_vec(l, out);
            node_to_vec(m, out);
            node_to_vec(r, out);
        }
    }
}

fn fuzzy_cursor(data: &[u8]) {
    let mut control = Vec::new();
    let mut m = Arr::new();
    let half = data.len() / 2;

    for data in data.chunks_exact(2) {
        let b = data[0];
        let at = data[1] as usize;
        match b {
            0...63 => {
                if at > control.len() {
                    continue;
                }
                m = m.insert(at, Value::int((b & 0b0011_1111) as i64));
                control.insert(at, Value::int((b & 0b0011_1111) as i64));
            }
            64...127 => {
                if at >= control.len() {
                    continue;
                }
                m = m.remove(at);
                control.remove(at);
            }
            128...191 => {
                if at > control.len() {
                    continue;
                }
                let (l, _) = m.split(at);
                m = l;
                control.split_off(at);
            }
            192...255 => {
                if at > control.len() {
                    continue;
                }
                let (_, r) = m.split(at);
                m = r;
                let new_control = control.split_off(at);
                control = new_control;
            }
        }
    }

    let out_control: Vec<Value> = control.into_iter().collect();
    let len = out_control.len();
    if len <= 1 {
        return;
    } else {
        let (mut cursor, mut control_index) = if data[0] % 2 == 0 {
            (
                match m.cursor_start() {
                    ActualCursor::Cursor(c) => c,
                    _ => unreachable!(),
                },
                0,
            )
        } else {
            (
                match m.cursor_end() {
                    ActualCursor::Cursor(c) => c,
                    _ => unreachable!(),
                },
                len - 1,
            )
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
                    Some(v) => assert!(v == out_control[control_index]),
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

#[test]
fn test_fuzzy_cursor() {
    fuzzy_cursor(&[0x1,0x0,0x0,0x0,0x0,0x1,0x1,0x0]);
    // fuzzy_cursor(&[0x1,0x0,0x0,0x0,0x1,0x1,0x0,0x0]);
    // fuzzy_cursor(&[0x1,0x0,0x0,0x0]);
    // fuzzy_cursor(&[0x1,0x0]);
}
