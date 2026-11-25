/**
 * Biomechanical muscle system for the body rig, drawn in CHIMERA's vector-display style
 * (src/ui/render/beams3d.ts & src/ui/colors.ts).
 *
 * 30 major superficial anatomical muscles (15 bilateral pairs across torso, arms and legs):
 *  - Each muscle connects anatomical origin and insertion anchors on the skeleton, tracks its
 *    instantaneous length against its rest length to derive kinematic contraction strain, and
 *    accepts optional dynamic gait/load excitation (`MuscleLoads`, e.g. stance-phase quad/glute/
 *    calf firing during walking).
 *  - Activation follows realistic muscle-twitch dynamics (fast rise ~45 ms, organic decay ~130 ms).
 *  - Contraction swells each muscle belly (constant-volume radius gain), both pushing nearby
 *    wireframe skin vertices outward along their normals (attenuated by subcutaneous body fat)
 *    and rendering glowing striated muscle fiber bundles and cross-sectional rings inside the
 *    body via `Beams` — cool vector blue at rest, flaring amber to hot red-core as they fire.
 */
import * as THREE from "three";
import { Beams, Poly } from "../ui/render/beams3d.ts";
import { V, mix } from "../ui/colors.ts";
import { neutral } from "./pose.ts";
import type { Rig } from "./rig.ts";
import type { SoftTissue } from "./physics.ts";

export type MuscleGroup = "torso" | "arm" | "leg";
export type Side = "L" | "R";

/**
 * Optional per-muscle dynamic load / EMG excitation hints (0..1), keyed by `<base>.<L|R>`,
 * e.g. `"quadriceps.R": 0.85`, `"gastrocnemius.L": 0.9`. Combined with kinematic shortening strain
 * so isometric/eccentric weight-bearing contractions fire accurately during walking.
 */
export type MuscleLoads = Partial<Record<string, number>>;

export interface MuscleLook {
  /** overall brightness of the muscle beams (0: hidden, 1: normal, 1.3: hero) */
  gain: number;
  /** show delicate pink contour rings on active breast/butt soft-tissue oscillators */
  tissueRings: boolean;
  /** how strongly muscle contraction bulges the outer wireframe skin (0..2, default 1) */
  skinBulge: number;
}
export const MUSCLE_LOOK_DEFAULT: Readonly<MuscleLook> = { gain: 0.9, tissueRings: true, skinBulge: 1.0 };

interface MuscleSpec {
  base: string;
  group: MuscleGroup;
  /** origin: bone name (supports `{S}`), fraction along head→tail, local offset [x, y, z] in cm (z mirrored by side) */
  origin: [string, number, [number, number, number]];
  /** mid-belly anchor bone, fraction, and outward arch offset [x, y, z] in cm */
  belly: [string, number, [number, number, number]];
  /** insertion bone, fraction, local offset [x, y, z] in cm */
  insert: [string, number, [number, number, number]];
  /** rest cross-section [width (lateral), depth (radial)] in cm (male baseline; scaled automatically for female) */
  size: [number, number];
  /** outward normal direction in rest space (approx) for belly arch & skin bulge */
  outward: [number, number, number];
  /** longitudinal fiber count (3..5) */
  fibers: number;
  /** max outward skin bulge in cm at full contraction & muscle=1 */
  bulgeMax: number;
  /** fractional shortening for full kinematic activation */
  strainScale: number;
  /** female size multiplier (keeps upper body sleeker & lower body sculpted) */
  femaleScale: number;
}

const SPECS: MuscleSpec[] = [
  // ---- Torso ----
  {
    base: "pectoralis",
    group: "torso",
    origin: ["chest", 0.55, [9.0, 1.5, 2.2]],
    belly: ["upperchest", 0.15, [10.2, -1.5, 7.2]],
    insert: ["upperarm.{S}", 0.28, [1.8, 0.5, -1.0]],
    size: [4.2, 1.9],
    outward: [0.95, 0.15, 0.25],
    fibers: 5,
    bulgeMax: 1.2,
    strainScale: 0.22,
    femaleScale: 0.72,
  },
  {
    base: "rectusAbdominis",
    group: "torso",
    origin: ["hips", 0.35, [9.2, 2.5, 2.4]],
    belly: ["spine2", 0.4, [11.2, 0.0, 2.8]],
    insert: ["chest", 0.45, [10.6, -2.5, 3.0]],
    size: [2.8, 1.4],
    outward: [1, 0, 0.1],
    fibers: 3,
    bulgeMax: 0.8,
    strainScale: 0.16,
    femaleScale: 0.82,
  },
  {
    base: "oblique",
    group: "torso",
    origin: ["hips", 0.55, [2.5, 2.0, 11.2]],
    belly: ["spine2", 0.5, [4.5, 0.0, 11.5]],
    insert: ["chest", 0.45, [4.5, -1.0, 11.0]],
    size: [3.4, 1.3],
    outward: [0.35, 0, 0.94],
    fibers: 4,
    bulgeMax: 0.75,
    strainScale: 0.18,
    femaleScale: 0.82,
  },
  {
    base: "latissimus",
    group: "torso",
    origin: ["spine1", 0.4, [-4.2, 0.5, 3.5]],
    belly: ["chest", 0.4, [-4.5, 0.0, 9.8]],
    insert: ["upperarm.{S}", 0.32, [-1.5, 0.0, -1.0]],
    size: [4.6, 1.6],
    outward: [-0.75, 0, 0.66],
    fibers: 4,
    bulgeMax: 1.0,
    strainScale: 0.22,
    femaleScale: 0.78,
  },
  {
    base: "trapezius",
    group: "torso",
    origin: ["neck", 0.25, [-2.2, -1.0, 2.0]],
    belly: ["upperchest", 0.68, [-2.6, -0.5, 6.2]],
    insert: ["clavicle.{S}", 0.84, [-1.4, 1.0, -0.8]],
    size: [3.2, 1.4],
    outward: [-0.65, 0.65, 0.4],
    fibers: 4,
    bulgeMax: 0.95,
    strainScale: 0.18,
    femaleScale: 0.76,
  },

  // ---- Arms ----
  {
    base: "deltoid",
    group: "arm",
    origin: ["clavicle.{S}", 0.85, [0.3, 1.5, 0.4]],
    belly: ["upperarm.{S}", 0.35, [0.3, 1.8, 2.2]],
    insert: ["upperarm2.{S}", 0.3, [0.2, 0.8, 1.4]],
    size: [3.6, 2.1],
    outward: [0.15, 0.55, 0.82],
    fibers: 5,
    bulgeMax: 1.15,
    strainScale: 0.2,
    femaleScale: 0.76,
  },
  {
    base: "biceps",
    group: "arm",
    origin: ["upperarm.{S}", 0.16, [2.2, 0.4, -0.2]],
    belly: ["upperarm2.{S}", 0.15, [3.2, 0.8, 0.2]],
    insert: ["forearm.{S}", 0.2, [1.6, 0.5, -0.3]],
    size: [2.7, 1.9],
    outward: [0.92, 0.35, 0.15],
    fibers: 4,
    bulgeMax: 1.35,
    strainScale: 0.22,
    femaleScale: 0.74,
  },
  {
    base: "triceps",
    group: "arm",
    origin: ["upperarm.{S}", 0.2, [-2.0, -0.5, 0.0]],
    belly: ["upperarm2.{S}", 0.12, [-2.8, -0.8, 0.2]],
    insert: ["forearm.{S}", 0.06, [-1.6, -0.5, 0.0]],
    size: [2.7, 1.9],
    outward: [-0.92, -0.35, 0.15],
    fibers: 4,
    bulgeMax: 1.1,
    strainScale: 0.22,
    femaleScale: 0.75,
  },
  {
    base: "forearmFlexor",
    group: "arm",
    origin: ["forearm.{S}", 0.12, [1.6, 0.5, -0.4]],
    belly: ["forearm.{S}", 0.65, [1.8, 0.6, -0.2]],
    insert: ["hand.{S}", 0.05, [0.8, 0.2, 0.0]],
    size: [2.0, 1.4],
    outward: [0.85, 0.4, -0.2],
    fibers: 3,
    bulgeMax: 0.7,
    strainScale: 0.18,
    femaleScale: 0.75,
  },

  // ---- Pelvis & Legs ----
  {
    base: "gluteus",
    group: "leg",
    origin: ["hips", 0.62, [-4.0, -1.2, 3.6]],
    belly: ["pelvis.{S}", 0.76, [-5.8, -4.4, 1.4]],
    insert: ["thigh.{S}", 0.28, [-3.0, -1.8, 2.0]],
    size: [5.4, 2.7],
    outward: [-0.9, -0.1, 0.42],
    fibers: 5,
    bulgeMax: 1.25,
    strainScale: 0.22,
    femaleScale: 0.96,
  },
  {
    base: "quadriceps",
    group: "leg",
    origin: ["thigh.{S}", 0.08, [4.6, 0.5, 0.2]],
    belly: ["thigh2.{S}", 0.12, [5.6, 0.0, 0.6]],
    insert: ["shin.{S}", 0.06, [3.4, 0.0, 0.0]],
    size: [4.8, 2.8],
    outward: [0.96, 0.05, 0.25],
    fibers: 5,
    bulgeMax: 1.45,
    strainScale: 0.2,
    femaleScale: 0.9,
  },
  {
    base: "hamstring",
    group: "leg",
    origin: ["hips", 0.12, [-4.0, -2.0, 5.8]],
    belly: ["thigh2.{S}", 0.12, [-4.8, 0.0, -0.5]],
    insert: ["shin.{S}", 0.12, [-2.8, 0.0, -0.3]],
    size: [3.8, 2.3],
    outward: [-0.98, 0, -0.1],
    fibers: 4,
    bulgeMax: 1.15,
    strainScale: 0.22,
    femaleScale: 0.9,
  },
  {
    base: "adductor",
    group: "leg",
    origin: ["hips", 0.1, [1.5, -1.5, 4.2]],
    belly: ["thigh.{S}", 0.55, [1.2, 0.0, -3.6]],
    insert: ["thigh2.{S}", 0.65, [0.5, 0.0, -2.6]],
    size: [3.1, 1.9],
    outward: [0.35, 0, -0.94],
    fibers: 3,
    bulgeMax: 0.85,
    strainScale: 0.2,
    femaleScale: 0.9,
  },
  {
    base: "gastrocnemius",
    group: "leg",
    origin: ["thigh2.{S}", 0.88, [-2.8, 0.0, 0.0]],
    belly: ["shin.{S}", 0.34, [-4.3, 0.5, 0.2]],
    insert: ["foot.{S}", 0.04, [-1.9, -1.5, 0.0]],
    size: [3.6, 2.3],
    outward: [-0.98, 0.05, 0.1],
    fibers: 4,
    bulgeMax: 1.3,
    strainScale: 0.18,
    femaleScale: 0.86,
  },
  {
    base: "tibialis",
    group: "leg",
    origin: ["shin.{S}", 0.18, [2.6, 0.0, 1.2]],
    belly: ["shin.{S}", 0.72, [2.8, 0.0, 1.0]],
    insert: ["foot.{S}", 0.35, [1.4, 0.8, -0.5]],
    size: [1.8, 1.3],
    outward: [0.85, 0, 0.52],
    fibers: 3,
    bulgeMax: 0.65,
    strainScale: 0.18,
    femaleScale: 0.84,
  },
];

export interface MuscleState {
  readonly name: string;
  readonly base: string;
  readonly side: Side;
  readonly group: MuscleGroup;
  readonly spec: MuscleSpec;
  readonly originBone: number;
  readonly bellyBone: number;
  readonly insertBone: number;
  /** rest anchor positions in body cm (adapted to current male/female proportions) */
  readonly originRest: THREE.Vector3;
  readonly bellyRest: THREE.Vector3;
  readonly insertRest: THREE.Vector3;
  /** rest outward normal (body cm) */
  readonly outwardRest: THREE.Vector3;
  /** rest length (origin → belly → insert) in cm */
  restLength: number;
  /** instantaneous posed length in cm */
  length: number;
  /** current smoothed muscle activation (0..1) */
  activation: number;
  /** raw target activation before twitch filter */
  targetActivation: number;
  /** sparse body vertices influenced by this muscle's surface bulge */
  verts: { idx: number; w: number }[];
}

const TAU_ATTACK = 0.045;
const TAU_DECAY = 0.13;
const FIBER_STEPS = 12;
const RING_STEPS = 14;

/** map muscle activation (0..1) to CHIMERA vector palette: mid blue → bright blue → amber → red */
function muscleColor(act: number): string {
  if (act < 0.22) return mix(V.blueMid, V.blue, act / 0.22);
  if (act < 0.52) return mix(V.blue, V.amber, (act - 0.22) / 0.3);
  return mix(V.amber, V.red, Math.min(1, (act - 0.52) / 0.48));
}

export class MuscleSystem {
  readonly group = new THREE.Group();
  readonly beams = new Beams();
  readonly muscles: MuscleState[] = [];
  readonly byName = new Map<string, MuscleState>();
  readonly look: MuscleLook = { ...MUSCLE_LOOK_DEFAULT };

  private readonly poly = new Poly();
  private readonly p0 = new THREE.Vector3();
  private readonly pm = new THREE.Vector3();
  private readonly p1 = new THREE.Vector3();
  private readonly pMid = new THREE.Vector3();
  private readonly vT = new THREE.Vector3();
  private readonly vN = new THREE.Vector3();
  private readonly vB = new THREE.Vector3();
  private readonly vPt = new THREE.Vector3();
  private readonly vTmp = new THREE.Vector3();
  private readonly vBoneDir = new THREE.Vector3();

  constructor(readonly rig: Rig) {
    this.group.add(this.beams.group);
    // Keep internal muscle beams rendering after RigView's dark fill in CHIMERA's max-blend pass
    for (const c of this.beams.group.children) c.renderOrder = -1;
    for (const spec of SPECS) {
      for (const side of ["R", "L"] as const) {
        const sub = (n: string) => n.replace("{S}", side);
        const name = `${spec.base}.${side}`;
        const m: MuscleState = {
          name,
          base: spec.base,
          side,
          group: spec.group,
          spec,
          originBone: rig.bone(sub(spec.origin[0])),
          bellyBone: rig.bone(sub(spec.belly[0])),
          insertBone: rig.bone(sub(spec.insert[0])),
          originRest: new THREE.Vector3(),
          bellyRest: new THREE.Vector3(),
          insertRest: new THREE.Vector3(),
          outwardRest: new THREE.Vector3(spec.outward[0], spec.outward[1], (side === "L" ? -1 : 1) * spec.outward[2]).normalize(),
          restLength: 1,
          length: 1,
          activation: 0,
          targetActivation: 0,
          verts: [],
        };
        this.muscles.push(m);
        this.byName.set(name, m);
      }
    }
    this.fitAnatomy();
  }

  /**
   * Recompute rest anchor positions and sparse skin-bulge influences from the rig's current
   * anatomy (call whenever `female` changes, or after `rig.setAnatomy`). A muscle's rest length
   * is its length standing at anatomical zero, so the rig is posed there and left there.
   */
  fitAnatomy(): void {
    const rig = this.rig;
    rig.pose(neutral());
    const f = rig.anatomy.female;
    const scaleSex = 1 - 0.14 * f;

    const anchorOn = (boneIdx: number, frac: number, off: [number, number, number], side: Side, out: THREE.Vector3) => {
      const b = rig.bones[boneIdx];
      const hx = b.head[0], hy = b.head[1], hz = b.head[2];
      const tx = b.tail[0], ty = b.tail[1], tz = b.tail[2];
      const sgn = side === "L" ? -1 : 1;
      // Torso bones shift forward on female (spine x shifts ~+3 cm); scale offsets cleanly by sex
      const sx = scaleSex;
      out.set(
        hx + (tx - hx) * frac + off[0] * sx,
        hy + (ty - hy) * frac + off[1] * sx,
        hz + (tz - hz) * frac + sgn * off[2] * sx,
      );
      return out;
    };

    const seg = new THREE.Line3();
    const v = new THREE.Vector3(), q = new THREE.Vector3();

    for (const m of this.muscles) {
      const s = m.spec;
      anchorOn(m.originBone, s.origin[1], s.origin[2], m.side, m.originRest);
      anchorOn(m.bellyBone, s.belly[1], s.belly[2], m.side, m.bellyRest);
      anchorOn(m.insertBone, s.insert[1], s.insert[2], m.side, m.insertRest);

      // On female bodies, pull pectoralis slightly deeper behind breast tissue so breasts sit naturally over it
      if (m.base === "pectoralis" && f > 0) {
        m.bellyRest.x -= 2.0 * f;
        m.originRest.x -= 1.5 * f;
      }
      // On female bodies, widen gluteus/oblique slightly to match the wider female pelvis
      if ((m.base === "gluteus" || m.base === "oblique") && f > 0) {
        const sgn = m.side === "L" ? -1 : 1;
        m.bellyRest.z += sgn * 1.1 * f;
      }

      // its rest length (origin → belly → insert) standing, not in the mesh's A pose: measured
      // from that, a body standing still read its pectorals a third contracted, and lit them
      this.posedPoint(m.originBone, m.originRest, this.p0);
      this.posedPoint(m.bellyBone, m.bellyRest, this.pm);
      this.posedPoint(m.insertBone, m.insertRest, this.p1);
      m.restLength = this.p0.distanceTo(this.pm) + this.pm.distanceTo(this.p1);
      m.length = m.restLength;

      // Precompute sparse body vertices influenced by this muscle's belly for dynamic skin bulging
      const reach = Math.max(4.5, s.size[0] * 1.75);
      const r2Max = reach * reach;
      const verts: { idx: number; w: number }[] = [];
      const b0 = m.originRest.clone().lerp(m.bellyRest, 0.45);
      const b1 = m.insertRest.clone().lerp(m.bellyRest, 0.45);
      seg.set(b0, b1);
      const sgn = m.side === "L" ? -1 : 1;

      for (let i = 0; i < rig.bodyVertices; i++) {
        const px = rig.rest[3 * i], py = rig.rest[3 * i + 1], pz = rig.rest[3 * i + 2];
        // stay on the muscle's own side (prevent left/right cross-bleed across midline)
        if (sgn * pz < -0.5) continue;
        v.set(px, py, pz);
        seg.closestPointToPoint(v, true, q);
        const d2 = v.distanceToSquared(q);
        if (d2 >= r2Max) continue;
        // weight fades smoothly with distance and favors vertices on the outward side of the muscle
        const u = 1 - Math.sqrt(d2) / reach;
        const dirDot = v.sub(q).normalize().dot(m.outwardRest);
        if (dirDot < -0.15) continue;
        const w = u * u * (3 - 2 * u) * Math.min(1, (dirDot + 0.15) / 0.75);
        if (w > 0.04) verts.push({ idx: i, w });
      }
      m.verts = verts;
    }
  }

  /** transform a rest-space point attached to bone `bi` into posed world space */
  private posedPoint(bi: number, restPt: THREE.Vector3, out: THREE.Vector3): THREE.Vector3 {
    const rig = this.rig;
    return out.copy(restPt).sub(rig.H[bi]).applyQuaternion(rig.S[bi]).add(rig.P[bi]);
  }

  /**
   * Update muscle lengths and twitch-filtered activation (call each frame after `rig.pose(...)`).
   * `loads` optionally provides dynamic EMG-like weight-bearing excitation (e.g. from `walk.ts`).
   */
  update(dt: number, loads?: MuscleLoads): void {
    const clampedDt = Math.max(0, Math.min(0.1, dt));
    for (const m of this.muscles) {
      this.posedPoint(m.originBone, m.originRest, this.p0);
      this.posedPoint(m.bellyBone, m.bellyRest, this.pm);
      this.posedPoint(m.insertBone, m.insertRest, this.p1);
      m.length = this.p0.distanceTo(this.pm) + this.pm.distanceTo(this.p1);

      // Kinematic shortening strain: positive when origin and insertion move closer
      const strain = Math.max(0, (m.restLength - m.length) / Math.max(1e-4, m.restLength));
      const kinAct = Math.min(1, strain / m.spec.strainScale);

      // Combine kinematic shortening with dynamic load/EMG excitation
      const load = loads?.[m.name] ?? 0;
      const target = Math.max(0, Math.min(1, Math.max(kinAct, load, 0.55 * kinAct + 0.65 * load)));
      m.targetActivation = target;

      if (clampedDt <= 0) {
        m.activation = target;
      } else {
        const tau = target > m.activation ? TAU_ATTACK : TAU_DECAY;
        const alpha = 1 - Math.exp(-clampedDt / tau);
        m.activation += (target - m.activation) * alpha;
      }
    }
  }

  /**
   * Apply volume-preserving muscle bulge to a skinned mesh (`n × 3`, in-place) along vertex
   * normals. Modulated by `rig.anatomy.muscle` and attenuated by subcutaneous `rig.anatomy.fat`.
   */
  deform(skinned: Float32Array, normals: Float32Array): void {
    const gain = this.look.skinBulge;
    if (gain <= 1e-4) return;
    const { muscle, fat } = this.rig.anatomy;
    // Higher muscularity increases bulge; thicker subcutaneous fat layer softens visible muscle striations
    const bodyFactor = (0.28 + 0.92 * muscle) * (1 - 0.45 * Math.min(1, fat)) * gain;
    if (bodyFactor <= 1e-4) return;

    for (const m of this.muscles) {
      if (m.activation < 0.015) continue;
      const amp = m.activation * m.spec.bulgeMax * bodyFactor;
      if (amp < 1e-3) continue;
      for (let k = 0; k < m.verts.length; k++) {
        const { idx, w } = m.verts[k];
        const o = 3 * idx;
        const d = w * amp;
        skinned[o] += normals[o] * d;
        skinned[o + 1] += normals[o + 1] * d;
        skinned[o + 2] += normals[o + 2] * d;
      }
    }
  }

  /**
   * Render all 30 striated vector muscles (and optional soft-tissue contour rings) in CHIMERA's
   * glowing `Beams` style. Call each frame after `update(dt, loads)` and `tissue?.step(dt)`.
   */
  draw(tissue?: SoftTissue | null, camera?: THREE.Camera | null): void {
    const g = this.look.gain;
    this.beams.begin();
    if (g <= 0.01) {
      this.beams.end();
      return;
    }
    this.beams.gain = g;
    const rig = this.rig;
    const { female, muscle } = rig.anatomy;
    const muscleScale = 0.72 + 0.52 * muscle;

    for (const m of this.muscles) {
      const s = m.spec;
      const act = m.activation;
      this.posedPoint(m.originBone, m.originRest, this.p0);
      this.posedPoint(m.bellyBone, m.bellyRest, this.pm);
      this.posedPoint(m.insertBone, m.insertRest, this.p1);

      // Tangent along origin → insertion, outward normal rotated by belly bone, binormal lateral
      this.vT.copy(this.p1).sub(this.p0);
      const chordLen = this.vT.length() || 1;
      this.vT.divideScalar(chordLen);
      this.vN.copy(m.outwardRest).applyQuaternion(rig.S[m.bellyBone]).normalize();
      // Orthogonalize N against T
      this.vN.addScaledVector(this.vT, -this.vN.dot(this.vT));
      if (this.vN.lengthSq() < 1e-4) this.vN.set(1, 0, 0);
      else this.vN.normalize();
      this.vB.crossVectors(this.vT, this.vN).normalize();

      // Volume-preserving cross-section swell: when muscle shortens or fires, belly thickens radially
      const sexFactor = 1 + (s.femaleScale - 1) * female;
      const swell = 1 + 0.35 * act;
      const radW = s.size[0] * sexFactor * muscleScale * swell * 0.55;
      const radD = s.size[1] * sexFactor * muscleScale * swell * 0.55;

      // Quadratic Bezier control point (lifts mid-belly slightly outward as muscle contracts)
      this.pMid.copy(this.pm).addScaledVector(this.vN, 0.85 * act * sexFactor);

      const col = muscleColor(act);
      const alpha = 0.44 + 0.56 * act;
      const width = act > 0.55 ? 1.6 : act > 0.25 ? 1.3 : 1.05;
      const hot = act > 0.52;

      // Longitudinal striated fibers (fusiform spindle profile from tendon to tendon)
      const nf = s.fibers;
      for (let fi = 0; fi < nf; fi++) {
        const lane = nf === 1 ? 0 : (fi / (nf - 1)) * 2 - 1; // -1..+1 across belly width
        const outerArc = Math.sqrt(Math.max(0, 1 - 0.72 * lane * lane));
        this.poly.reset();
        for (let k = 0; k <= FIBER_STEPS; k++) {
          const u = k / FIBER_STEPS;
          const om = 1 - u;
          // Quadratic Bezier along centerline (P0 → 2*pMid - 0.5*(P0+P1) → P1 so curve passes through pMid at u=0.5)
          const cx = om * om * this.p0.x + 2 * om * u * (2 * this.pMid.x - 0.5 * (this.p0.x + this.p1.x)) + u * u * this.p1.x;
          const cy = om * om * this.p0.y + 2 * om * u * (2 * this.pMid.y - 0.5 * (this.p0.y + this.p1.y)) + u * u * this.p1.y;
          const cz = om * om * this.p0.z + 2 * om * u * (2 * this.pMid.z - 0.5 * (this.p0.z + this.p1.z)) + u * u * this.p1.z;
          const env = Math.pow(Math.sin(Math.PI * u), 0.85);
          const bx = lane * radW * env;
          const nx = outerArc * radD * env * 0.65;
          this.poly.p(
            cx + this.vB.x * bx + this.vN.x * nx,
            cy + this.vB.y * bx + this.vN.y * nx,
            cz + this.vB.z * bx + this.vN.z * nx,
          );
        }
        const isCenter = Math.abs(lane) < 0.35;
        this.beams.poly(this.poly, col, isCenter ? width : width * 0.8, { alpha: isCenter ? alpha : alpha * 0.78, hot: hot && isCenter });
      }

      // Cross-sectional anatomical belly rings (echoing CHIMERA's ribbed vocal tract & tongue)
      for (const u of [0.32, 0.5, 0.68]) {
        const om = 1 - u;
        const cx = om * om * this.p0.x + 2 * om * u * (2 * this.pMid.x - 0.5 * (this.p0.x + this.p1.x)) + u * u * this.p1.x;
        const cy = om * om * this.p0.y + 2 * om * u * (2 * this.pMid.y - 0.5 * (this.p0.y + this.p1.y)) + u * u * this.p1.y;
        const cz = om * om * this.p0.z + 2 * om * u * (2 * this.pMid.z - 0.5 * (this.p0.z + this.p1.z)) + u * u * this.p1.z;
        const env = Math.pow(Math.sin(Math.PI * u), 0.85);
        const rw = radW * env, rd = radD * env * 0.65;
        this.poly.reset();
        for (let k = 0; k < RING_STEPS; k++) {
          const th = (2 * Math.PI * k) / RING_STEPS;
          const cs = Math.cos(th), sn = Math.sin(th);
          this.poly.p(
            cx + this.vB.x * rw * cs + this.vN.x * rd * sn,
            cy + this.vB.y * rw * cs + this.vN.y * rd * sn,
            cz + this.vB.z * rw * cs + this.vN.z * rd * sn,
          );
        }
        this.beams.poly(this.poly, col, 0.6, { alpha: (0.24 + 0.42 * act) * (u === 0.5 ? 1 : 0.75), close: true });
      }
    }

    // Optional soft-tissue contour rings (CHIMERA V.pink) showing breast & glute mass oscillators
    if (this.look.tissueRings && tissue && tissue.settings.enabled) {
      for (const r of tissue.regions) {
        if (r.compliance < 0.22) continue;
        const isBreast = r.def.name === "breastL" || r.def.name === "breastR";
        const isButt = r.def.name === "buttL" || r.def.name === "buttR";
        if (!isBreast && !isButt) continue;
        const bi = r.boneIndex;
        const speed = r.vel.length();
        const energy = Math.min(1, r.magnitude / 2.2 + speed / 95);
        const col = mix(V.blueMid, V.pink, 0.35 + 0.65 * energy);
        const alpha = (0.22 + 0.58 * energy) * Math.min(1, r.compliance);
        const rad = isBreast
          ? r.def.radius * (0.68 + 0.32 * rig.anatomy.breast + 0.12 * rig.anatomy.fat)
          : r.def.radius * (0.58 + 0.22 * rig.anatomy.butt + 0.14 * rig.anatomy.fat);
        // Basis in parent bone frame: outward x (forward for breast, back for butt), y up, z lateral
        this.vBoneDir.set(isBreast ? 1 : -1, 0, 0).applyQuaternion(rig.S[bi]).normalize();
        this.vN.set(0, 1, 0).applyQuaternion(rig.S[bi]).normalize();
        this.vB.set(0, 0, 1).applyQuaternion(rig.S[bi]).normalize();
        this.vPt.copy(r.target).add(r.dispWorld);

        // Dim soft-tissue rings softly when seen through the far side of the body (as head.ts does)
        let facingGain = 1;
        if (camera) {
          camera.getWorldPosition(this.vTmp);
          const d = this.vTmp.sub(this.vPt).normalize().dot(this.vBoneDir);
          const t = Math.max(0, Math.min(1, (d + 0.3) / 0.6));
          facingGain = 0.2 + 0.8 * t * t * (3 - 2 * t);
        }
        const ringAlpha = alpha * facingGain;

        // Two concentric soft-tissue rings + displacement vector tick
        for (const scale of [1.0, 0.58]) {
          this.poly.reset();
          const rr = rad * scale;
          const fwd = isBreast ? (1 - scale) * 2.2 : (1 - scale) * 1.4;
          for (let k = 0; k < 18; k++) {
            const th = (2 * Math.PI * k) / 18;
            const cs = Math.cos(th), sn = Math.sin(th);
            this.poly.p(
              this.vPt.x + this.vBoneDir.x * fwd + this.vB.x * rr * cs + this.vN.x * rr * 0.85 * sn,
              this.vPt.y + this.vBoneDir.y * fwd + this.vB.y * rr * cs + this.vN.y * rr * 0.85 * sn,
              this.vPt.z + this.vBoneDir.z * fwd + this.vB.z * rr * cs + this.vN.z * rr * 0.85 * sn,
            );
          }
          this.beams.poly(this.poly, col, scale === 1 ? 0.85 : 0.6, { alpha: scale === 1 ? ringAlpha : ringAlpha * 0.85, hot: energy > 0.55 && scale === 1 && facingGain > 0.6, close: true });
        }
        // Spring tether from rigid bone anchor to live soft-tissue mass
        if (r.magnitude > 0.15) {
          this.beams.seg(r.target.x, r.target.y, r.target.z, this.vPt.x, this.vPt.y, this.vPt.z, V.pink, 1.05, Math.min(0.95, (0.35 + energy) * facingGain));
        }
      }
    }

    this.beams.end();
  }

  resize(w: number, h: number): void {
    this.beams.setResolution(w, h);
  }

  dispose(): void {
    this.group.remove(this.beams.group);
    this.beams.dispose();
  }
}
