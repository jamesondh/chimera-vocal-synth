/**
 * Typed melodies: solfège names, flats and sharps, octave marks, held steps, rests and bar lines.
 */
import { describe, expect, test } from "bun:test";
import { parseMelody } from "../src/ui/melody.ts";

const midi = (text: string) => parseMelody(text).notes.map((n) => n.midi);
const steps = (text: string) => parseMelody(text).notes.map((n) => n.steps);

describe("parseMelody", () => {
  test("note names from middle C", () => {
    expect(midi("do re mi fa sol la si")).toEqual([60, 62, 64, 65, 67, 69, 71]);
  });

  test("flats, sharps and octave marks", () => {
    expect(midi("sib fa# do(i) la(k) mi(ii) sib(i)")).toEqual([70, 66, 72, 57, 88, 82]);
  });

  test("each _ holds one step longer; - is a rest", () => {
    const m = parseMelody("do sol___ - re_ -__ mi(i)_");
    expect(m.notes.map((n) => n.midi)).toEqual([60, 67, null, 62, null, 76]);
    expect(m.notes.map((n) => n.steps)).toEqual([1, 4, 1, 2, 3, 2]);
    expect(m.error).toBeNull();
  });

  test("bar lines, commas, capitals and line breaks are ignored", () => {
    expect(midi("DO re | Mİ,\nfa ||")).toEqual([60, 62, 64, 65]);
    expect(steps("do_ | re")).toEqual([2, 1]);
  });

  test("the first unreadable token is reported, with the notes before it", () => {
    const m = parseMelody("do re xyz mi");
    expect(m.error).toBe("xyz");
    expect(m.notes.map((n) => n.midi)).toEqual([60, 62]);
    expect(parseMelody("").notes).toEqual([]);
  });
});
