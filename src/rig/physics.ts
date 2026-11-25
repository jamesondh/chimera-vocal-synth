/**
 * Soft-tissue secondary motion (fat & jiggle physics) for the body rig: 3-DOF damped spring-mass
 * oscillators on the breasts (`breastL`, `breastR`), buttocks (`buttL`, `buttR`) and abdominal
 * adipose (`belly`).
 *
 * Each region tracks its parent bone's motion in world space (capturing vertical gait bounce,
 * lateral hip sway, pelvic list, thoracic counter-twist and abrupt stops). Displacements are
 * constrained in the parent bone's frame so tissue cannot sink into the ribcage or pelvis, and
 * applied to the skinned mesh with smooth falloff and subtle volume-preserving squash-and-stretch.
 *
 * At rest a region sits where the mesh has it: MakeHuman's shapes were drawn hanging under
 * gravity already, so the spring adds no sag of its own (it once did, 1.6 cm on a full breast,
 * and the whole breast slid down the chest into a ledge).
 *
 * A region's spring is damped against the body it hangs on, and its target is taken along the
 * path it moved through the frame, so a body that travels (a clip that walks across the scene)
 * carries its tissue with it (damped against the world, the tissue trailed 2.5 cm at a walk).
 *
 * A region doesn't move as a block: its skin follows the region's centre late, the later the
 * further it is from where the tissue is held (up to `WAVE` s at its middle), so a jolt runs
 * out through it as a wave. Moved as one piece, a buttock at full jiggle slid off the pelvis in
 * a lump with a corner at the hip; it is also held to half its sideways travel, and to 3.5 cm.
 * A breast is held to 3.5 cm too, and rises less than half as far as it drops.
 *
 * Units: body cm, seconds. Four fixed substeps a frame keep it from blowing up at any frame
 * rate; the motion still differs a little with it (a walk's breast peak: 3.7 cm at 24 frames a
 * second, 3.9 at 240), and under 20 a second it runs slow (`MAX_DT`).
 */
import * as THREE from "three";
import type { Rig, TissueDef } from "./rig.ts";

export interface SoftTissueSettings {
  /** turn secondary motion on or off */
  enabled: boolean;
  /** overall jiggle amplitude / responsiveness (1: natural; 0: rigid; up to 2: lively) */
  jiggle: number;
}
export const SOFT_TISSUE_SETTINGS: Readonly<SoftTissueSettings> = { enabled: true, jiggle: 1.0 };

export interface TissueRegionState {
  readonly def: TissueDef;
  readonly boneIndex: number;
  /** the body vertices it moves: each one's index, weight and smoothed weight */
  readonly verts: { idx: number; w: number; sw: number }[];
  /** current oscillator centre in world cm */
  readonly pos: THREE.Vector3;
  /** rigid parent-bone target centre in world cm, and where it was at the last step */
  readonly target: THREE.Vector3;
  readonly prevTarget: THREE.Vector3;
  /** current velocity in world cm/s */
  readonly vel: THREE.Vector3;
  /** clamped displacement (world cm) applied to vertices this frame */
  readonly dispWorld: THREE.Vector3;
  /** clamped displacement in parent-bone local frame (cm) */
  readonly dispLocal: THREE.Vector3;
  /** current displacement magnitude (cm) */
  magnitude: number;
  /** effective tissue softness/compliance factor (0..1+) */
  compliance: number;
  /** previous thigh flexion (for gluteal heel-strike / femur push coupling) */
  prevThighX: number;
  /** where its displacement has been lately: (time, x, y, z) in world cm, a ring `TRAIL` long, and where the ring is at */
  readonly trail: Float32Array;
  trailAt: number;
}

const SUBSTEPS = 4;
const MAX_DT = 0.05;
const TELEPORT_CM = 35;
/** how late the middle of a region follows its centre (s), and how many steps back it can look */
const WAVE = 0.055;
const TRAIL = 24;
const BANDS = 4;

/** smooth C1 saturation: linear near 0, asymptotically approaches `limit` */
const softClamp = (x: number, limit: number) => (limit > 1e-6 ? limit * Math.tanh(x / limit) : 0);

export class SoftTissue {
  readonly settings: SoftTissueSettings = { ...SOFT_TISSUE_SETTINGS };
  readonly regions: TissueRegionState[] = [];
  private readonly byName = new Map<string, TissueRegionState>();
  private seeded = false;
  /** the time stepped so far (s) */
  private clock = 0;
  /** a region's displacement at `BANDS` delays, for the frame being applied (xyz each) */
  private readonly late = new Float32Array(3 * BANDS);

  // reusable scratch vectors/quaternions (no per-frame allocations)
  private readonly vAnchor = new THREE.Vector3();
  private readonly vDelta = new THREE.Vector3();
  private readonly vLocal = new THREE.Vector3();
  private readonly vAcc = new THREE.Vector3();
  private readonly vGoes = new THREE.Vector3();
  private readonly vTo = new THREE.Vector3();
  private readonly vRel = new THREE.Vector3();
  private readonly vOut = new THREE.Vector3();
  private readonly qInv = new THREE.Quaternion();

  constructor(readonly rig: Rig) {
    for (const def of rig.tissueDefs) {
      const weights = rig.tissues[def.name];
      if (!weights) continue;
      const verts: { idx: number; w: number; sw: number }[] = [];
      for (let i = 0; i < rig.bodyVertices; i++) {
        const w = weights[i];
        if (w > 0.008) {
          const sw = w * w * (3 - 2 * w);
          verts.push({ idx: i, w, sw });
        }
      }
      const state: TissueRegionState = {
        def,
        boneIndex: rig.bone(def.bone),
        verts,
        pos: new THREE.Vector3(),
        target: new THREE.Vector3(),
        prevTarget: new THREE.Vector3(),
        vel: new THREE.Vector3(),
        dispWorld: new THREE.Vector3(),
        dispLocal: new THREE.Vector3(),
        magnitude: 0,
        compliance: 0,
        prevThighX: 0,
        trail: new Float32Array(4 * TRAIL),
        trailAt: 0,
      };
      this.regions.push(state);
      this.byName.set(def.name, state);
    }
  }

  region(name: string): TissueRegionState | undefined {
    return this.byName.get(name);
  }

  /** snap every region to its current rigid pose equilibrium (call after sex/preset jumps) */
  reset(): void {
    this.seeded = false;
    for (const r of this.regions) {
      r.vel.set(0, 0, 0);
      r.dispWorld.set(0, 0, 0);
      r.dispLocal.set(0, 0, 0);
      r.magnitude = 0;
      r.trail.fill(0);
      for (let k = 0; k < TRAIL; k++) r.trail[4 * k] = this.clock;
    }
  }

  /** inject a world-space velocity impulse (cm/s) into one region (for tests or interactive poke) */
  impulse(name: string, dv: THREE.Vector3): void {
    const r = this.byName.get(name);
    if (r) r.vel.add(dv);
  }

  /** biomechanics of a region from current anatomy (male/female, fat, muscle, breast, butt) */
  private paramsFor(r: TissueRegionState): {
    freq: number;
    zeta: number;
    maxDisp: number;
    maxInward: number;
    compliance: number;
    outwardX: number;
  } {
    const { female, fat, muscle, breast, butt } = this.rig.anatomy;
    const jig = Math.max(0, this.settings.jiggle);
    const name = r.def.name;

    if (name === "breastL" || name === "breastR") {
      // Breasts: works independently on both female and male! Quiet on lean male with breast=0;
      // full natural breast dynamics whenever female or breast volume is raised.
      const baseFemale = female * (0.45 + 0.3 * fat);
      const volume = baseFemale + 0.95 * breast + 0.16 * fat * (1 - female) * Math.min(1, fat);
      const compliance = Math.min(1.65, volume * (1 - 0.16 * muscle)) * jig;
      const freq = Math.max(2.0, 3.85 - 0.95 * breast - 0.45 * fat + 0.28 * muscle);
      const zeta = Math.max(0.15, 0.33 - 0.08 * breast - 0.06 * fat + 0.08 * muscle);
      const maxDisp = (0.5 + 1.3 * Math.min(1.5, volume)) * Math.min(1.45, Math.sqrt(Math.max(0.1, jig)));
      return { freq, zeta, maxDisp, maxInward: 0.75, compliance, outwardX: 1 };
    }

    if (name === "buttL" || name === "buttR") {
      // Gluteal soft tissue: independent on both sexes, scaling smoothly with butt volume and fat;
      // firmer and higher-frequency with stronger gluteal muscle tone.
      const volume = 0.28 + 0.9 * butt + 0.65 * fat + 0.18 * female;
      const tone = 1 - 0.26 * muscle;
      const compliance = Math.min(1.6, volume * tone) * jig;
      const freq = Math.max(2.4, 4.35 - 0.72 * butt - 0.68 * fat + 0.52 * muscle);
      const zeta = Math.max(0.17, 0.35 - 0.06 * fat - 0.05 * butt + 0.09 * muscle);
      const maxDisp = (0.6 + 1.5 * Math.min(1.45, volume * tone)) * Math.min(1.45, Math.sqrt(Math.max(0.1, jig)));
      return { freq, zeta, maxDisp, maxInward: 0.65, compliance, outwardX: -1 };
    }

    // Belly adipose pad: dormant when lean, soft sway & bounce when fat is raised
    const volume = Math.max(0, fat - 0.1) * (1 - 0.45 * muscle);
    const compliance = Math.min(1.4, volume * 1.15) * jig;
    const freq = Math.max(2.2, 3.5 - 0.7 * fat + 0.4 * muscle);
    const zeta = Math.max(0.2, 0.38 - 0.08 * fat + 0.1 * muscle);
    const maxDisp = (0.4 + 2.2 * volume) * Math.min(1.3, Math.sqrt(Math.max(0.1, jig)));
    return { freq, zeta, maxDisp, maxInward: 0.7, compliance, outwardX: 1 };
  }

  /** current rest anchor point of a tissue region in body space (cm) */
  private restAnchor(r: TissueRegionState, out: THREE.Vector3): THREE.Vector3 {
    const f = this.rig.anatomy.female;
    const cm = r.def.centerMale, cf = r.def.centerFemale;
    out.set(cm[0] + (cf[0] - cm[0]) * f, cm[1] + (cf[1] - cm[1]) * f, cm[2] + (cf[2] - cm[2]) * f);
    // slight outward shift with breast/butt/fat volume so the spring centre tracks the morph
    if (r.def.name === "breastL" || r.def.name === "breastR") {
      out.x += 2.2 * this.rig.anatomy.breast + 0.6 * this.rig.anatomy.fat;
      out.y -= 0.7 * this.rig.anatomy.breast;
    } else if (r.def.name === "buttL" || r.def.name === "buttR") {
      out.x -= 1.6 * this.rig.anatomy.butt + 0.8 * this.rig.anatomy.fat;
    } else if (r.def.name === "belly") {
      out.x += 1.8 * this.rig.anatomy.fat;
    }
    return out;
  }

  /** advance soft-tissue oscillators by `dt` seconds (call after `rig.pose(...)`) */
  step(dt: number): void {
    const rig = this.rig;
    if (!this.settings.enabled || this.settings.jiggle <= 1e-4) {
      for (const r of this.regions) {
        r.dispWorld.set(0, 0, 0);
        r.dispLocal.set(0, 0, 0);
        r.magnitude = 0;
        r.compliance = 0;
      }
      this.seeded = false;
      return;
    }

    const clampedDt = Math.max(0, Math.min(MAX_DT, dt));
    this.clock += clampedDt;
    const subDt = clampedDt / SUBSTEPS;

    for (const r of this.regions) {
      const bi = r.boneIndex;
      const p = this.paramsFor(r);
      r.compliance = p.compliance;

      // Rigid target centre in world space: S[bi] * (anchor - H[bi]) + P[bi]
      this.restAnchor(r, this.vAnchor);
      r.target.copy(this.vAnchor).sub(rig.H[bi]).applyQuaternion(rig.S[bi]).add(rig.P[bi]);

      const omega = 2 * Math.PI * p.freq;
      const kSpring = omega * omega;
      const cDamp = 2 * p.zeta * omega;

      // Seed cleanly on first frame or after a teleport (pose switch)
      if (!this.seeded || r.pos.distanceTo(r.target) > TELEPORT_CM) {
        r.pos.copy(r.target);
        r.prevTarget.copy(r.target);
        r.vel.set(0, 0, 0);
        if (r.def.name === "buttL" || r.def.name === "buttR") {
          const th = rig.bone(r.def.side < 0 ? "thigh.L" : "thigh.R");
          r.prevThighX = rig.P[rig.bone(r.def.side < 0 ? "shin.L" : "shin.R")].x - rig.P[th].x;
        }
      }

      // Gluteal femur coupling: when the ipsilateral thigh kicks back/forward, its acceleration
      // gently excites that glute cheek independently of the contralateral side.
      let gluteKick = 0;
      if ((r.def.name === "buttL" || r.def.name === "buttR") && clampedDt > 1e-4) {
        const side = r.def.side < 0 ? "L" : "R";
        const th = rig.bone(`thigh.${side}`), kn = rig.bone(`shin.${side}`);
        const thighX = rig.P[kn].x - rig.P[th].x;
        const dThigh = (thighX - r.prevThighX) / clampedDt;
        r.prevThighX = thighX;
        gluteKick = Math.max(-180, Math.min(180, dThigh * 0.45)) * p.compliance;
      }

      // Integrate 4 symplectic substeps (unconditionally stable for omega * subDt < 2)
      if (subDt > 0 && p.compliance > 1e-4) {
        // how fast the target goes: the damping is on the tissue's speed against it
        this.vGoes.copy(r.target).sub(r.prevTarget).divideScalar(clampedDt);
        for (let s = 0; s < SUBSTEPS; s++) {
          this.vTo.copy(r.prevTarget).lerp(r.target, s / SUBSTEPS);
          this.vAcc.copy(this.vTo).sub(r.pos).multiplyScalar(kSpring);
          this.vAcc.addScaledVector(r.vel, -cDamp).addScaledVector(this.vGoes, cDamp);
          if (gluteKick) {
            this.vAcc.y += 0.35 * Math.abs(gluteKick);
            this.vAcc.x -= 0.25 * gluteKick;
          }
          r.vel.addScaledVector(this.vAcc, subDt);
          r.pos.addScaledVector(r.vel, subDt);
        }
      } else if (p.compliance <= 1e-4) {
        r.pos.copy(r.target);
        r.vel.set(0, 0, 0);
      }

      r.prevTarget.copy(r.target);

      // Convert raw world displacement (pos - target) into parent-bone local frame for anatomical constraints
      this.vDelta.copy(r.pos).sub(r.target);
      this.qInv.copy(rig.S[bi]).invert();
      this.vLocal.copy(this.vDelta).applyQuaternion(this.qInv);

      // Scale response by compliance (keeps flat male chest quiet while female/curvy/fat moves freely)
      const scale = Math.min(1.35, p.compliance);
      this.vLocal.multiplyScalar(scale);
      // a buttock moves up and down and fore and aft; sideways it is held by the hip
      if (p.outwardX < 0) this.vLocal.z *= 0.5;
      // a breast hangs from the chest: it drops and swings, and rises under half as far (free to go
      // up as far as down, 6 cm each way on a jog, it was smeared flat up the chest at each stride)
      else if (r.def.side !== 0 && this.vLocal.y > 0) this.vLocal.y = softClamp(this.vLocal.y, 0.4 * p.maxDisp);

      // Anatomical inward collision limit (cannot collapse deep into ribcage or pelvis)
      const outward = this.vLocal.x * p.outwardX;
      if (outward < -p.maxInward) {
        const clampedOut = -p.maxInward - 0.15 * Math.tanh((-outward - p.maxInward) / 0.5);
        this.vLocal.x = clampedOut * p.outwardX;
      }

      // Smooth radial saturation at maxDisp so rapid motions never invert quads
      const mag = this.vLocal.length();
      if (mag > 1e-6) {
        const limited = softClamp(mag, p.maxDisp);
        this.vLocal.multiplyScalar(limited / mag);
      }

      r.dispLocal.copy(this.vLocal);
      r.dispWorld.copy(this.vLocal).applyQuaternion(rig.S[bi]);
      r.magnitude = r.dispWorld.length();
      if (clampedDt > 0) {
        r.trailAt = (r.trailAt + 1) % TRAIL;
        r.trail.set([this.clock, r.dispWorld.x, r.dispWorld.y, r.dispWorld.z], 4 * r.trailAt);
      }
    }

    this.seeded = true;
  }

  /**
   * Apply current soft-tissue displacements to a skinned vertex buffer (`n × 3`, in-place),
   * with smooth boundary falloff and volume-preserving squash-and-stretch.
   */
  apply(skinned: Float32Array): void {
    if (!this.settings.enabled || this.settings.jiggle <= 1e-4) return;
    const rig = this.rig;
    for (const r of this.regions) {
      if (r.magnitude < 1e-4 || r.compliance < 1e-4) continue;
      const bi = r.boneIndex;
      const S = rig.S[bi];
      // where the region's centre was at each of a few delays back: the skin takes the one its place calls for
      const late = this.late, T = r.trail;
      for (let b = 0; b < BANDS; b++) {
        const when = this.clock - (WAVE * b) / (BANDS - 1);
        let k = r.trailAt, older = (k + TRAIL - 1) % TRAIL, steps = 0;
        while (T[4 * older] > when && T[4 * older] < T[4 * k] && ++steps < TRAIL - 1) {
          k = older;
          older = (k + TRAIL - 1) % TRAIL;
        }
        const t1 = T[4 * k], t0 = T[4 * older], u = t1 > t0 && when < t1 ? Math.max(0, (when - t0) / (t1 - t0)) : 1;
        for (let c = 0; c < 3; c++) late[3 * b + c] = T[4 * older + 1 + c] + (T[4 * k + 1 + c] - T[4 * older + 1 + c]) * u;
      }
      const outSign = r.def.name === "buttL" || r.def.name === "buttR" ? -1 : 1;
      // Inward compression drives subtle radial bulge (Poisson-like volume preservation)
      const compress = Math.max(0, -(r.dispLocal.x * outSign));
      const bulge = 0.28 * compress / Math.max(3, r.def.radius);
      // Downward drop adds subtle forward/outward lower-pole pendulum swing
      const pend = -r.dispLocal.y * 0.025 * outSign;
      const cx = r.target.x, cy = r.target.y, cz = r.target.z;

      for (let k = 0; k < r.verts.length; k++) {
        const { idx, sw } = r.verts[k];
        const o = 3 * idx;
        // the more a vertex is of the region, the later it follows
        const at = sw * (BANDS - 1), b = Math.min(BANDS - 2, Math.floor(at)), f = at - b;
        let ox = sw * (late[3 * b] + (late[3 * b + 3] - late[3 * b]) * f);
        let oy = sw * (late[3 * b + 1] + (late[3 * b + 4] - late[3 * b + 1]) * f);
        let oz = sw * (late[3 * b + 2] + (late[3 * b + 5] - late[3 * b + 2]) * f);
        if (bulge > 1e-4 || Math.abs(pend) > 1e-4) {
          this.vRel.set(skinned[o] - cx, skinned[o + 1] - cy, skinned[o + 2] - cz);
          this.qInv.copy(S).invert();
          this.vRel.applyQuaternion(this.qInv);
          // radial expansion in tangent plane (y, z) on inward squash + pendulum tilt on vertical bounce
          this.vOut.set(pend * Math.max(0, -this.vRel.y), bulge * this.vRel.y, bulge * this.vRel.z).applyQuaternion(S);
          ox += sw * this.vOut.x;
          oy += sw * this.vOut.y;
          oz += sw * this.vOut.z;
        }
        skinned[o] += ox;
        skinned[o + 1] += oy;
        skinned[o + 2] += oz;
      }
    }
  }
}
