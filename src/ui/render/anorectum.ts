/**
 * The flatulence creature as an anatomical cutaway: a coronal section through the anal canal seen
 * from behind, in the style of a surgical anatomy plate drawn on the vector display. The rectal
 * ampulla with its gas pocket (sized by the engine's gas volume, lit by its pressure), the pelvic
 * floor, the internal and external sphincters around the canal (the external one lit by its tone),
 * the vibrating anal verge (its gap from the engine, magnified), the ischioanal fat and the
 * gluteal cheeks around the cleft (its gap from the engine's cleft areas), with the gas streaming
 * through at the engine's flow.
 *
 * Drawing units are centimetres, y up, the anal verge at the origin (as in the engine).
 */
import type { CreatureTelemetry } from "../../wasm/types.ts";
import { SLOTS, TEXT, V, heat, pressureLevel } from "../colors.ts";
import { beamPts, dot, label, labelWidth, ring, seg, type Pt } from "./vector.ts";

type P = [number, number];

/** the engine's anatomy at body size 1 (crates/chimera/src/creatures/flatus.rs) */
const HPZ = 3.6;
const VALVE = 1.0;
const ACROSS = 2 * (1.3 + 0.35);
/** the verge's gap is a fraction of a millimetre: drawn this many times wider */
export const GAP_MAGNIFY = 12;

export interface AnorectumState {
  /** particles along the gas's path (0..1 each) and the turbulence's phase */
  parts: number[];
  phase: number;
}

export const newAnorectumState = (): AnorectumState => ({ parts: Array.from({ length: 28 }, (_, i) => i / 28), phase: 0 });

export interface AnorectumLabel {
  text: string;
  x: number;
  y: number;
  color: string;
}

/** a mirrored pair of outlines from one side's points (x < 0 drawn as given, x > 0 mirrored) */
function both(pts: P[]): [P[], P[]] {
  return [pts, pts.map(([x, y]) => [-x, y] as P)];
}

/** a smooth closed-ish curve through points (Chaikin, two passes) */
function soft(pts: P[], it = 2): P[] {
  let p = pts;
  for (let k = 0; k < it; k++) {
    const q: P[] = [p[0]];
    for (let i = 0; i < p.length - 1; i++) {
      const [a, b] = [p[i], p[i + 1]];
      q.push([0.75 * a[0] + 0.25 * b[0], 0.75 * a[1] + 0.25 * b[1]], [0.25 * a[0] + 0.75 * b[0], 0.25 * a[1] + 0.75 * b[1]]);
    }
    q.push(p[p.length - 1]);
    p = q;
  }
  return p;
}

/**
 * Draw the cutaway. `S` maps drawing units to the screen, `sc` is pixels per cm, `live` says
 * whether the telemetry is from the playing voice (else a still preview). Returns labels for the
 * shared placement (drawn when the measurements view is on).
 */
export function drawAnorectum(ctx: CanvasRenderingContext2D, S: (p: P) => Pt, sc: number, tele: CreatureTelemetry, live: boolean, dt: number, st: AnorectumState): AnorectumLabel[] {
  const labels: AnorectumLabel[] = [];
  // a small screen (a phone) keeps only the labels of what the controls move
  const roomy = sc > 16;
  const put = (text: string, p: Pt, color: string, key = false) => {
    if (roomy || key) labels.push({ text, x: p[0], y: p[1], color });
  };
  /** a label that ends at the point (for structures on the left) */
  const putLeft = (text: string, p: Pt, color: string, key = false) => put(text, [p[0] - labelWidth(ctx, text, { size: 10, ls: 0.15 }) - 4, p[1]], color, key);
  const src = tele.source;
  const prm = tele.params;
  const size = prm.size ?? 1;
  const gate = live ? tele.gate : 0;
  const push = (prm.push ?? 0) * gate;
  const tone = prm.tone ?? 20;
  const spread = prm.spread ?? 0.2;
  const gas = Math.max(0, src.gas ?? 0);
  const pressure = src.pressure ?? 0;
  const flow = Math.max(0, live ? (src.flow ?? 0) : 0);
  const gap = Math.max(0, live ? (src.gap ?? 0) : 0);
  const line = (pts: P[], color: string, width: number, alpha = 1, hot = false) => beamPts(ctx, pts.map(S), color, width, { alpha, hot });

  // ---- gas pocket: an ellipse of the engine's volume, resting on the anorectal junction
  const r = Math.cbrt((3 * Math.max(gas, 0.5)) / (4 * Math.PI));
  const rx = r * 0.85, ry = r * 1.15;
  const top = HPZ * size;
  const cy = top + 0.4 + ry;
  // ---- rectal ampulla: distended around the pocket, narrowing to the junction; mucosa inside
  const half = Math.max(2.6, rx + 0.7);
  const ampulla: P[] = soft([[-0.9, top], [-half * 0.75, top + 1.2], [-half, cy - ry * 0.2], [-half * 0.98, cy + ry * 0.6], [-half * 0.7, cy + ry + 1.2], [-1.6, cy + ry + 3.2], [-1.4, cy + ry + 4.6]]);
  const mucosa: P[] = ampulla.map(([x, y]) => [x * 0.86, y] as P);
  for (const side of both(ampulla)) line(side, V.blue, 1.5, 1, true);
  for (const side of both(mucosa)) line(side, V.blueMid, 1, 0.8);
  // transverse rectal folds (Houston's valves): two shelves, alternating sides
  line([[-half * 0.97, cy + ry * 0.35], [-half * 0.45, cy + ry * 0.25]], V.blueMid, 1, 0.7);
  line([[half * 0.92, cy + ry * 0.95], [half * 0.4, cy + ry * 0.85]], V.blueMid, 1, 0.7);
  putLeft("RECTAL AMPULLA", S([-half - 0.3, cy + ry + 0.8]), TEXT.secondary);

  // the push: arrows pressing in on the ampulla (slot 0)
  if (push > 0.3) {
    const len = Math.min(2.2, push / 30);
    for (const [y, sgn] of [[cy - ry * 0.4, -1], [cy + ry * 0.4, -1], [cy - ry * 0.4, 1], [cy + ry * 0.4, 1]] as [number, number][]) {
      const x0 = sgn * (half + 0.6 + len), x1 = sgn * (half + 0.5);
      const [a, b] = [S([x0, y]), S([x1, y])];
      seg(ctx, a[0], a[1], b[0], b[1], SLOTS[0], 1.4, { alpha: 0.9 });
      const h = S([x1 + sgn * 0.45, y + 0.3]), k = S([x1 + sgn * 0.45, y - 0.3]);
      seg(ctx, b[0], b[1], h[0], h[1], SLOTS[0], 1.4);
      seg(ctx, b[0], b[1], k[0], k[1], SLOTS[0], 1.4);
    }
  }

  // the pocket, lit by its pressure (amber is sound and air; 0–60 cmH₂O)
  const lvl = Math.max(0, Math.min(1, pressure / 60));
  const pocket: P[] = [];
  for (let i = 0; i <= 48; i++) {
    const a = (i / 48) * Math.PI * 2;
    pocket.push([rx * Math.cos(a), cy + ry * Math.sin(a)]);
  }
  line(pocket, SLOTS[2], 1.3, 0.55 + 0.45 * lvl, lvl > 0.5);
  // gas: a sparse stipple, denser with pressure
  st.phase = (st.phase + dt * (0.4 + flow / 150)) % 1;
  const nGas = 10 + Math.round(18 * lvl);
  for (let i = 0; i < nGas; i++) {
    const a = i * 2.39996 + st.phase * 0.6, rr = Math.sqrt((i + 0.5) / nGas) * 0.82;
    const [x, y] = S([rx * rr * Math.cos(a), cy + ry * rr * Math.sin(a)]);
    dot(ctx, x, y, 1.1, heat(0.35 + 0.5 * lvl), 0.7);
  }
  const pl = S([0, cy + 0.15]);
  if (roomy) label(ctx, `${Math.round(gas)} ML`, pl[0], pl[1] - 6, { size: 11, color: SLOTS[2], align: "center", ls: 0.12 });
  if (roomy) label(ctx, `${pressure.toFixed(0)} CMH₂O`, pl[0], pl[1] + 10, { size: 10, color: V.amber, align: "center", ls: 0.12, alpha: 0.85 });
  put("GAS", S([half + 0.4, cy + ry * 0.9]), SLOTS[2], true);

  // ---- pelvic floor: levator ani sweeping out from the junction, puborectalis at its root
  for (const s of [-1, 1]) {
    const lev: P[] = soft([[s * 1.9, top - 0.2], [s * 3.5, top + 0.8], [s * 6.5, top + 2.4], [s * 9.2, top + 4.2]]);
    line(lev, V.blueMid, 1.2, 0.85);
    line(lev.map(([x, y]) => [x, y + 0.55] as P), V.blueDim, 1, 0.8);
  }
  put("LEVATOR ANI", S([6.4, top + 3.6]), TEXT.secondary);

  // ---- anal canal: the gas-filled upper canal (engine's area) and the vibrating verge (its gap)
  const canalA = arr(tele.tubes.canal?.areas);
  const upperHalf = canalA.length ? Math.sqrt(Math.max(0, canalA[0]) / Math.PI) : 0.5 * size;
  const vergeHalf = Math.min(0.45, (gap * GAP_MAGNIFY) / 2);
  const valveTop = VALVE * size;
  for (const s of [-1, 1]) {
    line([[s * vergeHalf, 0], [s * vergeHalf, valveTop * 0.9], [s * upperHalf * 0.8, valveTop + 0.2], [s * upperHalf, valveTop + 0.6], [s * upperHalf, top], [s * 1.2, top + 0.3]], V.blue, 1.4, 1, true);
  }
  // canal rungs lit by the acoustic pressure, as the other creatures' airways
  const canalR = arr(tele.tubes.canal?.rms);
  canalR.forEach((rms, i) => {
    const y = valveTop + 0.6 + ((i + 0.5) / canalR.length) * (top - valveTop - 0.6);
    const lv = live ? Math.pow(pressureLevel(rms * 0.6), 0.85) : 0;
    const [a, b] = [S([-upperHalf, y]), S([upperHalf, y])];
    seg(ctx, a[0], a[1], b[0], b[1], live ? heat(lv) : V.blueDim, 1, { alpha: live ? 0.35 + 0.65 * lv : 0.5 });
  });
  putLeft("ANAL CANAL", S([-upperHalf - 1.6, top - 0.6]), TEXT.secondary);

  // internal sphincter: the canal's thickened smooth muscle, ending at the intersphincteric groove
  for (const s of [-1, 1]) {
    const x0 = s * (upperHalf + 0.12), x1 = s * (upperHalf + 0.42);
    line([[x0, top + 0.2], [x0, valveTop + 0.4], [(x0 + x1) / 2, valveTop + 0.15], [x1, valveTop + 0.4], [x1, top + 0.4]], V.blueMid, 1.1, 0.9);
  }
  put("INTERNAL SPHINCTER", S([upperHalf + 1.6, top - 0.3]), TEXT.secondary);

  // external sphincter (slot 1, lit by its tone): deep and superficial parts beside the internal
  // one, the subcutaneous ring wrapping the verge below it
  const tl = Math.max(0, Math.min(1, tone / 100));
  const eas = SLOTS[1];
  for (const s of [-1, 1]) {
    const xi = s * (upperHalf + 0.55), xo = s * (upperHalf + 1.45);
    line([[xi, top + 0.5], [xo, top + 0.7], [xo, valveTop + 0.6], [xi, valveTop + 0.5]], eas, 1.2, 0.45 + 0.55 * tl, tl > 0.5);
    line([[xi, top + 0.5], [xi, valveTop + 0.5]], eas, 1, 0.35 + 0.4 * tl);
    // subcutaneous part: a ring in section around the verge
    const cx = s * (vergeHalf + 0.55), cyy = valveTop * 0.45;
    const sub: P[] = [];
    for (let i = 0; i <= 24; i++) {
      const a = (i / 24) * Math.PI * 2;
      sub.push([cx + 0.48 * Math.cos(a), cyy + 0.42 * Math.sin(a)]);
    }
    line(sub, eas, 1.3, 0.55 + 0.45 * tl, true);
  }
  put("EXTERNAL SPHINCTER", S([upperHalf + 1.6, valveTop + 1.2]), eas, true);

  // the verge as a source glyph: a ring sized by the opening (slot 1, like the tone that drives it)
  const opening = Math.max(0, Math.min(1, live ? (src.sphincter ?? 0) : 0));
  const [vx, vy] = S([0, valveTop * 0.45]);
  if (opening > 0.04) ring(ctx, vx, vy, 4 + 10 * opening, V.amber, 1, { alpha: 0.5 + 0.5 * opening });
  put("ANAL VERGE", S([vergeHalf + 1.2, -0.25]), TEXT.secondary);

  // ---- gluteal cleft and cheeks (slot 3, the spread): the inner surfaces follow the engine's gap
  const cleftA = arr(tele.tubes.cleft?.areas);
  const depth = 3.0 * size;
  const across = ACROSS * size;
  const gapAt = (i: number) => (cleftA.length ? Math.max(0, cleftA[Math.min(cleftA.length - 1, i)]) / across : 0.1);
  const n = Math.max(1, cleftA.length);
  const inner: P[] = [[-(vergeHalf + 0.05), 0]];
  for (let i = 0; i < n; i++) inner.push([-Math.max(0.05, gapAt(i) / 2), -((i + 0.5) / n) * depth]);
  const g = Math.max(0.05, gapAt(n - 1) / 2);
  inner.push([-g - 0.05 * depth, -depth]);
  const lift = spread * 1.5;
  const cheek: P[] = soft([...inner, [-g - 2.2, -depth - 1.6 + lift * 0.5], [-g - 6, -depth - 2.0 + lift], [-g - 9.6, -depth + 0.6 + lift], [-10.6, 2.5], [-10.2, 6.5]]);
  for (const side of both(cheek)) line(side, SLOTS[3], 1.6, 0.9, true);
  // the subcutaneous fat line and the ischioanal fossa's fat, dimmer
  const fat: P[] = soft([[-g - 1.5, -depth - 0.6 + lift * 0.4], [-g - 5.5, -depth - 0.9 + lift], [-g - 8.4, -depth + 1.0 + lift], [-9.2, 3.0], [-8.8, 6.0]]);
  for (const side of both(fat)) line(side, V.blueDim, 1, 0.9);
  const fossa: P[] = soft([[-(upperHalf + 1.7), top + 0.9], [-4.2, top + 1.6], [-5.6, top + 0.2], [-4.8, 0.2], [-2.6, -0.6], [-(vergeHalf + 1.2), -0.15]]);
  for (const side of both(fossa)) line(side, V.blueDim, 1, 0.75);
  putLeft("ISCHIOANAL FAT", S([-2.6, top - 2.0]), TEXT.secondary);
  put("GLUTEAL CLEFT", S([g + 0.6, -depth * 0.6]), SLOTS[3], true);
  put("BUTTOCK", S([-8.6, -depth - 0.4 + lift]), TEXT.secondary);

  // ---- the gas's path: pocket → canal → verge → cleft → out, particles at the engine's flow
  const path: P[] = [[0, cy - ry * 0.3], [0, top], [0, valveTop], [0, 0], [0, -depth], [0, -depth - 2.5]];
  const cum = [0];
  for (let i = 1; i < path.length; i++) cum.push(cum[i - 1] + Math.hypot(path[i][0] - path[i - 1][0], path[i][1] - path[i - 1][1]));
  const total = cum[cum.length - 1];
  const at = (u: number): P => {
    const s = u * total;
    let i = 1;
    while (i < cum.length - 1 && cum[i] < s) i++;
    const f = (s - cum[i - 1]) / Math.max(1e-9, cum[i] - cum[i - 1]);
    return [path[i - 1][0] + (path[i][0] - path[i - 1][0]) * f, path[i - 1][1] + (path[i][1] - path[i - 1][1]) * f];
  };
  // flow (mL/s) through the cleft's mean section sets the drift speed; capped to stay readable
  const speed = Math.min(1.6, flow / 400);
  if (flow > 2) {
    st.parts.forEach((u0, i) => {
      const u = (u0 + dt * speed) % 1;
      st.parts[i] = u;
      const [x, y] = at(u);
      // past the verge the jet fans out across the cleft's gap, then into the air
      const below = y < 0 ? Math.min(1, -y / depth) : 0;
      const wob = Math.sin(i * 12.9898) * (below * (g + 0.3) + (y < -depth ? (-depth - y) * 0.6 : 0));
      const [px, py] = S([x + wob, y]);
      dot(ctx, px, py, 1.4, V.amber, 0.45 + 0.5 * Math.min(1, flow / 300));
    });
    // turbulence at the jet below the verge: dots that flicker with the flow
    const nt = Math.min(24, Math.round(flow / 25));
    for (let i = 0; i < nt; i++) {
      const k = Math.sin((i + 1) * 78.233 + st.phase * 40) * 0.5 + 0.5;
      const j = Math.sin((i + 1) * 12.9898 + st.phase * 23) * 0.5 + 0.5;
      const [px, py] = S([(j - 0.5) * 2 * (g + 0.1), -k * depth * 0.8]);
      dot(ctx, px, py, 1, V.amber, 0.6);
    }
    const fl = S([g + 0.8, -depth - 0.9]);
    label(ctx, `${Math.round(flow)} ML/S`, fl[0], fl[1], { size: 10, color: V.amber, ls: 0.12 });
  }
  // the magnification of the verge's gap, so the drawing doesn't overstate it
  const note = S([vergeHalf + 1.2, -0.9]);
  if (sc > 14) label(ctx, `GAP ×${GAP_MAGNIFY}`, note[0], note[1], { size: 9, color: TEXT.secondary, ls: 0.14, alpha: 0.7 });
  return labels;
}

function arr(a: Float32Array | number[] | undefined): number[] {
  return a ? Array.from(a) : [];
}
