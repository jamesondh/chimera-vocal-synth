/**
 * Which voice of the web app takes a key (src/ui/voiceAlloc.ts): the plugin's rules, less MPE
 * (docs/plugin.md, "Voices"; the plugin's own cases are in crates/chimera-plugin/tests/polyphony.rs).
 */
import { expect, test } from "bun:test";
import { VoiceAlloc } from "../src/ui/voiceAlloc.ts";

const of = (n: number) => {
  const a = new VoiceAlloc();
  a.count = n;
  return a;
};
const press = (a: VoiceAlloc, midi: number) => {
  const v = a.pick(midi);
  a.press(midi, v);
  return v;
};

test("one voice takes every key, and they are its keys in order as before", () => {
  const a = of(1);
  expect([press(a, 60), press(a, 64), press(a, 67)]).toEqual([0, 0, 0]);
  expect(a.keys[0]).toEqual([60, 64, 67]);
  expect(a.lift(67)).toEqual({ voice: 0, back: 64 });
  expect(a.lift(60)).toEqual({ voice: 0, back: 64 });
  expect(a.lift(64)).toEqual({ voice: 0, back: null });
});

test("a key takes the voice free for the longest", () => {
  const a = of(3);
  expect([press(a, 60), press(a, 64)]).toEqual([0, 1]);
  a.lift(60); // voice 0 is free from now, voice 2 has been free since the start
  expect(press(a, 67)).toBe(2);
  expect(press(a, 72)).toBe(0);
});

test("a key already down stays on its voice", () => {
  const a = of(3);
  press(a, 60);
  press(a, 64);
  expect(a.pick(60)).toBe(0);
});

test("with every voice busy a key takes the voice whose note started longest ago, which goes back to its older key", () => {
  const a = of(2);
  press(a, 60);
  press(a, 64);
  expect(press(a, 67)).toBe(0);
  expect(a.keys[0]).toEqual([60, 67]);
  // voice 1's note (64) is now the oldest
  expect(press(a, 71)).toBe(1);
  // voice 0 back on 60 is as old as 60's note again: older than voice 1's 71
  expect(a.lift(67)).toEqual({ voice: 0, back: 60 });
  expect(press(a, 74)).toBe(0);
});

test("the newest note's voice is the one shown", () => {
  const a = of(3);
  press(a, 60);
  press(a, 64);
  expect(a.newest()).toBe(1);
  a.lift(64);
  expect(a.newest()).toBe(0);
  a.lift(60);
  expect(a.newest()).toBeNull();
});
