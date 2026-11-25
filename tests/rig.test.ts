/**
 * The body rig (src/rig/): posing, skinning, and the CHIMERA head worn on it. On the real engine
 * (the head is fitted to a VoiceModel), without a GPU.
 */
import { beforeAll, describe, expect, test } from "bun:test";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import * as THREE from "three";
import { initEngine } from "./harness.ts";
import type { AnatomyParams, Rig as RigT } from "../src/rig/rig.ts";
import type { Pose } from "../src/rig/pose.ts";

type Mods = {
  Rig: typeof import("../src/rig/rig.ts").Rig;
  vertexNormals: typeof import("../src/rig/rig.ts").vertexNormals;
  armClearance: typeof import("../src/rig/rig.ts").armClearance;
  motion: typeof import("../src/rig/motion.ts");
  pose: typeof import("../src/rig/pose.ts");
  head: typeof import("../src/rig/head.ts");
  view: typeof import("../src/rig/view.ts");
  physics: typeof import("../src/rig/physics.ts");
  maleAnatomy: typeof import("../src/rig/maleAnatomy.ts");
  muscles: typeof import("../src/rig/muscles.ts");
  walk: typeof import("../src/rig/walk.ts");
  VoiceModel: typeof import("../src/ui/analysis.ts").VoiceModel;
  BODIES: typeof import("../src/ui/body.ts").PRESETS;
  REST_ARTIC: typeof import("../src/ui/render/anatomy3d.ts").REST_ARTIC;
};
let M: Mods;
beforeAll(async () => {
  await initEngine();
  // the head reads the engine's tables as it loads: after the engine
  const [rig, pose, head, view, analysis, body, a3d, motion, physics, maleAnatomy, muscles, walk] = await Promise.all([
    import("../src/rig/rig.ts"),
    import("../src/rig/pose.ts"),
    import("../src/rig/head.ts"),
    import("../src/rig/view.ts"),
    import("../src/ui/analysis.ts"),
    import("../src/ui/body.ts"),
    import("../src/ui/render/anatomy3d.ts"),
    import("../src/rig/motion.ts"),
    import("../src/rig/physics.ts"),
    import("../src/rig/maleAnatomy.ts"),
    import("../src/rig/muscles.ts"),
    import("../src/rig/walk.ts"),
  ]);
  M = { Rig: rig.Rig, vertexNormals: rig.vertexNormals, armClearance: rig.armClearance, motion, pose, head, view, physics, maleAnatomy, muscles, walk, VoiceModel: analysis.VoiceModel, BODIES: body.PRESETS, REST_ARTIC: a3d.REST_ARTIC };
});

function loadRig(): RigT {
  const b = readFileSync(new URL("../src/rig/body.rig.bin", import.meta.url));
  return new M.Rig(b.buffer.slice(b.byteOffset, b.byteOffset + b.byteLength) as ArrayBuffer);
}

/** the mesh's edges that only one quad has, as vertex pairs */
function openEdges(rig: RigT): [number, number][] {
  const count = new Map<number, number>();
  for (let f = 0; f < rig.nq; f++) {
    for (let j = 0; j < 4; j++) {
      const a = rig.quads[4 * f + j], b = rig.quads[4 * f + ((j + 1) % 4)];
      const k = Math.min(a, b) * 1048576 + Math.max(a, b);
      count.set(k, (count.get(k) ?? 0) + 1);
    }
  }
  return [...count].filter(([, c]) => c === 1).map(([k]) => [Math.floor(k / 1048576), k % 1048576]);
}

const at = (P: ArrayLike<number>, i: number) => new THREE.Vector3(P[3 * i], P[3 * i + 1], P[3 * i + 2]);
const maxMove = (A: ArrayLike<number>, B: ArrayLike<number>, ids: Iterable<number>) => {
  let m = 0;
  for (const i of ids) m = Math.max(m, at(A, i).distanceTo(at(B, i)));
  return m;
};

describe("the body", () => {
  test("its only open edges are the neck's rim", () => {
    const rig = loadRig(), open = openEdges(rig);
    expect(open.length).toBe(rig.rim.length);
    const rim = new Set(rig.rim);
    expect(open.every(([a, b]) => rim.has(a) && rim.has(b))).toBe(true);
  });

  test("unposed, both skinnings give the rest mesh back", () => {
    const rig = loadRig(), out = new Float32Array(3 * rig.n);
    for (const method of ["cor", "lbs"] as const) {
      rig.skin(out, method);
      for (let i = 0; i < out.length; i++) expect(Math.abs(out[i] - rig.rest[i])).toBeLessThan(1e-3);
    }
  });

  test("at anatomical zero the arms hang straight down and the legs stand under the hips", () => {
    const rig = loadRig();
    rig.pose(M.pose.neutral());
    const down = (from: string, to: string) => rig.P[rig.bone(to)].clone().sub(rig.P[rig.bone(from)]).normalize().y;
    for (const s of ["R", "L"]) {
      expect(down(`upperarm.${s}`, `forearm.${s}`)).toBeLessThan(-0.999);
      expect(down(`forearm.${s}`, `hand.${s}`)).toBeLessThan(-0.999);
      // seen from the front the hip, knee and ankle are in one vertical line
      const hip = rig.P[rig.bone(`thigh.${s}`)], ankle = rig.P[rig.bone(`foot.${s}`)];
      expect(Math.abs(ankle.z - hip.z)).toBeLessThan(0.05);
    }
  });
});

describe("the head on the body", () => {
  /** the voices' bodies: the head is fitted to each (a child's has fewer rows under its mouth) */
  const BODIES = ["male", "female", "child"] as const;
  const wear = (body: (typeof BODIES)[number] = "male") => {
    const rig = loadRig(), model = new M.VoiceModel(M.BODIES[body], 48000), head = M.head.wearHead(rig, model);
    return { rig, model, head };
  };
  /** the lips' cut: pairs of head points that lie together at rest (each upper lip point and its lower copy) */
  const lipPairs = (rig: RigT) => {
    const pairs: [number, number][] = [];
    for (let i = rig.bodyVertices; i < rig.n; i++) for (let j = i + 1; j < rig.n; j++) if (at(rig.rest, i).distanceTo(at(rig.rest, j)) < 1e-5) pairs.push([i, j]);
    return pairs;
  };
  /** the body posed and skinned, with the head drawn and its face moved for an articulation */
  const frame = (w: ReturnType<typeof wear>, p: import("../src/rig/pose.ts").Pose, a: import("../src/wasm/types.ts").HumanArtic) => {
    const { rig, head } = w, base = new Float32Array(3 * rig.n), out = new Float32Array(3 * rig.n);
    rig.pose(p);
    rig.skin(base);
    head.update(new THREE.PerspectiveCamera(), a, null, 1 / 60);
    head.face(base, out);
    return { base, out };
  };

  test.each(BODIES)("the skin closes over the head: the only open edges are the lips' cut (%s)", (body) => {
    const { rig } = wear(body), open = openEdges(rig), pairs = lipPairs(rig);
    expect(pairs.length).toBeGreaterThan(2);
    const cut = new Set(pairs.flat());
    // the cut runs between the mouth's corners, which both lips share: each edge has at least one cut point, none a body point
    expect(open.length).toBe(2 * (pairs.length + 1));
    expect(open.every(([a, b]) => a >= rig.bodyVertices && b >= rig.bodyVertices && (cut.has(a) || cut.has(b)))).toBe(true);
  });

  /** a point's distance from a polyline (flat xyz) carried by a matrix */
  const fromLine = (p: THREE.Vector3, line: Float32Array, m: THREE.Matrix4) => {
    const a = new THREE.Vector3(), b = new THREE.Vector3(), seg = new THREE.Line3(), q = new THREE.Vector3();
    let d = Infinity;
    for (let i = 0; i + 5 < line.length; i += 3) {
      seg.set(a.fromArray(line, i).applyMatrix4(m), b.fromArray(line, i + 3).applyMatrix4(m));
      d = Math.min(d, seg.closestPointToPoint(p, true, q).distanceTo(p));
    }
    return d;
  };

  /** a point's distance from a polyline of points */
  const fromPoints = (p: THREE.Vector3, pts: THREE.Vector3[]) => {
    const seg = new THREE.Line3(), q = new THREE.Vector3();
    let d = Infinity;
    for (let i = 0; i + 1 < pts.length; i++) d = Math.min(d, seg.set(pts[i], pts[i + 1]).closestPointToPoint(p, true, q).distanceTo(p));
    return d;
  };

  test.each(BODIES)("the lips' cut lies on the lips the head draws: together at rest; apart, narrow or wide, with the head turned (%s)", (body) => {
    const w = wear(body), { rig } = w, pairs = lipPairs(rig);
    const shut = frame(w, M.pose.neutral(), M.REST_ARTIC).out;
    for (const [u, l] of pairs) expect(at(shut, u).distanceTo(at(shut, l))).toBeLessThan(1e-3);
    // the cut's corners: its ends, which both lips share
    const cut = new Set(pairs.flat()), corners = [...new Set(openEdges(rig).flat())].filter((i) => !cut.has(i));
    expect(corners.length).toBe(2);
    // each lip's side of the cut, corner to corner (at rest the cut runs across z)
    const byZ = (ids: number[]) => ids.sort((i, j) => rig.rest[3 * i + 2] - rig.rest[3 * j + 2]);
    const sides = [byZ([...corners, ...pairs.map(([u]) => u)]), byZ([...corners, ...pairs.map(([, l]) => l)])];
    const turned = M.pose.neutral();
    turned.neck.twist = 40;
    turned.neck.flex = 20;
    for (const p of [M.pose.neutral(), turned]) {
      for (const lipWidth of [0, 1]) {
        const a = { ...M.REST_ARTIC, jaw: 1.8, lipAp: 1.8, lipWidth };
        const { out } = frame(w, p, a);
        const e = w.head.head.lipEdges(w.model, a), m = w.head.mount.matrix;
        // apart (the drawn lips' gap at the middle, in body cm)
        const mid = 3 * Math.floor(e.upper.length / 6);
        expect(new THREE.Vector3().fromArray(e.upper, mid).distanceTo(new THREE.Vector3().fromArray(e.lower, mid)) / w.head.s).toBeGreaterThan(0.5);
        for (const [u, l] of pairs) {
          expect(fromLine(at(out, u), e.upper, m)).toBeLessThan(0.01);
          expect(fromLine(at(out, l), e.lower, m)).toBeLessThan(0.01);
        }
        // the cut's ends are the mouth's corners, however wide it is
        const ends = [new THREE.Vector3().fromArray(e.upper, 0), new THREE.Vector3().fromArray(e.upper, e.upper.length - 3)].map((c) => c.applyMatrix4(m));
        for (const c of corners) expect(Math.min(...ends.map((x) => x.distanceTo(at(out, c))))).toBeLessThan(0.01);
        // and the drawn lips are near the cut's edges all along (straight between the grid's
        // columns, so within a fraction of a column's width of the curve)
        [e.upper, e.lower].forEach((line, k) => {
          const edge = sides[k].map((i) => at(out, i));
          for (let i = 0; i < line.length; i += 3) expect(fromPoints(new THREE.Vector3().fromArray(line, i).applyMatrix4(m), edge)).toBeLessThan(0.4);
        });
      }
    }
  });

  test.each(BODIES)("the head's quads don't fold, the mouth wide open or pursed with the head bent and turned (%s)", (body) => {
    const w = wear(body), { rig } = w, p = M.pose.neutral();
    p.neck.flex = 40;
    p.neck.twist = 60;
    for (const a of [{ ...M.REST_ARTIC, jaw: 2.4, lipAp: 2.2, lipWidth: 1 }, { ...M.REST_ARTIC, jaw: 2.4, lipAp: 2.2, lipWidth: 0.05, lipProt: 1.4 }]) {
      const { out } = frame(w, p, a);
      for (let f = 0; f < rig.nq; f++) {
        const q = [0, 1, 2, 3].map((j) => rig.quads[4 * f + j]);
        if (q.every((i) => i < rig.bodyVertices)) continue;
        // the two triangles the view draws (a b c, a c d) face the same way
        const [A, B, C, D] = q.map((i) => at(out, i));
        const n1 = new THREE.Vector3().crossVectors(B.clone().sub(A), C.clone().sub(A)), n2 = new THREE.Vector3().crossVectors(C.clone().sub(A), D.clone().sub(A));
        if (n1.length() < 1e-6 || n2.length() < 1e-6) continue;
        expect(n1.normalize().dot(n2.normalize())).toBeGreaterThan(0);
      }
    }
  });

  test("every point's weights sum to 1 (a fitted neck once dropped a share)", () => {
    const { rig } = wear();
    for (let i = 0; i < rig.n; i++) {
      let sum = 0;
      for (let j = 0; j < rig.INF; j++) sum += rig.skinWeight[rig.INF * i + j];
      expect(Math.abs(sum - 1)).toBeLessThan(1e-5);
    }
  });

  test("the head reads its parent's transform as it is now (the features dim with the face turned away)", () => {
    const w = wear(), parent = new THREE.Group(), camera = new THREE.PerspectiveCamera();
    parent.add(w.head.mount);
    camera.position.set(400, 60, 0);
    w.rig.pose(M.pose.neutral());
    w.head.update(camera, M.REST_ARTIC, null, 0);
    const facing = w.head.head.drawing.features;
    // the body turned round under its parent since the last frame: the face is away from the camera now
    parent.rotation.y = Math.PI;
    w.head.update(camera, M.REST_ARTIC, null, 0);
    expect(facing).toBeCloseTo(w.head.look.features, 3);
    expect(w.head.head.drawing.features).toBeLessThan(0.2 * w.head.look.features);
  });

  test("the face's motion moves the face only: not the crown, not the rim (a mouth opening once dented the crown)", () => {
    const w = wear(), { rig } = w;
    const p = M.pose.neutral();
    p.neck.twist = 40;
    p.neck.flex = 20;
    const { base, out } = frame(w, p, { ...M.REST_ARTIC, jaw: 1.8, lipAp: 1.8 });
    // the crown: the head's points above its eyes
    let top = -Infinity;
    for (let i = rig.bodyVertices; i < rig.n; i++) top = Math.max(top, rig.rest[3 * i + 1]);
    const crown = [...Array(rig.n).keys()].filter((i) => i >= rig.bodyVertices && rig.rest[3 * i + 1] > top - 6);
    expect(crown.length).toBeGreaterThan(20);
    expect(maxMove(base, out, crown)).toBeLessThan(1e-4);
    expect(maxMove(base, out, rig.rim)).toBe(0);
    expect(maxMove(base, out, lipPairs(rig).map(([, l]) => l))).toBeGreaterThan(0.5);
  });
});

/**
 * The rig's own skeleton as a BVH file, as another tool would write it: bound at `bind`, then
 * posed through `frames` (half a second apart), turned `yaw`° about the vertical and measured in
 * `unit`s per cm, standing on the file's floor (y = 0) with its root's OFFSET at 0 as BVHs have;
 * without the `leave` joints (their parents end there).
 */
function toBVH(rig: RigT, bind: Pose, frames: Pose[], yaw: number, unit: number, leave: string[] = []): string {
  const G = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), (yaw * Math.PI) / 180), Gi = G.clone().invert();
  const joints = rig.bones.map((_, i) => i).filter((i) => !rig.bones[i].helper && !leave.includes(rig.bones[i].name));
  const kids = (i: number) => joints.filter((j) => rig.bones[j].parent === i);
  rig.pose(bind);
  const Pb = rig.P.map((p) => p.clone()), Wb = rig.W.map((q) => q.clone().invert()), Sb = rig.S.map((q) => q.clone());
  const floor = Math.min(...rig.P.map((p) => p.y));
  const file = (v: THREE.Vector3) => v.clone().applyQuaternion(G).multiplyScalar(unit);
  const lines = ["HIERARCHY"], order: number[] = [];
  const write = (i: number, pad: string) => {
    order.push(i);
    const par = rig.bones[i].parent, o = par < 0 ? new THREE.Vector3() : file(Pb[i].clone().sub(Pb[par]));
    lines.push(`${pad}${par < 0 ? "ROOT" : "JOINT"} ${rig.bones[i].name.replace(".", "_")}`, `${pad}{`, `${pad}  OFFSET ${o.x} ${o.y} ${o.z}`);
    lines.push(`${pad}  CHANNELS ${par < 0 ? "6 Xposition Yposition Zposition " : "3 "}Zrotation Xrotation Yrotation`);
    for (const j of kids(i)) write(j, pad + "  ");
    if (!kids(i).length) {
      const t = file(new THREE.Vector3().fromArray(rig.bones[i].tail).sub(rig.H[i]).applyQuaternion(Sb[i]));
      lines.push(`${pad}  End Site`, `${pad}  {`, `${pad}    OFFSET ${t.x} ${t.y} ${t.z}`, `${pad}  }`);
    }
    lines.push(`${pad}}`);
  };
  write(0, "");
  lines.push("MOTION", `Frames: ${frames.length}`, "Frame Time: 0.5");
  for (const f of frames) {
    rig.pose(f);
    const R = rig.W.map((w, i) => G.clone().multiply(w).multiply(Wb[i]).multiply(Gi));
    const vals: number[] = [];
    for (const i of order) {
      const par = rig.bones[i].parent;
      if (par < 0) vals.push(...file(rig.P[0].clone().sub(new THREE.Vector3(0, floor, 0))).toArray());
      const e = new THREE.Euler().setFromQuaternion(par < 0 ? R[i] : R[par].clone().invert().multiply(R[i]), "ZXY");
      vals.push(...[e.z, e.x, e.y].map((a) => (a * 180) / Math.PI));
    }
    lines.push(vals.join(" "));
  }
  return lines.join("\n");
}
const buffer = (text: string) => new TextEncoder().encode(text).buffer as ArrayBuffer;
/** how far (degrees) the hands of `b` are turned from `a`'s */
const handsApart = (a: RigT, b: RigT) => Math.max(...["hand.L", "hand.R"].map((h) => (a.S[a.bone(h)].angleTo(b.S[b.bone(h)]) * 180) / Math.PI));
/** the farthest any joint of `b` is from `a`'s, measured from each one's root (or not) */
function jointsApart(a: RigT, b: RigT, fromRoot: boolean): { cm: number; at: string } {
  let cm = 0, at = "";
  a.bones.forEach((bone, i) => {
    if (bone.helper) return;
    const d = fromRoot ? a.P[i].clone().sub(a.P[0]).distanceTo(b.P[i].clone().sub(b.P[0])) : a.P[i].distanceTo(b.P[i]);
    if (!(d <= cm)) [cm, at] = [Number.isNaN(d) ? Infinity : d, bone.name];
  });
  return { cm, at };
}

/**
 * A small skeleton as other tools make them: Mixamo's names, a T pose (palms down), y up, facing
 * +z, in metres. `change` can add to it or move it before it's used.
 */
function mixamo(change?: (b: Record<string, THREE.Bone>) => void): Record<string, THREE.Bone> {
  const b: Record<string, THREE.Bone> = {};
  const bone = (name: string, parent: string | null, x: number, y: number, z: number) => {
    const o = (b[name] = new THREE.Bone());
    o.name = name;
    o.position.set(x, y, z);
    if (parent) b[parent].add(o);
  };
  bone("Hips", null, 0, 1, 0);
  bone("Spine", "Hips", 0, 0.1, 0);
  bone("Chest", "Spine", 0, 0.2, 0);
  bone("Neck", "Chest", 0, 0.2, 0);
  bone("Head", "Neck", 0, 0.1, 0);
  bone("HeadTop_End", "Head", 0, 0.18, 0);
  for (const [s, x] of [["Left", 1], ["Right", -1]] as const) {
    bone(`${s}Shoulder`, "Chest", 0.05 * x, 0.15, 0);
    bone(`${s}Arm`, `${s}Shoulder`, 0.12 * x, 0, 0);
    bone(`${s}ForeArm`, `${s}Arm`, 0.27 * x, 0, 0);
    bone(`${s}Hand`, `${s}ForeArm`, 0.27 * x, 0, 0);
    bone(`${s}HandMiddle1`, `${s}Hand`, 0.1 * x, 0, 0);
    bone(`${s}HandPinky1`, `${s}Hand`, 0.09 * x, 0, -0.03);
    bone(`${s}UpLeg`, "Hips", 0.09 * x, -0.05, 0);
    bone(`${s}Leg`, `${s}UpLeg`, 0, -0.45, 0);
    bone(`${s}Foot`, `${s}Leg`, 0, -0.42, 0);
    bone(`${s}ToeBase`, `${s}Foot`, 0, -0.06, 0.12);
  }
  change?.(b);
  return b;
}
/** a clip turning bones (by name) about an axis, through angles (degrees), a key every 0.05 s */
function turning(turns: Record<string, { axis: [number, number, number]; degrees: number[] }>): THREE.AnimationClip {
  return new THREE.AnimationClip("turning", -1, Object.entries(turns).map(([name, { axis, degrees }]) => new THREE.QuaternionKeyframeTrack(
    `${name}.quaternion`, degrees.map((_, i) => 0.05 * i),
    degrees.flatMap((d) => new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(...axis), (d * Math.PI) / 180).toArray()),
  )));
}
/** how far (degrees) a rig bone points from the source's bone (its joint to the next one's), in the rig's frame */
function aimedApart(motion: InstanceType<Mods["motion"]["Motion"]>, bone: string, next: string, from: THREE.Object3D, to: THREE.Object3D): number {
  const r = motion.rig, p = (o: THREE.Object3D) => o.getWorldPosition(new THREE.Vector3());
  const want = p(to).sub(p(from)).applyQuaternion(motion.C).normalize();
  return (r.P[r.bone(next)].clone().sub(r.P[r.bone(bone)]).normalize().angleTo(want) * 180) / Math.PI;
}

describe("motion from other tools", () => {
  test("bones are matched by name across tools' conventions; twist bones, fingers and end sites aren't", () => {
    const cases: Record<string, string | null> = {
      // Mixamo (FBX and glTF, as three.js's loaders name them), CMU and DAZ BVHs
      "mixamorig:LeftArm": "upperarm.L", mixamorigRightForeArm: "forearm.R", mixamorig1LeftUpLeg: "thigh.L", mixamorigLeftShoulder: "clavicle.L",
      mixamorigRightToeBase: "toes.R", mixamorigHead: "head", mixamorigLeftHandIndex1: null, mixamorigLeftToe_End: null, mixamorigHeadTop_End: null,
      LeftLeg: "shin.L", RightFoot: "foot.R", LHipJoint: null, LeftFingerBase: null, lShldr: "upperarm.L", rCollar: "clavicle.R", rShin: "shin.R", lButtock: null, head: "head",
      // VRM and VRoid, Unreal, Rigify, 3ds Max Biped, and the rig's own
      leftUpperArm: "upperarm.L", rightLowerLeg: "shin.R", J_Bip_L_UpperArm: "upperarm.L", J_Bip_C_Head: "head", J_Bip_R_ToeBase: "toes.R",
      upperarm_l: "upperarm.L", calf_r: "shin.R", ball_l: "toes.L", upperarm_twist_01_l: null, "DEF-upper_arm.L": "upperarm.L", "DEF-upper_armL": "upperarm.L",
      "DEF-upper_arm.L.001": null, "Bip01 L Thigh": "thigh.L", "Bip01 R Calf": "shin.R", "Bip01 L Toe0": "toes.L",
      "forearm.R": "forearm.R", "upperarm2.R": null, "shoulderHelper.L": null, "pelvis.R": null,
    };
    for (const [name, want] of Object.entries(cases)) expect([name, M.motion.rigName(name)]).toEqual([name, want]);
  });

  test("the rig's own poses come back through a BVH, whatever its bind, facing and units", async () => {
    const { PRESETS } = M.pose, names = Object.keys(PRESETS);
    // the clip comes back round to neutral, so its last pose isn't the loop's wrap
    const frames = [...names.map((n) => PRESETS[n]), PRESETS.neutral];
    for (const [bind, yaw, unit] of [["neutral", 90, 1 / 2.54], ["T pose", -30, 0.01]] as const) {
      const src = loadRig(), dst = loadRig();
      // the source's thigh twists about the thigh: the rig's own posing twists it about the
      // vertical, a few degrees off, which bends its second half off the line
      src.settings.thighTwist = 1;
      const motion = await M.motion.Motion.load(dst, buffer(toBVH(src, PRESETS[bind], frames, yaw, unit)), "rig.bvh");
      names.forEach((name, k) => {
        src.pose(PRESETS[name]);
        motion.pose(0.5 * k);
        expect([bind, name, jointsApart(src, dst, true).cm < 0.1, handsApart(src, dst) < 0.5]).toEqual([bind, name, true, true]);
      });
    }
  });

  test("a skeleton without toes, or with a clip of one frame, still plays; seeking goes anywhere in any order", async () => {
    const { PRESETS } = M.pose, names = ["run", "lotus", "twist & look", "wrists"];
    const src = loadRig(), dst = loadRig();
    src.settings.thighTwist = 1;
    const motion = await M.motion.Motion.load(dst, buffer(toBVH(src, PRESETS.neutral, [...names.map((n) => PRESETS[n]), PRESETS.neutral], 0, 1, ["toes.L", "toes.R"])), "rig.bvh");
    expect(motion.bones["toes.L"]).toBeUndefined();
    // [time in frames, the frame it shows]: past the end (2 s) it loops
    for (const [k, shows] of [[3, 3], [0, 0], [2, 2], [1, 1], [6, 2]]) {
      src.pose(PRESETS[names[shows]]);
      motion.pose(0.5 * k);
      expect(jointsApart(src, dst, true).cm).toBeLessThan(0.1);
    }
    const ref = loadRig(), still = await M.motion.Motion.load(loadRig(), buffer(toBVH(loadRig(), PRESETS.neutral, [PRESETS.run], 0, 1)), "rig.bvh");
    expect(still.duration).toBe(0);
    still.pose(3.7);
    ref.pose(PRESETS.run);
    expect(jointsApart(ref, still.rig, true).cm).toBeLessThan(0.1);
  });

  test("standing on its floor, the source's hips put the rig's where they'd stand", async () => {
    const { PRESETS } = M.pose, frames = ["neutral", "T pose", "arms forward", "flex biceps", "neutral"].map((n) => PRESETS[n]);
    const src = loadRig(), dst = loadRig();
    src.settings.thighTwist = 1;
    const motion = await M.motion.Motion.load(dst, buffer(toBVH(src, PRESETS["T pose"], frames, 45, 1)), "rig.bvh");
    frames.forEach((f, k) => {
      src.pose(f);
      motion.pose(0.5 * k);
      expect(jointsApart(src, dst, false).cm).toBeLessThan(0.1);
    });
  });

  test("an explicit map wins; of two bones that could be one of the rig's, the one the clip moves", () => {
    const { Motion } = M.motion;
    const withControl = () => mixamo((b) => {
      // a control bone that also reads as the left upper arm
      b.upperarm_L = new THREE.Bone();
      b.upperarm_L.name = "upperarm_L";
      b.Hips.add(b.upperarm_L);
    });
    const raise = { axis: [0, 0, 1] as [number, number, number], degrees: [0, 45] };
    let b = withControl();
    expect(new Motion(loadRig(), b.Hips, turning({ LeftArm: raise })).bones["upperarm.L"]).toBe("LeftArm");
    b = withControl();
    expect(() => new Motion(loadRig(), b.Hips, turning({ LeftArm: raise, upperarm_L: raise }))).toThrow(/could be/);
    b = withControl();
    expect(new Motion(loadRig(), b.Hips, turning({ LeftArm: raise, upperarm_L: raise }), { map: { LeftArm: "upperarm.L" } }).bones["upperarm.L"]).toBe("LeftArm");
  });

  test("a pivot between the chest and a shoulder is fine; an unevenly scaled or unstanding source is refused", () => {
    const { Motion } = M.motion;
    const b = mixamo((b) => {
      const pivot = new THREE.Bone();
      pivot.name = "ShoulderPivot";
      b.Chest.add(pivot);
      pivot.add(b.LeftShoulder);
    });
    const motion = new Motion(loadRig(), b.Hips, turning({ LeftArm: { axis: [0, 1, 0], degrees: [0, 90] }, LeftForeArm: { axis: [0, 0, 1], degrees: [0, 60] } }));
    motion.pose(0.05);
    expect(aimedApart(motion, "upperarm.L", "forearm.L", b.LeftArm, b.LeftForeArm)).toBeLessThan(0.1);
    expect(aimedApart(motion, "forearm.L", "hand.L", b.LeftForeArm, b.LeftHand)).toBeLessThan(0.1);
    const squashed = mixamo((b) => b.Chest.scale.set(1, 2, 1));
    expect(() => new Motion(loadRig(), squashed.Hips, turning({ LeftArm: { axis: [0, 1, 0], degrees: [0, 90] } }))).toThrow(/unevenly/);
    // sitting: the knees forward, the feet as high as the hips
    const seated = mixamo((b) => {
      for (const s of ["Left", "Right"]) b[`${s}Leg`].position.set(0, 0, 0.45);
      for (const s of ["Left", "Right"]) b[`${s}Foot`].position.set(0, 0, 0.42);
    });
    expect(() => new Motion(loadRig(), seated.Hips, turning({ LeftArm: { axis: [0, 1, 0], degrees: [0, 90] } }))).toThrow(/standing/);
  });

  test("the forearm doesn't snap as the wrist turns through half a turn", () => {
    const b = mixamo(), rig = loadRig();
    const motion = new M.motion.Motion(rig, b.Hips, turning({ LeftHand: { axis: [1, 0, 0], degrees: Array.from({ length: 41 }, (_, i) => 160 + i) } }));
    const bones = ["forearm.L", "forearm2.L", "hand.L"].map((n) => rig.bone(n));
    let last: THREE.Quaternion[] | null = null, most = 0;
    for (let t = 0; t <= 2; t += 0.01) {
      motion.pose(t);
      const now = bones.map((i) => rig.W[i].clone());
      if (last) most = Math.max(most, ...now.map((q, k) => (q.angleTo(last![k]) * 180) / Math.PI));
      last = now;
    }
    // the wrist turns 0.2° a step
    expect(most).toBeLessThan(2);
  });

  test("the source can be put anywhere in a scene, by its parent", async () => {
    const { PRESETS } = M.pose, frames = ["neutral", "run", "neutral"].map((n) => PRESETS[n]);
    const src = loadRig(), dst = loadRig(), still = loadRig();
    const text = toBVH(src, PRESETS.neutral, frames, 0, 1);
    const moved = await M.motion.Motion.load(dst, buffer(text), "rig.bvh");
    const motion = await M.motion.Motion.load(still, buffer(text), "rig.bvh");
    const parent = new THREE.Group();
    parent.position.set(30, -20, 5);
    parent.quaternion.setFromEuler(new THREE.Euler(0.3, 1.2, -0.2));
    parent.scale.setScalar(2.5);
    parent.add(moved.root);
    parent.updateMatrixWorld(true);
    moved.pose(0.5);
    motion.pose(0.5);
    expect(jointsApart(dst, still, false).cm).toBeLessThan(0.01);
  });
});

describe("anatomy morphs, soft-tissue jiggle, muscles & walking gait", () => {
  test("male and female anatomy morph cleanly with valid skeletal proportions and closed head join", () => {
    const male = loadRig();
    const female = loadRig();
    female.setAnatomy({ female: 1, fat: 0.4, muscle: 0.5, breast: 0.6, butt: 0.6 });

    // Female body is shorter at the neck rim with distinct chest/pelvis geometry
    expect(female.header.rimShape.centre[1]).toBeLessThan(male.header.rimShape.centre[1] - 8);
    expect(female.H[female.bone("upperchest")].y).toBeLessThan(male.H[male.bone("upperchest")].y - 6);

    // Body-shaping morphs (fat, muscle, breast, butt) preserve the neck rim so they can update live
    const rimBefore = female.rim.map((i) => at(female.rest, i));
    female.setAnatomy({ fat: 1.0, muscle: 0.2, breast: 1.0, butt: 1.0 });
    female.rim.forEach((i, k) => expect(at(female.rest, i).distanceTo(rimBefore[k])).toBeLessThan(1e-3));

    // Female CHIMERA head fits cleanly onto the morphed female neck rim with only the lips cut open
    const model = new M.VoiceModel(M.BODIES.female, 48000);
    M.head.wearHead(female, model);
    const open = openEdges(female);
    expect(open.every(([a, b]) => a >= female.bodyVertices && b >= female.bodyVertices)).toBe(true);
    // Neutral pose still hangs arms straight down on the female skeleton
    female.pose(M.pose.neutral());
    for (const s of ["R", "L"]) {
      const dir = female.P[female.bone(`forearm.${s}`)].clone().sub(female.P[female.bone(`upperarm.${s}`)]).normalize();
      expect(dir.y).toBeLessThan(-0.999);
    }
    model.dispose();
  });

  test("soft-tissue physics produces stable breast and butt jiggle during female walking and quiets when disabled", () => {
    const rig = loadRig();
    rig.setAnatomy({ female: 1, fat: 0.55, muscle: 0.3, breast: 0.85, butt: 0.85 });
    const tissue = new M.physics.SoftTissue(rig);
    const skinned = new Float32Array(3 * rig.n);
    let peakBreast = 0, peakButt = 0;

    for (let step = 0; step < 120; step++) {
      const wf = M.walk.walkFrame(step / 60, "catwalk", rig.anatomy);
      rig.pose(wf.pose);
      tissue.step(1 / 60);
      rig.skin(skinned);
      const before = skinned.slice();
      tissue.apply(skinned);
      for (let i = 0; i < skinned.length; i++) expect(Number.isFinite(skinned[i])).toBe(true);
      peakBreast = Math.max(peakBreast, tissue.region("breastL")!.magnitude, tissue.region("breastR")!.magnitude);
      peakButt = Math.max(peakButt, tissue.region("buttL")!.magnitude, tissue.region("buttR")!.magnitude);
      if (step === 60) expect(maxMove(before, skinned, [...Array(rig.bodyVertices).keys()])).toBeGreaterThan(0.2);
    }
    // Realistic anatomical bounce range (cm): clearly visible, bounded below explosion
    expect(peakBreast).toBeGreaterThan(0.4);
    expect(peakBreast).toBeLessThan(6.0);
    expect(peakButt).toBeGreaterThan(0.25);
    expect(peakButt).toBeLessThan(5.0);

    // Lean male chest has minimal breast jiggle compared to female
    const male = loadRig();
    male.setAnatomy({ female: 0, fat: 0.08, muscle: 1.0, breast: 0, butt: 0.3 });
    const maleTissue = new M.physics.SoftTissue(male);
    let maleBreast = 0;
    for (let step = 0; step < 60; step++) {
      male.pose(M.walk.walkFrame(step / 60, "walk", male.anatomy).pose);
      maleTissue.step(1 / 60);
      maleBreast = Math.max(maleBreast, maleTissue.region("breastL")!.magnitude);
    }
    expect(maleBreast).toBeLessThan(0.15);

    // Disabling jiggle yields zero displacement
    tissue.settings.enabled = false;
    tissue.step(1 / 60);
    expect(tissue.region("breastL")!.magnitude).toBe(0);
    expect(tissue.region("buttR")!.magnitude).toBe(0);
  });

  test("muscle system fires kinematically on flex biceps, follows gait loads, bulges skin, and renders cleanly", () => {
    const rig = loadRig();
    rig.setAnatomy({ female: 0, fat: 0.05, muscle: 1.0, breast: 0, butt: 0.3 });
    const ms = new M.muscles.MuscleSystem(rig);
    expect(ms.muscles.length).toBe(30);

    // Kinematic activation: flexing biceps fires biceps.L and biceps.R strongly without external loads
    rig.pose(M.pose.PRESETS["flex biceps"]);
    ms.update(0);
    expect(ms.byName.get("biceps.R")!.activation).toBeGreaterThan(0.5);
    expect(ms.byName.get("biceps.L")!.activation).toBeGreaterThan(0.5);

    // Volume-preserving surface bulge moves nearby skin vertices outward and attenuates under high body fat
    const skinned = new Float32Array(3 * rig.n);
    const norms = new Float32Array(3 * rig.n);
    rig.skin(skinned);
    M.vertexNormals(rig, skinned, norms);
    const leanBase = skinned.slice();
    ms.deform(skinned, norms);
    const leanMove = maxMove(leanBase, skinned, [...Array(rig.bodyVertices).keys()]);
    expect(leanMove).toBeGreaterThan(0.3);

    rig.setAnatomy({ fat: 1.0 });
    const fatSkinned = leanBase.slice();
    ms.deform(fatSkinned, norms);
    const fatMove = maxMove(leanBase, fatSkinned, [...Array(rig.bodyVertices).keys()]);
    expect(fatMove).toBeLessThan(leanMove);

    // Gait load firing & vector beam draw (headless safe)
    const wf = M.walk.walkFrame(0.44 / 0.96, "walk", rig.anatomy);
    rig.pose(wf.pose);
    ms.update(0, wf.loads);
    expect(ms.byName.get("gastrocnemius.R")!.activation).toBeGreaterThan(0.6);
    ms.draw();
    ms.dispose();
  });

  test("walking gait is periodic, finite across all styles, and adapts hip sway to female anatomy", () => {
    for (const style of M.walk.GAIT_STYLES) {
      const a = M.walk.walkFrame(0, style, { female: 1 });
      const b = M.walk.walkFrame(100, style, { female: 1 });
      expect(a.speedCmPerSec).toBeGreaterThan(50);
      expect(Number.isFinite(b.pose.body.y)).toBe(true);
      expect(Number.isFinite(b.pose.legs.R.flex)).toBe(true);
    }
    // Periodicity: exactly one cycle returns identical angles
    const t0 = M.walk.walkFrame(0.15, "walk", { female: 0.5 });
    const t1 = M.walk.walkFrame(0.15 + 1 / 0.96, "walk", { female: 0.5 });
    expect(t0.pose.legs.R.flex).toBeCloseTo(t1.pose.legs.R.flex, 4);
    expect(t0.pose.legs.L.knee).toBeCloseTo(t1.pose.legs.L.knee, 4);
    // Female gait has fuller pelvic list / sway at mid-stance than male gait
    const mMid = M.walk.walkFrame(0.25 / 0.96, "walk", { female: 0 });
    const fMid = M.walk.walkFrame(0.25 / 0.96, "walk", { female: 1 });
    expect(Math.abs(fMid.pose.body.lean)).toBeGreaterThan(Math.abs(mMid.pose.body.lean));
    expect(Math.abs(fMid.pose.body.z)).toBeGreaterThan(Math.abs(mMid.pose.body.z));
  });

  test("adaptive 4× Catmull-Clark subdivision, independent male breast/butt jiggle, and male penis anatomy work cleanly", () => {
    const rig = loadRig();
    rig.setAnatomy({ female: 0, fat: 0.2, muscle: 0.4, breast: 1.0, butt: 1.0 });
    const view = new M.view.RigView(rig);
    expect(view.quadCount).toBe(rig.nq);
    view.subdivide = 1;
    expect(view.quadCount).toBe(4 * rig.nq);
    rig.pose(M.walk.walkFrame(0.2, "walk", rig.anatomy).pose);
    rig.skin(view.posed);
    view.update();
    view.dispose();

    // Independent breast & butt jiggle works on male when breast/butt sliders are raised
    const tissue = new M.physics.SoftTissue(rig);
    let maleBreast = 0, maleButt = 0;
    for (let step = 0; step < 60; step++) {
      rig.pose(M.walk.walkFrame(step / 60, "walk", rig.anatomy).pose);
      tissue.step(1 / 60);
      maleBreast = Math.max(maleBreast, tissue.region("breastL")!.magnitude);
      maleButt = Math.max(maleButt, tissue.region("buttL")!.magnitude);
    }
    expect(maleBreast).toBeGreaterThan(0.25);
    expect(maleButt).toBeGreaterThan(0.15);

    // Male external anatomy toggles cleanly with size and oscillates stably during walking
    const maleGeo = new M.maleAnatomy.MaleAnatomy(rig);
    maleGeo.size = M.maleAnatomy.PENIS_SIZES.none;
    maleGeo.update(1 / 60);
    expect(maleGeo.group.visible).toBe(false);
    maleGeo.size = M.maleAnatomy.PENIS_SIZES.L;
    let peakPenis = 0;
    for (let step = 0; step < 60; step++) {
      rig.pose(M.walk.walkFrame(step / 60, "walk", rig.anatomy).pose);
      maleGeo.update(1 / 60);
      peakPenis = Math.max(peakPenis, maleGeo.magnitude);
    }
    expect(maleGeo.group.visible).toBe(true);
    expect(peakPenis).toBeGreaterThan(0.2);
    expect(peakPenis).toBeLessThan(6.0);
    maleGeo.dispose();
  });
});

describe("the shaped body and the rest of the rig", () => {
  /** bodies from slight to the sliders' far ends, of each sex */
  const SHAPES: Record<string, Partial<AnatomyParams>> = {
    "the file's": {},
    "female, slight": { female: 1, fat: 0, muscle: 0.2, breast: 0, butt: 0 },
    "female, full": { female: 1, fat: 0.15, muscle: 0.3, breast: 0.9, butt: 1 },
    "female, heavy": { female: 1, fat: 0.8, muscle: 0.25, breast: 1, butt: 1 },
    "male, muscular": { female: 0, fat: 0.08, muscle: 1.15, breast: 0, butt: 0.45 },
    "male, heavy": { female: 0, fat: 0.95, muscle: 0.3, breast: 0.15, butt: 0.65 },
  };
  /** how far (cm) the deepest forearm or hand vertex is behind the skin of the trunk and thighs nearest it */
  function sunk(rig: RigT, P: Float32Array, N: Float32Array): number {
    const INF = rig.INF;
    const bone = (i: number) => {
      let most = 0, at = 0;
      for (let j = 0; j < INF; j++) {
        if (rig.skinWeight[INF * i + j] > most) {
          most = rig.skinWeight[INF * i + j];
          at = rig.skinIndex[INF * i + j];
        }
      }
      return rig.bones[at].name;
    };
    let deepest = 0;
    for (const s of ["R", "L"]) {
      const arm: number[] = [], body: number[] = [];
      for (let i = 0; i < rig.bodyVertices; i++) {
        const n = bone(i);
        if ([`forearm.${s}`, `forearm2.${s}`, `hand.${s}`].includes(n)) arm.push(i);
        else if (/^(hips|spine|chest|upperchest)/.test(n) || [`pelvis.${s}`, `thigh.${s}`, `thigh2.${s}`, `hipHelper.${s}`, `hipFront.${s}`].includes(n)) body.push(i);
      }
      for (const i of arm) {
        let best = 36, q = -1;
        for (const b of body) {
          const d = at(P, i).distanceToSquared(at(P, b));
          if (d < best) {
            best = d;
            q = b;
          }
        }
        if (q >= 0) deepest = Math.max(deepest, -at(P, i).sub(at(P, q)).dot(at(N, q)));
      }
    }
    return deepest;
  }

  test("with no anatomy set the body is the one from before there was any, byte for byte (a third taken as a weight, not a division, put one midline vertex on another triangle, 0.56 cm off)", () => {
    const b = readFileSync(new URL("../src/rig/body.rig.bin", import.meta.url));
    const len = b.readUInt32LE(0), h = JSON.parse(b.subarray(4, 4 + len).toString("utf8"));
    expect([h.vertices, h.quads, h.bones.length, h.rim.length]).toEqual([2915, 2896, 43, 36]);
    // the rest mesh, the skin's bones and weights, the quads: what main's file holds after its header
    const core = b.subarray(4 + len, 4 + len + h.vertices * (12 + 3 * h.influences) + 8 * h.quads);
    expect(createHash("sha256").update(core).digest("hex")).toBe("b811aa00cc27d71598cc5cc4bfd86b4cb94f82627fc168267145683249b3ffb2");
    // and a rig nothing was asked of rests where the file has it
    const plain = loadRig(), shaped = loadRig();
    shaped.setAnatomy({ fat: 0.5, breast: 0.7 });
    shaped.setAnatomy({ fat: 0, breast: 0 });
    expect(maxMove(plain.rest, shaped.rest, Array(plain.bodyVertices).keys())).toBeLessThan(1e-4);
  });

  test("a slider moved with the head on leaves the neck as the head fitted it (the rest mesh was rebuilt from the file's, and the rim sprang back 2 cm under the head); the sex then is refused", () => {
    for (const sex of ["male", "female"] as const) {
      const rig = loadRig();
      rig.setAnatomy({ female: sex === "female" ? 1 : 0, fat: 0.35, muscle: 0.45, breast: 0.55, butt: 0.55 });
      const model = new M.VoiceModel(M.BODIES[sex], 48000);
      M.head.wearHead(rig, model);
      const fitted = rig.rest.slice(), y = rig.header.rimShape.centre[1];
      const neck = [...Array(rig.bodyVertices).keys()].filter((i) => fitted[3 * i + 1] > y - 1.5);
      expect(neck.length).toBeGreaterThan(rig.rim.length);
      rig.setAnatomy({ fat: 1, muscle: 0.1, breast: 1, butt: 1 });
      expect([sex, maxMove(fitted, rig.rest, neck) < 1e-3]).toEqual([sex, true]);
      // the body under it did take the shape
      expect(maxMove(fitted, rig.rest, Array(rig.bodyVertices).keys())).toBeGreaterThan(1);
      expect(() => rig.setAnatomy({ female: sex === "female" ? 0 : 1 })).toThrow("before wearHead");
      expect(rig.anatomy.female).toBe(sex === "female" ? 1 : 0);
      model.dispose();
    }
  });

  test("standing still, the soft tissue stays where the mesh has it (a sag of its own slid a full breast 1.6 cm down the chest, into a ledge)", () => {
    const rig = loadRig();
    rig.setAnatomy(SHAPES["female, heavy"]);
    const tissue = new M.physics.SoftTissue(rig);
    tissue.settings.jiggle = 2;
    rig.pose(M.pose.PRESETS.neutral);
    for (let k = 0; k < 240; k++) tissue.step(1 / 60);
    for (const r of tissue.regions) expect([r.def.name, r.magnitude < 0.01]).toEqual([r.def.name, true]);
  });

  test("hanging arms clear every body, standing and through each gait (a hand hung 3 cm inside the file's thigh and 6 inside a wide hip; the walk turned the upper arm in, the hand into the thigh)", () => {
    for (const [name, shape] of Object.entries(SHAPES)) {
      const rig = loadRig();
      rig.setAnatomy(shape);
      const P = new Float32Array(3 * rig.n), N = new Float32Array(3 * rig.n);
      const deep = (p: Pose) => {
        rig.pose(p);
        rig.skin(P);
        M.vertexNormals(rig, P, N);
        return sunk(rig, P, N);
      };
      // anatomical zero itself is left as it is, hands in the thighs
      expect([name, deep(M.pose.PRESETS.neutral) > 2]).toEqual([name, true]);
      const out = M.armClearance(rig);
      expect([name, out > 2 && out < 20]).toEqual([name, true]);
      expect([name, "standing", deep(M.pose.armsOut(M.pose.PRESETS.neutral, out)) < 0.5]).toEqual([name, "standing", true]);
      for (const style of M.walk.GAIT_STYLES) {
        let worst = 0;
        // 4 s: more than a cycle of each
        for (let f = 0; f < 96; f++) worst = Math.max(worst, deep(M.walk.walkFrame(f / 24, style, rig.anatomy, 1, out).pose));
        expect([name, style, worst < 0.5]).toEqual([name, style, true]);
      }
    }
    // an arm already further out is left where it is, and so is one that doesn't hang at the side:
    // raised, or across the body (held out regardless, twist & look's arm across the chest went 21 cm off it)
    const { PRESETS, armsOut, pose } = M.pose;
    const out = (a: Pose["arms"]["R"]) => a.raise * Math.cos((a.plane * Math.PI) / 180);
    for (const [name, p] of Object.entries(PRESETS)) {
      for (const s of ["R", "L"] as const) {
        const a = p.arms[s], b = armsOut(p, 8).arms[s];
        if (a.raise >= 45 || Math.abs(a.plane) >= 90 || out(a) >= 8) expect([name, s, b]).toEqual([name, s, a]);
        else expect([name, s, out(b) >= out(a) - 1e-9 && out(b) <= 8 + 1e-9]).toEqual([name, s, true]);
      }
    }
    expect(out(armsOut(PRESETS.neutral, 8).arms.R)).toBeCloseTo(8, 6);
    expect(out(armsOut(pose({ bothArms: { raise: 8, plane: 40 } }), 8).arms.L)).toBeCloseTo(8, 6);
    const across = pose({ bothArms: { raise: 40, plane: 120, elbow: 90 } });
    expect(armsOut(across, 8)).toEqual(across);
  });

  test("a clip from another tool lands on a shaped female body as it does on the file's", async () => {
    const { PRESETS } = M.pose, names = Object.keys(PRESETS);
    const frames = [...names.map((n) => PRESETS[n]), PRESETS.neutral];
    const src = loadRig(), dst = loadRig();
    for (const rig of [src, dst]) rig.setAnatomy(SHAPES["female, full"]);
    src.settings.thighTwist = 1;
    const motion = await M.motion.Motion.load(dst, buffer(toBVH(src, PRESETS.neutral, frames, 90, 1 / 2.54)), "rig.bvh");
    names.forEach((name, k) => {
      src.pose(PRESETS[name]);
      motion.pose(0.5 * k);
      expect([name, jointsApart(src, dst, true).cm < 0.1, handsApart(src, dst) < 0.5]).toEqual([name, true, true]);
    });
  });

  test("a body that travels carries its soft tissue with it (damped against the world, not the body, the buttocks trailed 2.5 cm at a walk's pace and the breasts sat on their stops)", () => {
    const rig = loadRig();
    rig.setAnatomy(SHAPES["female, heavy"]);
    const tissue = new M.physics.SoftTissue(rig), part = new M.maleAnatomy.MaleAnatomy(rig);
    tissue.settings.jiggle = 2;
    part.size = M.maleAnatomy.PENIS_SIZES.M;
    const run = (speed: number) => {
      tissue.reset();
      part.reset();
      for (let f = 0; f < 180; f++) {
        rig.pose(M.pose.pose({ body: { x: (speed * f) / 60 } }));
        tissue.step(1 / 60);
        part.update(1 / 60);
      }
      return part.magnitude;
    };
    const still = run(0), going = run(138);
    for (const r of tissue.regions) expect([r.def.name, r.magnitude < 0.01]).toEqual([r.def.name, true]);
    expect(Math.abs(going - still)).toBeLessThan(0.01);
    part.dispose();
  });

  test("the male part is rooted under the skin of every body, and swings the same whichever way the body faces (placed from two sliders it stood up to 2.5 cm proud; its swing read the world's x)", () => {
    /** how far the skin is ahead of a point, along the body's forward: over 0 if the point is inside */
    const under = (rig: RigT, P: Float32Array, at: THREE.Vector3): number => {
      // a little off the midline, where the ray would run along the mesh's own edges
      const py = at.y + 0.113, pz = at.z + 0.371;
      let ahead = Infinity, hits = 0;
      for (let f = 0; f < rig.nq; f++) {
        const q = [0, 1, 2, 3].map((j) => rig.quads[4 * f + j]);
        if (q.some((v) => v >= rig.bodyVertices)) continue;
        for (const [a, b, c] of [[q[0], q[1], q[2]], [q[0], q[2], q[3]]]) {
          const ay = P[3 * a + 1] - py, az = P[3 * a + 2] - pz, by = P[3 * b + 1] - py, bz = P[3 * b + 2] - pz, cy = P[3 * c + 1] - py, cz = P[3 * c + 2] - pz;
          const w0 = by * cz - bz * cy, w1 = cy * az - cz * ay, w2 = ay * bz - az * by, sum = w0 + w1 + w2;
          if (!sum || !((w0 >= 0 && w1 >= 0 && w2 >= 0) || (w0 <= 0 && w1 <= 0 && w2 <= 0))) continue;
          const x = (w0 * P[3 * a] + w1 * P[3 * b] + w2 * P[3 * c]) / sum - at.x;
          if (x > 0) {
            hits++;
            ahead = Math.min(ahead, x);
          }
        }
      }
      return hits % 2 ? ahead : -1;
    };
    for (const [name, shape] of Object.entries(SHAPES)) {
      const rig = loadRig();
      rig.setAnatomy(shape);
      const part = new M.maleAnatomy.MaleAnatomy(rig), P = new Float32Array(3 * rig.n);
      part.size = M.maleAnatomy.PENIS_SIZES.M;
      for (let f = 0; f <= 24; f++) {
        rig.pose(f ? M.walk.walkFrame(f / 24, "walk", rig.anatomy).pose : M.pose.PRESETS.neutral);
        rig.skin(P);
        // given the frame's skin, 1 cm under it; without it, where the hips' bone carries its place: deeper, never out
        part.update(1 / 60, P);
        const d = under(rig, P, part.root);
        expect([name, f, d > 0.5 && d < 1.6]).toEqual([name, f, true]);
        part.update(0);
        expect([name, f, under(rig, P, part.root) > 0.4]).toEqual([name, f, true]);
      }
      part.dispose();
    }
    const rig = loadRig(), part = new M.maleAnatomy.MaleAnatomy(rig);
    part.size = M.maleAnatomy.PENIS_SIZES.L;
    const swing = (turn: number) => {
      part.reset();
      let sum = 0;
      for (let f = 0; f < 120; f++) {
        const p = M.walk.walkFrame(f / 60, "walk", rig.anatomy).pose;
        p.body.turn += turn;
        rig.pose(p);
        part.update(1 / 60);
        if (f >= 60) sum += part.magnitude;
      }
      return sum / 60;
    };
    const ahead = swing(0);
    expect(ahead).toBeGreaterThan(0.5);
    for (const turn of [90, 180]) expect(Math.abs(swing(turn) - ahead)).toBeLessThan(0.05);
    part.dispose();
  });

  test("standing at anatomical zero no muscle reads as contracted (their rest lengths were the A pose's: the pectorals read a third contracted, and lit)", () => {
    for (const [name, shape] of Object.entries(SHAPES)) {
      const rig = loadRig();
      rig.setAnatomy(shape);
      const ms = new M.muscles.MuscleSystem(rig);
      rig.pose(M.pose.PRESETS.neutral);
      ms.update(0);
      for (const m of ms.muscles) expect([name, m.name, m.activation < 0.02]).toEqual([name, m.name, true]);
      ms.dispose();
    }
  });

  test("the subdivided skin passes through the mesh it is made from, at one level and at two, and stays smooth (smoothed inwards, a full breast drew 1.4 cm inside its own vertices and the lips' cut 0.2 cm off the drawn lips; put back vertex by vertex, it creased along the mesh's edges and a buttock drew pointed)", () => {
    const rig = loadRig();
    rig.setAnatomy(SHAPES["female, heavy"]);
    const model = new M.VoiceModel(M.BODIES.female, 48000);
    M.head.wearHead(rig, model);
    const view = new M.view.RigView(rig);
    expect(view.subdivided).toBe(null);
    rig.pose(M.pose.PRESETS["flex biceps"]);
    rig.skin(view.posed);
    for (const level of [1, 2] as const) {
      view.subdivide = level;
      view.update();
      const drawn = view.subdivided!;
      // the mesh's own vertices are the surface's first points: within 2.5 mm of where the mesh has them
      expect([level, maxMove(view.posed, drawn, Array(rig.n).keys()) < 0.25]).toEqual([level, true]);
      // the lines drawn: four to each of the mesh's quads, sixteen where its quads are big
      expect(view.quadCount >= 4 * rig.nq && (level === 1 ? view.quadCount === 4 * rig.nq : view.quadCount > 4 * rig.nq && view.quadCount < 16 * rig.nq)).toBe(true);
    }
    // smooth over the buttock's big quads: the surface at each quad's middle (its point after the mesh's
    // own vertices) is no further from the mean of the quad's corners than an eighth of the quad's size
    const drawn = view.subdivided!, butt = rig.tissues.buttL;
    let worst = 0;
    for (let f = 0; f < rig.nq; f++) {
      const q = [0, 1, 2, 3].map((j) => rig.quads[4 * f + j]);
      if (q.some((v) => v >= rig.bodyVertices || butt[v] < 0.6)) continue;
      const mid = at(drawn, rig.n + f), mean = q.reduce((s, v) => s.add(at(view.posed, v)), new THREE.Vector3()).multiplyScalar(0.25);
      const size = at(view.posed, q[0]).distanceTo(at(view.posed, q[2]));
      worst = Math.max(worst, mid.distanceTo(mean) / size);
    }
    expect(worst).toBeLessThan(0.12);
    // the mesh's own normals are kept up while it is drawn subdivided
    expect(maxMove(view.normals, M.vertexNormals(rig, view.posed), Array(rig.n).keys())).toBeLessThan(1e-6);
    view.dispose();
    model.dispose();
  });

  test("a refused change of sex leaves the anatomy as it was (the rest of what was asked stayed set, on a body that hadn't moved)", () => {
    const rig = loadRig();
    rig.setAnatomy({ female: 1, fat: 0.2 });
    const model = new M.VoiceModel(M.BODIES.female, 48000);
    M.head.wearHead(rig, model);
    const was = { ...rig.anatomy }, rest = rig.rest.slice();
    expect(() => rig.setAnatomy({ female: 0, fat: 1.2, breast: 1 })).toThrow("before wearHead");
    expect(rig.anatomy).toEqual(was);
    expect(maxMove(rest, rig.rest, Array(rig.n).keys())).toBe(0);
    model.dispose();
  });

  test("the waist comes in and the hips fill out without the body with no anatomy changing (Curvy's waist was as wide as its ribs)", () => {
    /** the body's half width (cm) at a height, in the rest mesh, the arms left out */
    const half = (rig: RigT, y: number) => {
      let w = 0;
      for (let i = 0; i < rig.bodyVertices; i++) {
        let most = 0, bone = 0;
        for (let j = 0; j < rig.INF; j++) {
          if (rig.skinWeight[rig.INF * i + j] > most) {
            most = rig.skinWeight[rig.INF * i + j];
            bone = rig.skinIndex[rig.INF * i + j];
          }
        }
        if (/arm|hand|clavicle|shoulder|elbow/.test(rig.bones[bone].name)) continue;
        if (Math.abs(rig.rest[3 * i + 1] - y) < 1.5) w = Math.max(w, Math.abs(rig.rest[3 * i + 2]));
      }
      return w;
    };
    const plain = loadRig(), full = loadRig();
    plain.setAnatomy({ female: 1 });
    full.setAnatomy({ female: 1, butt: 1, waist: 1 });
    // the waist: the narrowest the trunk is between the hips' joint and the chest's; the hips: the widest under it
    const lo = plain.bones[plain.bone("hips")].head[1], hi = plain.bones[plain.bone("chest")].head[1];
    const waist = (rig: RigT) => Math.min(...Array.from({ length: 12 }, (_, k) => half(rig, lo + 6 + ((hi - lo - 6) * k) / 11)));
    const hips = (rig: RigT) => Math.max(...Array.from({ length: 8 }, (_, k) => half(rig, lo - 6 + (12 * k) / 7)));
    const was = [waist(plain), hips(plain)], now = [waist(full), hips(full)];
    // the waist narrower, the hips wider, and the one to the other well under what it was
    // the waist narrower (0.7 cm a side), the hips wider (1.5), and the one to the other well under what it was
    expect([was[0] - now[0] > 0.5, now[1] - was[1] > 1, now[0] / now[1] < 0.9 * (was[0] / was[1])]).toEqual([true, true, true]);
    // the buttocks carried further back
    const back = (rig: RigT) => {
      let x = Infinity;
      for (let i = 0; i < rig.bodyVertices; i++) if (rig.tissues.buttL[i] > 0.5) x = Math.min(x, rig.rest[3 * i]);
      return x;
    };
    expect(back(plain) - back(full)).toBeGreaterThan(1.5);
    // taken off again, the body is the one it was
    full.setAnatomy({ butt: 0, waist: 0 });
    expect(maxMove(plain.rest, full.rest, Array(plain.bodyVertices).keys())).toBeLessThan(1e-4);
  });

  test("a jolt runs through the soft tissue as a wave, a buttock stays on the pelvis and a breast isn't thrown up the chest (moved as one piece, at full jiggle a buttock slid off in a lump, 5 cm, with a corner at the hip)", () => {
    const rig = loadRig();
    rig.setAnatomy({ female: 1, fat: 0.05, muscle: 0.35, breast: 0.85, butt: 1, waist: 1 });
    const tissue = new M.physics.SoftTissue(rig);
    tissue.settings.jiggle = 2;
    const butt = tissue.region("buttL")!, P = new Float32Array(3 * rig.n), Q = new Float32Array(3 * rig.n);
    // the vertex most of the region, and one at its edge
    const mid = butt.verts.reduce((a, v) => (v.sw > a.sw ? v : a)), edge = butt.verts.reduce((a, v) => (Math.abs(v.sw - 0.15) < Math.abs(a.sw - 0.15) ? v : a));
    let far = 0, apart = 0;
    for (let f = 0; f < 240; f++) {
      rig.pose(M.walk.walkFrame(f / 60, "jog", rig.anatomy).pose);
      tissue.step(1 / 60);
      rig.skin(P);
      Q.set(P);
      tissue.apply(Q);
      if (f < 60) continue;
      const moved = (v: { idx: number; sw: number }) => at(Q, v.idx).sub(at(P, v.idx));
      far = Math.max(far, moved(mid).length());
      // as one piece, every vertex moves the way the centre does, by its weight: the middle is now late on that
      apart = Math.max(apart, moved(mid).sub(butt.dispWorld.clone().multiplyScalar(mid.sw)).length());
      expect(moved(edge).length()).toBeLessThan(1.5);
    }
    expect(far).toBeGreaterThan(1);
    expect(far).toBeLessThan(3.6);
    expect(apart).toBeGreaterThan(0.3);
    // a breast drops and swings, and rises under half as far (free to rise as far, 6 cm on this jog, it was flattened up the chest each stride)
    const breast = tissue.region("breastL")!;
    let up = 0, down = 0;
    for (let f = 0; f < 180; f++) {
      rig.pose(M.walk.walkFrame(4 + f / 60, "jog", rig.anatomy).pose);
      tissue.step(1 / 60);
      up = Math.max(up, breast.dispLocal.y);
      down = Math.max(down, -breast.dispLocal.y);
    }
    expect([down > 1.5 && down < 3.6, up < 0.5 * down]).toEqual([true, true]);
    // and standing still it comes to rest on the mesh
    for (let f = 0; f < 240; f++) {
      rig.pose(M.pose.PRESETS.neutral);
      tissue.step(1 / 60);
    }
    rig.skin(P);
    Q.set(P);
    tissue.apply(Q);
    expect(maxMove(P, Q, Array(rig.bodyVertices).keys())).toBeLessThan(0.01);
  });
});
