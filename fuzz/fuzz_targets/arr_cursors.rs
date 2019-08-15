#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate pavo_bootstrap;

use pavo_bootstrap::{
    value::Value,
    arr::{Arr, arr_to_vec, ActualCursor},
};

fuzz_target!(|data: &[u8]| {
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

        for b in &data[half..] {
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
});
