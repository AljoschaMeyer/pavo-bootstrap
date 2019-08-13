#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate pavo_bootstrap;

use std::collections::BTreeMap;

use pavo_bootstrap::{
    value::Value,
    map::{Map, Foo, map_to_vec, Cursor},
};

fuzz_target!(|data: &[u8]| {
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

        for b in &data[half..] {
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
});
