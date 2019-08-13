use gc::{Gc, Trace, Finalize, custom_trace};
use gc_derive::{Trace, Finalize};

use crate::value::Value;

// A balanced Rope based on 2-3 trees.
#[derive(Debug, Clone, Trace, Finalize)]
pub enum Arr {
    Empty,
    Singleton(Value),
    Tree(Node, usize /* count of items in the node */),
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Node {
    Leaf(Value),
    N2(Gc<Node>, usize /* count of left subtree */, Gc<Node>),
    N3(
        Gc<Node>, usize /* count of left subtree */,
        Gc<Node>, usize /* sum of counts of left and middle subtree */,
        Gc<Node>,
    ),
}

impl Node {
    // fn remove(&self, idx: usize) -> Removal {
    //     match self {
    //
    //     }
    //     unimplemented!()
    // }
}

enum Removal {
    Done(Node),
    BubbleUp(Node),
}


// use gc::{Gc, Trace, Finalize, custom_trace};
// use gc_derive::{Trace, Finalize};
//
// use crate::value::Value;
//
// // A balanced Rope based on 2-3 trees.
// #[derive(Debug, Clone, Trace, Finalize)]
// pub enum Arr {
//     Empty,
//     Singleton(Value),
//     Tree(Node),
// }
//
// impl Arr {
//     /// Create an empty Arr.
//     pub fn new() -> Self {
//         Arr::Empty
//     }
//
//     /// Create an Arr containing one element.
//     pub fn unit(v: Value) -> Self {
//         Arr::Singleton(v)
//     }
//
//     /// Return the number of elements in the Arr.
//     /// Time: O(1)
//     pub fn len(&self) -> usize {
//         match self {
//             Arr::Empty => 0,
//             Arr::Singleton(_) => 1,
//             Arr::Tree(node) => node.len(),
//         }
//     }
//
//     /// Retrieve by index.
//     /// Time: O(log n)
//     pub fn get(&self, idx: usize) -> Option<&Value> {
//         match self {
//             Arr::Empty => None,
//             Arr::Singleton(v) => if idx == 0 { Some(v) } else { None }
//             Arr::Tree(node) => node.get(idx),
//         }
//     }
//
//     // If 0 <= idx <= self.len(), returns a new Arr where v has been inserted at position idx.
//     // Panics if index is out of bounds.
//     // Time: O(log n)
//     pub fn insert(&self, idx: usize, v: Value) -> Arr {
//         match self {
//             Arr::Empty => if idx == 0 { Arr::Singleton(v) } else { panic!() },
//             Arr::Singleton(old) => match idx {
//                 0 => Arr::Tree(Terminal2(v, old.clone())),
//                 1 => Arr::Tree(Terminal2(old.clone(), v)),
//                 _ => panic!(),
//             }
//             Arr::Tree(node) => match node.insert(idx, v) {
//                 Insertion::Done(new) => return Arr::Tree(new),
//                 Insertion::BubbleUp(left, right) => return Arr::Tree(Inner2 {
//                     len01: left.len() + right.len(),
//                     len0: left.len(),
//                     child0: left,
//                     child1: right,
//                 }),
//             }
//         }
//     }
//
//     // If 0 <= idx < self.len(), returns a new Arr where the value at index idx has been removed.
//     // Panics if index is out of bounds.
//     // Time: O(log n)
//     pub fn remove(&self, idx: usize) -> Arr {
//         match self {
//             Arr::Empty => panic!(),
//             Arr::Singleton(_) => if idx == 0 {
//                 Arr::Empty
//             } else {
//                 panic!()
//             }
//             Arr::Tree(node) => match node.remove(idx) {
//                 Removal::Done(new) => return Arr::Tree(new),
//                 Removal::BubbleTerminal(v) => return Arr::Singleton(v),
//                 Removal::BubbleTree(node) => return Arr::Tree((*node).clone()),
//             }
//         }
//     }
//
//     // If 0 <= idx < self.len(), returns a new Arr where v replaces the item at position idx.
//     // Panics if index is out of bounds.
//     // Time: O(log n)
//     pub fn update(&self, idx: usize, v: Value) -> Arr {
//         match self {
//             Arr::Empty => panic!(),
//             Arr::Singleton(_) => if idx == 0 {
//                 Arr::Singleton(v)
//             } else {
//                 panic!()
//             }
//             Arr::Tree(node) => match node.update(idx, v) {
//                 yay => Arr::Tree(yay),
//             },
//         }
//     }
// }
//
// #[derive(Debug, Clone, Trace, Finalize)]
// pub enum Node {
//     Inner2 {
//         child0: Gc<Node>,
//         len0: usize, // len of child0
//         child1: Gc<Node>,
//         len01: usize, // len of child_0 plus len of child_1
//     },
//     Inner3 {
//         child0: Gc<Node>,
//         len0: usize, // len of child0
//         child1: Gc<Node>,
//         len01: usize, // len of child_0 plus len of child_1
//         child2: Gc<Node>,
//         len012: usize, // len of child_0 plus len of child_1 plus len of child_2
//     },
//     Terminal2(Value, Value),
//     Terminal3(Value, Value, Value),
// }
// use self::Node::*;
//
// impl Node {
//     /// Return the number of elements in the Node.
//     /// Time: O(1)
//     pub fn len(&self) -> usize {
//         match self {
//             Inner2 {len01, ..} => *len01,
//             Inner3 {len012, ..} => *len012,
//             Terminal2(..) => 2,
//             Terminal3(..) => 3,
//         }
//     }
//
//     /// Retrieve by index.
//     /// Time: O(log n)
//     pub fn get(&self, idx: usize) -> Option<&Value> {
//         match self {
//             Terminal2(v0, v1) => match idx {
//                 0 => Some(v0),
//                 1 => Some(v1),
//                 _ => None,
//             }
//             Terminal3(v0, v1, v2) => match idx {
//                 0 => Some(v0),
//                 1 => Some(v1),
//                 2 => Some(v2),
//                 _ => None,
//             }
//             Inner2 {child0, len0, child1, ..} => {
//                 if idx < *len0 {
//                     return child0.get(idx);
//                 } else {
//                     return child1.get(idx - *len0);
//                 }
//             }
//             Inner3 {child0, len0, child1, len01, child2, ..} => {
//                 if idx < *len0 {
//                     return child0.get(idx);
//                 } else if idx < *len01 {
//                     return child1.get(idx - *len0);
//                 } else {
//                     return child2.get(idx - *len01);
//                 }
//             }
//         }
//     }
//
//     fn insert(&self, idx: usize, v: Value) -> Insertion {
//         match self {
//             Terminal2(v0, v1) => match idx {
//                 0 => Insertion::Done(Terminal3(v, v0.clone(), v1.clone())),
//                 1 => Insertion::Done(Terminal3(v0.clone(), v, v1.clone())),
//                 2 => Insertion::Done(Terminal3(v0.clone(), v1.clone(), v)),
//                 _ => panic!(),
//             }
//
//             Terminal3(v0, v1, v2) => match idx {
//                 0 => Insertion::BubbleUp(
//                     Gc::new(Terminal2(v, v0.clone())),
//                     Gc::new(Terminal2(v1.clone(), v2.clone())),
//                 ),
//                 1 => Insertion::BubbleUp(
//                     Gc::new(Terminal2(v0.clone(), v)),
//                     Gc::new(Terminal2(v1.clone(), v2.clone())),
//                 ),
//                 2 => Insertion::BubbleUp(
//                     Gc::new(Terminal2(v0.clone(), v1.clone())),
//                     Gc::new(Terminal2(v, v2.clone())),
//                 ),
//                 3 => Insertion::BubbleUp(
//                     Gc::new(Terminal2(v0.clone(), v1.clone())),
//                     Gc::new(Terminal2(v2.clone(), v)),
//                 ),
//                 _ => panic!(),
//             }
//
//             Inner2 {child0, len0, child1, len01} => {
//                 if idx < *len0 {
//                     match child0.insert(idx, v) {
//                         Insertion::Done(yay) => Insertion::Done(Inner2 {
//                             len01: yay.len() + child1.len(),
//                             len0: yay.len(),
//                             child0: Gc::new(yay),
//                             child1: child1.clone(),
//                         }),
//                         Insertion::BubbleUp(left, right) => Insertion::Done(Inner3 {
//                             len012: left.len() + right.len() + child1.len(),
//                             len01: left.len() + right.len(),
//                             len0: left.len(),
//                             child0: left,
//                             child1: right,
//                             child2: child1.clone(),
//                         }),
//                     }
//                 } else if idx < *len01 {
//                     match child1.insert(idx - *len0, v) {
//                         Insertion::Done(yay) => Insertion::Done(Inner2 {
//                             len01: child0.len() + yay.len(),
//                             len0: child0.len(),
//                             child0: child0.clone(),
//                             child1: Gc::new(yay),
//                         }),
//                         Insertion::BubbleUp(left, right) => Insertion::Done(Inner3 {
//                             len012: child0.len() + left.len() + right.len(),
//                             len01: child0.len() + left.len(),
//                             len0: child0.len(),
//                             child0: child0.clone(),
//                             child1: left,
//                             child2: right,
//                         }),
//                     }
//                 } else {
//                     panic!()
//                 }
//             }
//
//             Inner3 {child0, len0, child1, len01, child2, len012} => {
//                 if idx < *len0 {
//                     match child0.insert(idx, v) {
//                         Insertion::Done(yay) => Insertion::Done(Inner3 {
//                             len012: yay.len() + child1.len() + child2.len(),
//                             len01: yay.len() + child1.len(),
//                             len0: yay.len(),
//                             child0: Gc::new(yay),
//                             child1: child1.clone(),
//                             child2: child2.clone(),
//                         }),
//                         Insertion::BubbleUp(left, right) => {
//                             let new_child0 = Gc::new(Inner2 {
//                                 len01: left.len() + right.len(),
//                                 len0: left.len(),
//                                 child0: left,
//                                 child1: right,
//                             });
//                             let new_child1 = Gc::new(Inner2 {
//                                 len01: child1.len() + child2.len(),
//                                 len0: child1.len(),
//                                 child0: child1.clone(),
//                                 child1: child2.clone(),
//                             });
//                             Insertion::Done(Inner2 {
//                                 len01: new_child0.len() + new_child1.len(),
//                                 len0: new_child0.len(),
//                                 child0: new_child0,
//                                 child1: new_child1,
//                             })
//                         }
//                     }
//                 } else if idx < *len01 {
//                     match child1.insert(idx - *len0, v) {
//                         Insertion::Done(yay) => Insertion::Done(Inner3 {
//                             len012: child0.len() + yay.len() + child2.len(),
//                             len01: child0.len() + yay.len(),
//                             len0: child0.len(),
//                             child0: child0.clone(),
//                             child1: Gc::new(yay),
//                             child2: child2.clone(),
//                         }),
//                         Insertion::BubbleUp(left, right) => {
//                             let new_child0 = Gc::new(Inner2 {
//                                 len01: child0.len() + left.len(),
//                                 len0: child0.len(),
//                                 child0: child0.clone(),
//                                 child1: left,
//                             });
//                             let new_child1 = Gc::new(Inner2 {
//                                 len01: right.len() + child2.len(),
//                                 len0: right.len(),
//                                 child0: right.clone(),
//                                 child1: child2.clone(),
//                             });
//                             Insertion::Done(Inner2 {
//                                 len01: new_child0.len() + new_child1.len(),
//                                 len0: new_child0.len(),
//                                 child0: new_child0,
//                                 child1: new_child1,
//                             })
//                         }
//                     }
//                 } else if idx < *len012 {
//                     match child2.insert(idx - *len01, v) {
//                         Insertion::Done(yay) => Insertion::Done(Inner3 {
//                             len012: child0.len() + child1.len() + yay.len(),
//                             len01: child0.len() + child1.len(),
//                             len0: child0.len(),
//                             child0: child0.clone(),
//                             child1: child1.clone(),
//                             child2: Gc::new(yay),
//                         }),
//                         Insertion::BubbleUp(left, right) => {
//                             let new_child0 = Gc::new(Inner2 {
//                                 len01: child0.len() + child1.len(),
//                                 len0: child0.len(),
//                                 child0: child0.clone(),
//                                 child1: child1.clone(),
//                             });
//                             let new_child1 = Gc::new(Inner2 {
//                                 len01: left.len() + right.len(),
//                                 len0: left.len(),
//                                 child0: left,
//                                 child1: right,
//                             });
//                             Insertion::Done(Inner2 {
//                                 len01: new_child0.len() + new_child1.len(),
//                                 len0: new_child0.len(),
//                                 child0: new_child0,
//                                 child1: new_child1,
//                             })
//                         }
//                     }
//                 } else {
//                     panic!()
//                 }
//             }
//         }
//     }
//
//     fn remove(&self, idx: usize) -> Removal {
//         match self {
//             Terminal2(v0, v1) => match idx {
//                 0 => Removal::BubbleTerminal(v1.clone()),
//                 1 => Removal::BubbleTerminal(v0.clone()),
//                 _ => panic!(),
//             }
//
//             Terminal3(v0, v1, v2) => match idx {
//                 0 => Removal::Done(Terminal2(v1.clone(), v2.clone())),
//                 1 => Removal::Done(Terminal2(v0.clone(), v2.clone())),
//                 2 => Removal::Done(Terminal2(v0.clone(), v1.clone())),
//                 _ => panic!(),
//             }
//
//             Inner2 {child0, len0, child1, len01} => {
//                 if idx < *len0 {
//                     match child0.remove(idx) {
//                         Removal::Done(yay) => Removal::Done(Inner2 {
//                             len01: yay.len() + child1.len(),
//                             len0: yay.len(),
//                             child0: Gc::new(yay),
//                             child1: child1.clone(),
//                         }),
//
//                         Removal::BubbleTerminal(up) => match **child1 {
//                             Terminal2(ref r0, ref r1) => {
//                                 Removal::BubbleTree(Gc::new(Terminal3(
//                                     up, r0.clone(), r1.clone(),
//                                 )))
//                             }
//                             Terminal3(ref r0, ref r1, ref r2) => {
//                                 Removal::Done(Inner2 {
//                                     child0: Gc::new(Terminal2(up, r0.clone())),
//                                     len0: 2,
//                                     child1: Gc::new(Terminal2(r1.clone(), r2.clone())),
//                                     len01: 4,
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//
//                         Removal::BubbleTree(up) => match **child1 {
//                             Inner2 {ref child0, ref child1, ..} => {
//                                 Removal::BubbleTree(Gc::new(Inner3 {
//                                     len012: up.len() + child0.len() + child1.len(),
//                                     len01: up.len() + child0.len(),
//                                     len0: up.len(),
//                                     child0: up,
//                                     child1: child0.clone(),
//                                     child2: child1.clone(),
//                                 }))
//                             }
//                             Inner3 {ref child0, ref child1, ref child2, ..} => {
//                                 let new_child0 = Gc::new(Inner2 {
//                                     len01: up.len() + child0.len(),
//                                     len0: up.len(),
//                                     child0: up,
//                                     child1: child0.clone(),
//                                 });
//                                 let new_child1 = Gc::new(Inner2 {
//                                     len01: child1.len() + child2.len(),
//                                     len0: child1.len(),
//                                     child0: child1.clone(),
//                                     child1: child2.clone(),
//                                 });
//                                 Removal::Done(Inner2 {
//                                     len01: new_child1.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: new_child1,
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//                     }
//                 } else if idx < *len01 {
//                     match child1.remove(idx - *len0) {
//                         Removal::Done(yay) => Removal::Done(Inner2 {
//                             len01: child0.len() + yay.len(),
//                             len0: child0.len(),
//                             child0: child0.clone(),
//                             child1: Gc::new(yay),
//                         }),
//
//                         Removal::BubbleTerminal(up) => match **child0 {
//                             Terminal2(ref l0, ref l1) => {
//                                 Removal::BubbleTree(Gc::new(Terminal3(
//                                     l0.clone(), l1.clone(), up,
//                                 )))
//                             }
//                             Terminal3(ref l0, ref l1, ref l2) => {
//                                 Removal::Done(Inner2 {
//                                     child0: Gc::new(Terminal2(l0.clone(), l1.clone())),
//                                     len0: 2,
//                                     child1: Gc::new(Terminal2(l2.clone(), up)),
//                                     len01: 4,
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//
//                         Removal::BubbleTree(up) => match **child0 {
//                             Inner2 {ref child0, ref child1, ..} => {
//                                 Removal::BubbleTree(Gc::new(Inner3 {
//                                     len012: child0.len() + child1.len() + up.len(),
//                                     len01: child0.len() + child1.len(),
//                                     len0: child0.len(),
//                                     child0: child0.clone(),
//                                     child1: child1.clone(),
//                                     child2: up,
//                                 }))
//                             }
//                             Inner3 {ref child0, ref child1, ref child2, ..} => {
//                                 let new_child0 = Gc::new(Inner2 {
//                                     len01: child0.len() + child1.len(),
//                                     len0: child0.len(),
//                                     child0: child0.clone(),
//                                     child1: child1.clone(),
//                                 });
//                                 let new_child1 = Gc::new(Inner2 {
//                                     len01: child2.len() + up.len(),
//                                     len0: child2.len(),
//                                     child0: child2.clone(),
//                                     child1: up,
//                                 });
//                                 Removal::Done(Inner2 {
//                                     len01: new_child1.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: new_child1,
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//                     }
//                 } else {
//                     panic!()
//                 }
//             }
//
//             Inner3 {child0, len0, child1, len01, child2, len012} => {
//                 if idx < *len0 {
//                     match child0.remove(idx) {
//                         Removal::Done(yay) => Removal::Done(Inner3 {
//                             len012: yay.len() + child1.len() + child2.len(),
//                             len01: yay.len() + child1.len(),
//                             len0: yay.len(),
//                             child0: Gc::new(yay),
//                             child1: child1.clone(),
//                             child2: child2.clone(),
//                         }),
//
//                         Removal::BubbleTerminal(up) => match **child1 {
//                             Terminal2(ref m0, ref m1) => {
//                                 let new_child0 = Gc::new(Terminal3(
//                                     up, m0.clone(), m1.clone(),
//                                 ));
//                                 Removal::Done(Inner2 {
//                                     len01: new_child0.len() + child2.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: child2.clone(),
//                                 })
//                             }
//                             Terminal3(ref m0, ref m1, ref m2) => {
//                                 let new_child0 = Gc::new(Terminal2(up, m0.clone()));
//                                 let new_child1 = Gc::new(Terminal2(m1.clone(), m2.clone()));
//                                 Removal::Done(Inner3 {
//                                     len012: new_child0.len() + new_child1.len() + child2.len(),
//                                     len01: new_child0.len() + new_child1.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: new_child1,
//                                     child2: child2.clone(),
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//
//                         Removal::BubbleTree(up) => match **child1 {
//                             Inner2 {
//                                 child0: ref m0,
//                                 child1: ref  m1,
//                                 ..
//                               } => {
//                                   let new_child0 = Gc::new(Inner3 {
//                                       len012: up.len() + m0.len() + m1.len(),
//                                       len01: up.len() + m0.len(),
//                                       len0: up.len(),
//                                       child0: up,
//                                       child1: m0.clone(),
//                                       child2: m1.clone(),
//                                   });
//                                 Removal::Done(Inner2 {
//                                     len01: new_child0.len() + child2.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: child2.clone(),
//                                 })
//                             }
//                             Inner3 {
//                                 child0: ref m0,
//                                 child1: ref  m1,
//                                 child2: ref  m2,
//                                 ..
//                               } => {
//                                   let new_child0 = Gc::new(Inner2 {
//                                       len01: up.len() + m0.len(),
//                                       len0: up.len(),
//                                       child0: up,
//                                       child1: m0.clone(),
//                                   });
//                                   let new_child1 = Gc::new(Inner2 {
//                                       len01: m1.len() + m2.len(),
//                                       len0: m1.len(),
//                                       child0: m1.clone(),
//                                       child1: m2.clone(),
//                                   });
//                                 Removal::Done(Inner3 {
//                                     len012: new_child0.len() + new_child1.len() + child2.len(),
//                                     len01: new_child0.len() + new_child1.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: new_child1.clone(),
//                                     child2: child2.clone(),
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//                     }
//                 } else if idx < *len01 {
//                     match child1.remove(idx - *len0) {
//                         Removal::Done(yay) => Removal::Done(Inner3 {
//                             len012: child0.len() + yay.len() + child2.len(),
//                             len01: child0.len() + yay.len(),
//                             len0: child0.len(),
//                             child0: child0.clone(),
//                             child1: Gc::new(yay),
//                             child2: child2.clone(),
//                         }),
//
//                         Removal::BubbleTerminal(up) => match **child0 {
//                             Terminal2(ref l0, ref l1) => {
//                                 let new_child0 = Gc::new(Terminal3(
//                                     l0.clone(), l1.clone(), up,
//                                 ));
//                                 Removal::Done(Inner2 {
//                                     len01: new_child0.len() + child2.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: child2.clone(),
//                                 })
//                             }
//                             Terminal3(ref l0, ref l1, ref l2) => {
//                                 let new_child0 = Gc::new(Terminal2(l0.clone(), l1.clone()));
//                                 let new_child1 = Gc::new(Terminal2(l2.clone(), up));
//                                 Removal::Done(Inner3 {
//                                     len012: new_child0.len() + new_child1.len() + child2.len(),
//                                     len01: new_child0.len() + new_child1.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: new_child1,
//                                     child2: child2.clone(),
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//
//                         Removal::BubbleTree(up) => match **child0 {
//                             Inner2 {
//                                 child0: ref l0,
//                                 child1: ref  l1,
//                                 ..
//                               } => {
//                                   let new_child0 = Gc::new(Inner3 {
//                                       len012: l0.len() + l1.len() + up.len(),
//                                       len01: l0.len() + l1.len(),
//                                       len0: l0.len(),
//                                       child0: l0.clone(),
//                                       child1: l1.clone(),
//                                       child2: up.clone(),
//                                   });
//                                 Removal::Done(Inner2 {
//                                     len01: new_child0.len() + child2.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: child2.clone(),
//                                 })
//                             }
//                             Inner3 {
//                                 child0: ref l0,
//                                 child1: ref  l1,
//                                 child2: ref  l2,
//                                 ..
//                               } => {
//                                   let new_child0 = Gc::new(Inner2 {
//                                       len01: l0.len() + l1.len(),
//                                       len0: l0.len(),
//                                       child0: l0.clone(),
//                                       child1: l1.clone(),
//                                   });
//                                   let new_child1 = Gc::new(Inner2 {
//                                       len01: l2.len() + up.len(),
//                                       len0: l2.len(),
//                                       child0: l2.clone(),
//                                       child1: up,
//                                   });
//                                 Removal::Done(Inner3 {
//                                     len012: new_child0.len() + new_child1.len() + child2.len(),
//                                     len01: new_child0.len() + new_child1.len(),
//                                     len0: new_child0.len(),
//                                     child0: new_child0,
//                                     child1: new_child1,
//                                     child2: child2.clone(),
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//                     }
//                 } else if idx < *len012 {
//                     match child2.remove(idx - *len01) {
//                         Removal::Done(yay) => Removal::Done(Inner3 {
//                             len012: child0.len() + child1.len() + yay.len(),
//                             len01: child0.len() + child1.len(),
//                             len0: child0.len(),
//                             child0: child0.clone(),
//                             child1: child1.clone(),
//                             child2: Gc::new(yay),
//                         }),
//
//                         Removal::BubbleTerminal(up) => match **child1 {
//                             Terminal2(ref m0, ref m1) => {
//                                 let new_child1 = Gc::new(Terminal3(
//                                     m0.clone(), m1.clone(), up,
//                                 ));
//                                 Removal::Done(Inner2 {
//                                     len01: child0.len() + new_child1.len(),
//                                     len0: child0.len(),
//                                     child0: child0.clone(),
//                                     child1: new_child1,
//                                 })
//                             }
//                             Terminal3(ref m0, ref m1, ref m2) => {
//                                 let new_child1 = Gc::new(Terminal2(m0.clone(), m1.clone()));
//                                 let new_child2 = Gc::new(Terminal2(m2.clone(), up));
//                                 Removal::Done(Inner3 {
//                                     len012: child0.len() + new_child1.len() + new_child2.len(),
//                                     len01: child0.len() + new_child1.len(),
//                                     len0: child0.len(),
//                                     child0: child0.clone(),
//                                     child1: new_child1,
//                                     child2: new_child2,
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//
//                         Removal::BubbleTree(up) => match **child0 {
//                             Inner2 {
//                                 child0: ref m0,
//                                 child1: ref  m1,
//                                 ..
//                               } => {
//                                   let new_child1 = Gc::new(Inner3 {
//                                       len012: m0.len() + m1.len() + up.len(),
//                                       len01: m0.len() + m1.len(),
//                                       len0: m0.len(),
//                                       child0: m0.clone(),
//                                       child1: m1.clone(),
//                                       child2: up.clone(),
//                                   });
//                                 Removal::Done(Inner2 {
//                                     len01: child0.len() + new_child1.len(),
//                                     len0: child0.len(),
//                                     child0: child0.clone(),
//                                     child1: new_child1,
//                                 })
//                             }
//                             Inner3 {
//                                 child0: ref m0,
//                                 child1: ref  m1,
//                                 child2: ref  m2,
//                                 ..
//                               } => {
//                                   let new_child1 = Gc::new(Inner2 {
//                                       len01: m0.len() + m1.len(),
//                                       len0: m0.len(),
//                                       child0: m0.clone(),
//                                       child1: m1.clone(),
//                                   });
//                                   let new_child2 = Gc::new(Inner2 {
//                                       len01: m2.len() + up.len(),
//                                       len0: m2.len(),
//                                       child0: m2.clone(),
//                                       child1: up,
//                                   });
//                                 Removal::Done(Inner3 {
//                                     len012: child0.len() + new_child1.len() + new_child2.len(),
//                                     len01: child0.len() + new_child1.len(),
//                                     len0: child0.len(),
//                                     child0: child0.clone(),
//                                     child1: new_child1,
//                                     child2: new_child2,
//                                 })
//                             }
//                             _ => unreachable!("all terminals are at same height"),
//                         }
//                     }
//                 } else {
//                     panic!()
//                 }
//             }
//         }
//     }
//
//     pub fn update(&self, idx: usize, v: Value) -> Node {
//         match self {
//             Terminal2(l, r) => match idx {
//                 0 => Terminal2(v, r.clone()),
//                 1 => Terminal2(l.clone(), v),
//                 _ => panic!(),
//             }
//
//             Terminal3(l, m, r) => match idx {
//                 0 => Terminal3(v, m.clone(), r.clone()),
//                 1 => Terminal3(l.clone(), v, r.clone()),
//                 2 => Terminal3(l.clone(), m.clone(), v),
//                 _ => panic!(),
//             }
//
//             Inner2 {child0, len0, child1, len01} => {
//                 if idx < *len0 {
//                     let yay = child0.update(idx, v);
//                     Inner2 {
//                         child0: Gc::new(yay),
//                         len0: *len0,
//                         child1: child1.clone(),
//                         len01: *len01,
//                     }
//                 } else if idx < *len01 {
//                     let yay = child1.update(idx - *len0, v);
//                     Inner2 {
//                         child0: child0.clone(),
//                         len0: *len0,
//                         child1: Gc::new(yay),
//                         len01: *len01,
//                     }
//                 } else {
//                     panic!()
//                 }
//             }
//
//             Inner3 {child0, len0, child1, len01, child2, len012} => {
//                 if idx < *len0 {
//                     let yay = child0.update(idx, v);
//                     Inner3 {
//                         child0: Gc::new(yay),
//                         len0: *len0,
//                         child1: child1.clone(),
//                         len01: *len01,
//                         child2: child2.clone(),
//                         len012: *len012,
//                     }
//                 } else if idx < *len01 {
//                     let yay = child1.update(idx - *len0, v);
//                     Inner3 {
//                         child0: child0.clone(),
//                         len0: *len0,
//                         child1: Gc::new(yay),
//                         len01: *len01,
//                         child2: child2.clone(),
//                         len012: *len012,
//                     }
//                 } else if idx < *len012 {
//                     let yay = child2.update(idx - *len01, v);
//                     Inner3 {
//                         child0: child0.clone(),
//                         len0: *len0,
//                         child1: child1.clone(),
//                         len01: *len01,
//                         child2: Gc::new(yay),
//                         len012: *len012,
//                     }
//                 } else {
//                     panic!()
//                 }
//             }
//         }
//     }
// }
//
// enum Insertion {
//     Done(Node),
//     BubbleUp(Gc<Node>, Gc<Node>),
// }
//
// enum Removal {
//     Done(Node),
//     BubbleTerminal(Value),
//     BubbleTree(Gc<Node>),
// }
//
// // unimplemented!()
