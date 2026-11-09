/**
 * An alien's head in the side view: the human outlines (profile.ts) reshaped, and the parts no
 * human has. The default alien is homesick and old: a domed cranium, big eyes and a long wrinkled
 * neck (the throat setting stretches it). Its organs show when it has them: the throat sac
 * swelling with its breath, the crest arching back over the skull, and its other mouths as faint
 * copies of the first's tract set back into the head.
 * Everything is in body cm, like the profile it changes.
 */
import type { AlienShape } from "../../wasm/types.ts";
import type { VoiceModel } from "../analysis.ts";
import type { Vec } from "./geom.ts";
import { catmullRom } from "./geom.ts";
import { bodyPoint, ellipse, VIEW, type Profile } from "./profile.ts";

const smooth = (a: number, b: number, x: number) => {
  const t = Math.max(0, Math.min(1, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

/** the visible part of an alien's head (body cm): the human view scaled to it, with room for the
 * long neck below and the crest above */
export function alienView(model: VoiceModel, a: AlienShape): typeof VIEW {
  const k = model.tract.scale;
  const B = bodyPoint(model);
  return {
    // the dome pushes the back of the skull out to ≈−12.7 (profile.ts's −8.7, 1.3 × 1.12 from 0.2)
    x0: -13.2 * k - (a.crest > 0 ? 4 * a.crest * k : 0),
    x1: VIEW.x1 * k,
    // a little lower than a human's: a crest makes the view tall, and the measurements at the
    // glottis then fell off the bottom of the canvas
    y0: B(0, VIEW.y0 - 1.2).y,
    // the dome rises to ≈25.6 (profile.ts's crown, 22.9, pushed out 1.3× from 14.2), with room
    // above it for the screen's header and view toggles, which sit over the canvas's top edge
    y1: Math.max(27.6, a.crest > 0 ? 28 + 5 * a.crest : 0) * k,
  };
}

/** the human outlines with the cranium domed and widened (points above the brow pushed out) */
export function alienProfile(model: VoiceModel, g: Profile): Profile {
  const k = model.tract.scale;
  const c = { x: 0.2 * k, y: 14.2 * k };
  const dome = (p: Vec): Vec => {
    const t = smooth(11.5, 18.5, p.y / k);
    if (t <= 0) return p;
    const s = 1 + 0.3 * t;
    const back = p.x < c.x ? 1 + 0.12 * t : 1;
    return { x: c.x + (p.x - c.x) * s * back, y: c.y + (p.y - c.y) * s };
  };
  const B = bodyPoint(model);
  const eyeC = B(6.0, 15.3);
  return {
    ...g,
    skinUpper: g.skinUpper.map(dome),
    inset: g.inset.map(dome),
    skull: g.skull.map(dome),
    brain: g.brain.map(dome),
    sulcus: g.sulcus.map(dome),
    eye: ellipse(eyeC, 1.85 * k, 1.55 * k),
    eyeC,
  };
}

/** wrinkles across the front and back of the neck, one every ≈0.9 cm (more on a longer neck) */
export function neckWrinkles(model: VoiceModel): Vec[][] {
  const k = model.tract.scale;
  const B = bodyPoint(model);
  const out: Vec[][] = [];
  for (let y = -4.2; y <= 2.2; y += 0.55) {
    const yy = B(0, y).y;
    const front = B(2.4 + 0.08 * Math.max(0, y), y).x;
    const w = (0.9 + 0.35 * Math.sin(y * 3.1)) * k;
    out.push(catmullRom([{ x: front - w, y: yy + 0.12 * k }, { x: front - w * 0.45, y: yy - 0.06 * k }, { x: front + 0.05 * k, y: yy + 0.1 * k }], 4));
    const back = B(-6.6, y).x;
    out.push(catmullRom([{ x: back - 0.05 * k, y: yy + 0.05 * k }, { x: back + w * 0.55, y: yy - 0.1 * k }, { x: back + w, y: yy + 0.08 * k }], 4));
  }
  return out;
}

/** the throat sac under the jaw: its size from the shape, swelling with the breath (0..1) */
export function throatSac(model: VoiceModel, sac: number, breath: number): Vec[] {
  const k = model.tract.scale;
  const c = bodyPoint(model)(3.6, 1.6);
  const s = (0.55 + 0.45 * breath) * k;
  return ellipse({ x: c.x + (0.4 + 1.6 * sac) * s, y: c.y }, (0.7 + 2.0 * sac) * s, (0.6 + 1.5 * sac) * s, 0.25);
}

/** the crest: a hollow tube from the top of the nose arching back over the skull */
export function crest(model: VoiceModel, size: number): Vec[][] {
  const k = model.tract.scale;
  const B = bodyPoint(model);
  const mid = catmullRom([B(2.0, 14.0), B(4.4, 19.5), B(2.8, 23.8 + 2 * size), B(-2.5, 25.5 + 4 * size), B(-7.5 - 4 * size, 23.5 + 4 * size), B(-10 - 5 * size, 19 + 3 * size)], 6);
  const half = 0.42 * k;
  const side = (sgn: number) =>
    mid.map((p, i) => {
      const q = mid[Math.min(mid.length - 1, i + 1)], o = mid[Math.max(0, i - 1)];
      const dx = q.x - o.x, dy = q.y - o.y, n = Math.hypot(dx, dy) || 1;
      return { x: p.x - (dy / n) * half * sgn, y: p.y + (dx / n) * half * sgn };
    });
  return [side(1), side(-1)];
}

/** where each of an alien's other mouths is drawn, from the first's (body cm): back into the head
 * and a little up, as if seen through it */
export function mouthOffsets(model: VoiceModel, a: AlienShape): Vec[] {
  const k = model.tract.scale;
  return Array.from({ length: Math.max(0, Math.round(a.mouths) - 1) }, (_, i) => ({ x: -1.6 * (i + 1) * k, y: 0.6 * (i + 1) * k }));
}
