#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate pavo_bootstrap;

use pavo_bootstrap::{
    value::Value,
    arr::{Arr, arr_to_vec},
};

fuzz_target!(|data: &[u8]| {
    // Foo
    let mut control = Vec::new();
    let mut m = Arr::new();

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

    let mut out = vec![];
    arr_to_vec(&m, &mut out);
    let out_control: Vec<Value> = control.into_iter().collect();

    if out != out_control {
        println!("{:?}", out_control);
        println!("{:?}", out);
    }

    assert!(out == out_control);
});
