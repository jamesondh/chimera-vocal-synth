/**
 * Which voice of the web app takes a key (src/ui/voiceAlloc.ts): the plugin's rules, less MPE
 * (docs/plugin.md, "Voices"). The cases are tests/voice-rules.json, which the plugin's tests run
 * too (crates/chimera-plugin/tests/polyphony.rs): the rules are written twice, in TypeScript and
 * in Rust, and one table holds both to the same answers.
 */
import { expect, test } from "bun:test";
import { VoiceAlloc } from "../src/ui/voiceAlloc.ts";
import rules from "./voice-rules.json";

const of = (n: number) => {
  const a = new VoiceAlloc();
  a.count = n;
  return a;
};
const press = (a: VoiceAlloc, midi: number) => a.press(midi, a.pick(midi));

for (const rule of rules as { name: string; voices: number; steps: ["on" | "off", number, number[]][] }[]) {
  test(rule.name, () => {
    const a = of(rule.voices);
    for (const [i, [what, midi, sounding]] of rule.steps.entries()) {
      if (what === "on") press(a, midi);
      else a.lift(midi);
      // the note each voice sounds: its newest key still down (0: none)
      expect([i, a.keys.slice(0, rule.voices).map((k) => k.at(-1) ?? 0)]).toEqual([i, sounding]);
    }
  });
}

test("a lift names the voice and the key it goes back to", () => {
  const a = of(1);
  press(a, 60);
  press(a, 64);
  expect(a.lift(64)).toEqual({ voice: 0, back: 60 });
  expect(a.lift(60)).toEqual({ voice: 0, back: null });
  expect(a.lift(60)).toBeNull();
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
