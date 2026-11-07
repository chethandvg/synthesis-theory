// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama

//! Golden-vector tests. The Python and WebAssembly bindings check the same
//! `vectors/*.json` cases, so all three implementations give the same answers.

use golden_vectors::{Outcome, f64_arg, i64_arg};
use serde_json::{Value, json};
use std::error::Error;
use synthesis_theory as st;

type Res = Result<Value, Box<dyn Error>>;

/// A string: (freq, dur, sr, decay, seed, pluck_brightness_hz) to samples.
type StringFn = fn(f64, f64, u32, f64, u64, Option<f64>) -> st::Result<Vec<f64>>;

/// Argument `i` as a whole number of type `T`; out-of-range values (a negative
/// count, say) are rejected, as the Python and JavaScript bindings reject them.
fn int<T: TryFrom<i64>>(args: &[Value], i: usize) -> Result<T, Box<dyn Error>>
where
    T::Error: Error + 'static,
{
    Ok(T::try_from(i64_arg(args, i))?)
}

/// Runs the crate's function `name`; `None` if there is no such function.
fn call(name: &str, a: &[Value]) -> Option<Res> {
    let x = |i| f64_arg(a, i);
    let run = |f: &dyn Fn() -> Res| Some(f());
    // pluck_brightness_hz is a number or null.
    let string = |f: StringFn| -> Res {
        Ok(json!(f(
            x(0),
            x(1),
            int(a, 2)?,
            x(3),
            int(a, 4)?,
            a[5].as_f64()
        )?))
    };
    match name {
        "modal_chime" => run(&|| Ok(json!(st::modal_chime(x(0), x(1), x(2), int(a, 3)?)?))),
        "fm_bell" => run(&|| Ok(json!(st::fm_bell(x(0), x(1), x(2), x(3), int(a, 4)?)?))),
        "fm_tone" => run(&|| Ok(json!(st::fm_tone(x(0), x(1), x(2), x(3), int(a, 4)?)?))),
        "fm_spectrum" => run(&|| {
            Ok(json!(st::fm_spectrum(
                x(0),
                x(1),
                x(2),
                int(a, 3)?,
                x(4),
                int(a, 5)?
            )?))
        }),
        "karplus_strong" => run(&|| string(st::karplus_strong)),
        "karplus_strong_tuned" => run(&|| string(st::karplus_strong_tuned)),
        "karplus_strong_pitch" => run(&|| Ok(json!(st::karplus_strong_pitch(x(0), int(a, 1)?)?))),
        "karplus_strong_detune_cents" => {
            run(&|| Ok(json!(st::karplus_strong_detune_cents(x(0), int(a, 1)?)?)))
        }
        "soft_voice" => run(&|| Ok(json!(st::soft_voice(x(0), x(1), int(a, 2)?, x(3), x(4))?))),
        "heartbeat" => run(&|| Ok(json!(st::heartbeat(x(0), x(1), int(a, 2)?, x(3))?))),
        "free_bar_roots" => run(&|| Ok(json!(st::free_bar_roots(int(a, 0)?)?))),
        "free_bar_ratios" => run(&|| Ok(json!(st::free_bar_ratios(int(a, 0)?)?))),
        _ => None,
    }
}

/// Maps each vector's `fn` name to this crate's implementation.
fn dispatch(name: &str, args: &[Value]) -> Outcome {
    call(name, args).map_or_else(|| Outcome::unknown(name, args), Outcome::from)
}

#[test]
fn golden_vectors() {
    golden_vectors::check_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/vectors"), dispatch);
}
