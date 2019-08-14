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

//
// use std::collections::BTreeArr;

// fn fuzzy(data: &[u8]) {
//     // Foo
//     let mut control = BTreeArr::new();
//     let mut m = Arr::new();
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
//                 let mut new_control = BTreeArr::new();
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
//                 let mut new_control = BTreeArr::new();
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
//     let mut control = BTreeArr::new();
//     let mut m = Arr::new();
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
//                 let mut new_control = BTreeArr::new();
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
//                 let mut new_control = BTreeArr::new();
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
//     let mut control = BTreeArr::new();
//     let mut control2 = BTreeArr::new();
//     let mut m = Arr::new();
//     let mut n = Arr::new();
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
