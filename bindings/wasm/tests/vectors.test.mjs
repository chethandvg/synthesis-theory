// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama
// The shared golden vectors, checked against the WebAssembly build.

import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import { checkDir } from "golden-vectors";

const impl = createRequire(import.meta.url)("../pkg/node/synthesis_theory_wasm.js");

test("golden vectors", () => {
  assert.ok(checkDir(new URL("../../../vectors/", import.meta.url), impl) > 0);
});

test("signals are typed arrays and errors throw", () => {
  const string = impl.karplus_strong(440, 0.01, 44100, 0.996, 0, 2000);
  assert.ok(string instanceof Float64Array);
  assert.equal(string.length, 441);
  assert.equal(impl.karplus_strong(440, 0.01, 44100, 0.996, 0).length, 441); // pluck_brightness_hz omitted
  assert.equal(impl.karplus_strong_pitch(440, 44100), 44100 / 100.5);
  assert.throws(() => impl.karplus_strong(440, 0.01, 44100.5, 0.996, 0, null), /sr must be a whole number/);
  assert.throws(() => impl.karplus_strong(440, 0.01, 44100, 0.996, -1, null), /seed must be a whole number/);
  assert.throws(() => impl.karplus_strong(440, 0.01, 44100, 1.5, 0, null), /decay must be in \[0, 1\]/);
  assert.throws(() => impl.free_bar_roots(2 ** 53), /count must be a whole number/);
});

test("10 s of 44.1 kHz instruments synthesise quickly", () => {
  const sr = 44100;
  const start = performance.now();
  const parts = [
    impl.modal_chime(880, 10, 1, sr),
    impl.fm_bell(440, 10, 1.4, 10, sr),
    impl.karplus_strong(220, 10, sr, 0.996, 0, 2000),
    impl.karplus_strong_tuned(220, 10, sr, 0.996, 0, 2000),
    impl.soft_voice(220, 10, sr, 5, 12),
    impl.heartbeat(72, 10, sr, 48),
  ];
  const seconds = (performance.now() - start) / 1000;
  assert.ok(parts.every((p) => p.length === 10 * sr));
  console.log(`# synthesised six 10 s instruments in ${seconds.toFixed(2)} s`);
  assert.ok(seconds < 10, `took ${seconds} s`);
});
