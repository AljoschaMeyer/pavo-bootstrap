#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate pavo_bootstrap;

use std::collections::BTreeMap;

use pavo_bootstrap::{
    value::Value,
    map::{Map, Foo, map_to_vec},
};

fn control_delete_min(m: &mut BTreeMap<Value, Foo>) {
    if m.len() != 0 {
        let min: Value = m.keys().next().unwrap().clone();
        m.remove(&min);
    }
}

fn control_delete_max(m: &mut BTreeMap<Value, Foo>) {
    if m.len() != 0 {
        let max: Value = m.keys().rev().next().unwrap().clone();
        m.remove(&max);
    }
}

fuzz_target!(|data: &[u8]| {
    // Foo
    let mut control = BTreeMap::new();
    let mut m = Map::new();

    for b in data {
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

                // m = l;
                match k {
                    None => m = l,
                    Some((k, v)) => m = l.insert(k.clone(), v.clone()),
                }

                let mut new_control = BTreeMap::new();
                for (k, v) in control.iter() {
                    // if k < &key {
                    //     new_control.insert(k.clone(), v.clone());
                    // }
                    if k <= &key {
                        new_control.insert(k.clone(), v.clone());
                    }
                }
                control = new_control;
            }
            192...255 => {
                let key = Value::int((b & 0b0011_1111) as i64);
                let (_, k, r) = m.split(&key);

                // m = r;
                match k {
                    None => m = r,
                    Some((k, v)) => m = r.insert(k.clone(), v.clone()),
                }

                let mut new_control = BTreeMap::new();
                for (k, v) in control.iter() {
                    if k >= &key {
                        new_control.insert(k.clone(), v.clone());
                    }
                    // if k > &key {
                    //     new_control.insert(k.clone(), v.clone());
                    // }
                }
                control = new_control;
            }
        }
    }

    let mut out = vec![];
    map_to_vec(&m, &mut out);
    let out_control: Vec<(Value, Foo)> = control.into_iter().collect();

    if out != out_control {
        println!("{:?}", out_control);
        println!("{:?}", out);
    }

    assert!(out == out_control);
});
