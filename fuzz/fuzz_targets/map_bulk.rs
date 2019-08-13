#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate pavo_bootstrap;

use std::collections::BTreeMap;

use pavo_bootstrap::{
    value::Value,
    map::{Map, Foo, map_to_vec},
};

fuzz_target!(|data: &[u8]| {
    // Foo
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
        0...63 => {
            let union_ = m.union(&n);
            map_to_vec(&union_, &mut out);

            let mut tmp = control2.clone();
            for (k, v) in control.into_iter() {
                tmp.insert(k, v);
            }
            out_control = tmp.into_iter().collect();
        }
        64...127 => {
            let intersection = m.intersection(&n);
            map_to_vec(&intersection, &mut out);

            let mut tmp1 = control.clone();
            for k in control2.keys() {
                tmp1.remove(k);
            }
            let mut tmp2 = control2.clone();
            for k in control.keys() {
                tmp2.remove(k);
            }
            let mut tmp = control2.clone();
            for (k, v) in control.into_iter() {
                tmp.insert(k, v);
            }
            for k in tmp1.keys() {
                tmp.remove(k);
            }
            for k in tmp2.keys() {
                tmp.remove(k);
            }
            out_control = tmp.into_iter().collect();
        }
        128...191 => {
            let diff = m.difference(&n);
            map_to_vec(&diff, &mut out);

            let mut tmp = control.clone();
            for k in control2.keys() {
                tmp.remove(k);
            }
            out_control = tmp.into_iter().collect();
        }
        192...255 => {
            let sym_diff = m.symmetric_difference(&n);
            map_to_vec(&sym_diff, &mut out);

            let mut tmp1 = control.clone();
            for k in control2.keys() {
                tmp1.remove(k);
            }
            let mut tmp2 = control2.clone();
            for k in control.keys() {
                tmp2.remove(k);
            }
            let mut tmp3 = control2.clone();
            for (k, v) in control.clone().into_iter() {
                tmp3.insert(k, v);
            }
            for k in tmp1.keys() {
                tmp3.remove(k);
            }
            for k in tmp2.keys() {
                tmp3.remove(k);
            }
            let mut tmp = control2.clone();
            for (k, v) in control.into_iter() {
                tmp.insert(k, v);
            }
            for k in tmp3.keys() {
                tmp.remove(k);
            }
            out_control = tmp.into_iter().collect();
        }
    }

    if out != out_control {
        println!("{:?}", out_control);
        println!("{:?}", out);
    }

    assert!(out == out_control);
});
