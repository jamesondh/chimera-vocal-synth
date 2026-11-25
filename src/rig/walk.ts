/**
 * Procedural biomechanical walking, catwalk and jog gait generator for the body rig.
 *
 * Uses periodic Catmull-Rom curves derived from clinical human gait kinematics (heel strike,
 * loading-response knee yield, mid-stance vault, terminal-stance push-off with toe extension,
 * swing-phase knee clearance, pelvic list/yaw/sway, thoracic counter-rotation and head
 * stabilization), adapting smoothly between male and female gait proportions.
 *
 * Also outputs phase-locked EMG-like `MuscleLoads` so weight-bearing contractions (stance
 * quadriceps and glutes, push-off gastrocnemius, terminal-swing hamstrings, core obliques) fire
 * authentically in sync with every step.
 */
import { pose, type Arm, type Leg, type Pose } from "./pose.ts";
import type { MuscleLoads } from "./muscles.ts";
import type { AnatomyParams } from "./rig.ts";

export type GaitStyle = "walk" | "catwalk" | "jog";
export const GAIT_STYLES: readonly GaitStyle[] = ["walk", "catwalk", "jog"];

export interface WalkFrame {
  pose: Pose;
  loads: MuscleLoads;
  /** gait cycle phase in [0, 1) (0: right heel strike, 0.5: left heel strike) */
  phase: number;
  /** forward ground speed in body cm/s (for scrolling a floor reference grid) */
  speedCmPerSec: number;
  /** foot stance contact weights (0..1) for left and right */
  contact: { L: number; R: number };
}

type Key = readonly [number, number];

/** periodic Catmull-Rom interpolation over sorted [u, val] keys with u in [0, 1) */
function samplePeriodic(keys: readonly Key[], uRaw: number): number {
  const u = ((uRaw % 1) + 1) % 1;
  const n = keys.length;
  let i1 = 0;
  for (let i = 0; i < n; i++) {
    if (keys[i][0] <= u) i1 = i;
  }
  const i0 = (i1 + n - 1) % n;
  const i2 = (i1 + 1) % n;
  const i3 = (i1 + 2) % n;
  const u1 = keys[i1][0];
  let span = keys[i2][0] - u1;
  if (span <= 0) span += 1;
  let du = u - u1;
  if (du < 0) du += 1;
  const t = du / span;
  const t2 = t * t, t3 = t2 * t;
  const p0 = keys[i0][1], p1 = keys[i1][1], p2 = keys[i2][1], p3 = keys[i3][1];
  return 0.5 * (2 * p1 + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t2 + (-p0 + 3 * p1 - 3 * p2 + p3) * t3);
}

/** smooth periodic pulse peaking at `center` with half-width `width` (in phase 0..1) */
function pulse(uRaw: number, center: number, width: number): number {
  let d = (((uRaw - center) % 1) + 1.5) % 1 - 0.5;
  d = Math.abs(d) / width;
  if (d >= 1) return 0;
  const x = 1 - d;
  return x * x * (3 - 2 * x);
}

// Clinical gait cycle curves (degrees), u = 0 at heel strike, u ≈ 0.55 at toe-off
const WALK_HIP: Key[] = [
  [0.0, 27], [0.12, 22], [0.3, 1], [0.48, -17], [0.6, -7], [0.74, 17], [0.88, 29],
];
const WALK_KNEE: Key[] = [
  [0.0, 5], [0.12, 16], [0.3, 6], [0.46, 11], [0.58, 36], [0.72, 62], [0.88, 22],
];
const WALK_ANKLE: Key[] = [
  [0.0, 4], [0.1, -6], [0.32, 7], [0.48, -14], [0.58, -20], [0.74, 4], [0.9, 6],
];
const WALK_TOES: Key[] = [
  [0.0, 0], [0.34, 0], [0.48, 34], [0.56, 40], [0.66, 8], [0.82, 0],
];

const JOG_HIP: Key[] = [
  [0.0, 38], [0.14, 24], [0.32, -14], [0.44, -26], [0.62, 16], [0.8, 44], [0.92, 42],
];
const JOG_KNEE: Key[] = [
  [0.0, 14], [0.14, 38], [0.32, 20], [0.46, 36], [0.64, 92], [0.8, 55], [0.92, 18],
];
const JOG_ANKLE: Key[] = [
  [0.0, 6], [0.14, 14], [0.34, -22], [0.48, -28], [0.68, 4], [0.86, 8],
];
const JOG_TOES: Key[] = [
  [0.0, 0], [0.22, 6], [0.36, 44], [0.48, 22], [0.64, 0],
];

interface GaitConfig {
  /** cycles per second (1 cycle = left + right step) */
  hz: number;
  /** forward speed in cm/s */
  speed: number;
  /** vertical CoM bounce amplitude (cm) */
  bob: number;
  /** baseline CoM lowering (cm) */
  drop: number;
  /** lateral pelvic sway (cm) */
  sway: number;
  /** pelvic yaw amplitude (deg) */
  pelvisYaw: number;
  /** pelvic list / hip drop amplitude (deg) */
  pelvisList: number;
  /** forward trunk lean (deg) */
  leanFwd: number;
  /** chest counter-twist relative to pelvis (deg) */
  chestTwist: number;
  /** arm forward/back swing amplitude (deg) */
  armSwing: number;
  /** baseline elbow bend & swing variation (deg) */
  elbowBase: number;
  elbowAmp: number;
  /** hip adduction inward bias (deg, negative = closer to midline) */
  stepIn: number;
  /** EMG excitation gain */
  effort: number;
  jog: boolean;
}

const CONFIGS: Record<GaitStyle, GaitConfig> = {
  walk: {
    hz: 0.96,
    speed: 138,
    bob: 2.8,
    drop: -2.4,
    sway: 2.3,
    pelvisYaw: 7.5,
    pelvisList: 4.0,
    leanFwd: 4.5,
    chestTwist: 12.5,
    armSwing: 26,
    elbowBase: 16,
    elbowAmp: 14,
    stepIn: -1.5,
    effort: 0.78,
    jog: false,
  },
  catwalk: {
    hz: 0.94,
    speed: 132,
    bob: 3.1,
    drop: -2.0,
    sway: 4.0,
    pelvisYaw: 12.0,
    pelvisList: 7.2,
    leanFwd: 2.0,
    chestTwist: 17.5,
    armSwing: 22,
    elbowBase: 12,
    elbowAmp: 10,
    stepIn: -4.8,
    effort: 0.82,
    jog: false,
  },
  jog: {
    hz: 1.42,
    speed: 255,
    bob: 6.2,
    drop: -5.8,
    sway: 2.1,
    pelvisYaw: 9.5,
    pelvisList: 4.6,
    leanFwd: 11.5,
    chestTwist: 18.0,
    armSwing: 46,
    elbowBase: 76,
    elbowAmp: 20,
    stepIn: -2.2,
    effort: 1.0,
    jog: true,
  },
};

/**
 * Evaluate a single frame of procedural walking/catwalk/jogging at `timeSec` seconds.
 * Adapts pelvic sway, step width and stride length smoothly to `anatomy`. `armOut` is how far out
 * (degrees) a hanging arm clears the body: the rig's own measure (rig.ts, `armClearance`); left
 * out, a guess from the anatomy, which a wide hip can be wider than.
 */
export function walkFrame(
  timeSec: number,
  style: GaitStyle = "walk",
  anatomy?: Partial<AnatomyParams>,
  speedScale = 1,
  armOut?: number,
): WalkFrame {
  const cfg = CONFIGS[style];
  const female = anatomy?.female ?? 0;
  const fat = anatomy?.fat ?? 0;
  const phase = (((timeSec * cfg.hz * speedScale) % 1) + 1) % 1;
  const uR = phase;
  const uL = (phase + 0.5) % 1;
  const tau = 2 * Math.PI * phase;

  // Sex-aware gait nuances: female gait has slightly narrower foot placement, fuller hip
  // figure-8 list/yaw, and slightly shorter stride; male gait has slightly broader stance & shoulders.
  const hipList = cfg.pelvisList * (0.75 + 0.55 * female);
  const hipYaw = cfg.pelvisYaw * (0.85 + 0.32 * female);
  const hipSway = cfg.sway * (0.8 + 0.45 * female);
  const stepIn = cfg.stepIn - 2.0 * female + 1.5 * fat;
  // more than a hanging arm needs: the hips sway and list towards the hand as it passes (and a
  // jog's bent arm brings the hand by nearer the hip)
  const armClear = (armOut ?? 4 + 4 * female + 5 * fat) + (cfg.jog ? 8 : 4);

  const evalLeg = (u: number): Leg => {
    const flex = samplePeriodic(cfg.jog ? JOG_HIP : WALK_HIP, u);
    const knee = Math.max(0, samplePeriodic(cfg.jog ? JOG_KNEE : WALK_KNEE, u));
    const ankle = samplePeriodic(cfg.jog ? JOG_ANKLE : WALK_ANKLE, u);
    const toes = Math.max(0, samplePeriodic(cfg.jog ? JOG_TOES : WALK_TOES, u));
    // subtle abduction during swing clearance, slight adduction toward midline in mid-stance
    const abduct = stepIn + 2.2 * pulse(u, 0.72, 0.25);
    // slight external rotation in mid-stance / push-off
    const rotate = 3.5 + 3.0 * Math.sin(2 * Math.PI * u);
    const invert = 2.0 * pulse(u, 0.06, 0.16) - 2.0 * pulse(u, 0.48, 0.16);
    return { flex, abduct, rotate, knee, ankle, invert, toes };
  };

  const evalArm = (uLegOpposite: number): Arm => {
    // Opposite arm swings forward with the contralateral leg: peak forward at uLegOpposite ≈ 0
    const swing = cfg.armSwing * Math.cos(2 * Math.PI * uLegOpposite);
    const out = armClear + (cfg.jog ? 5 : 2) * Math.max(0, -Math.cos(2 * Math.PI * uLegOpposite));
    const raise = Math.hypot(swing, out);
    const plane = (Math.atan2(swing, out) * 180) / Math.PI;
    // Elbow bends more on forward swing, straightens slightly on back swing
    const elbow = Math.max(4, cfg.elbowBase + cfg.elbowAmp * Math.max(0, Math.cos(2 * Math.PI * uLegOpposite - 0.35)));
    const turn = cfg.jog ? 35 : 18 + 8 * Math.cos(2 * Math.PI * uLegOpposite);
    // the upper arm isn't turned in: with the elbow bent that carried the hand into the thigh at
    // mid swing (4 cm on the walk, 6 on the jog)
    const rotate = 0;
    const wrist = -4 + 5 * Math.sin(2 * Math.PI * uLegOpposite);
    return { shrug: 0, reach: 0, raise, plane, rotate, elbow, turn, wrist, deviate: 0 };
  };

  // Vertical CoM oscillation at 2× cycle frequency: lowest right after heel strikes (0.08, 0.58),
  // highest at mid-stance vaults (0.30, 0.80) — sharp heel-strike deceleration excites soft-tissue jiggle!
  const bobPhase = 4 * Math.PI * phase;
  const bob = cfg.drop - cfg.bob * Math.cos(bobPhase - 0.55) - 0.28 * cfg.bob * Math.sin(2 * bobPhase);
  // Lateral sway over stance foot: +z (right) during right mid-stance (uR ≈ 0.25)
  const swayZ = hipSway * Math.sin(tau);
  // Pelvic yaw: right hip forward (+turn) at right heel strike (phase = 0)
  const pelvisTurn = hipYaw * Math.cos(tau);
  // Pelvic list (Trendelenburg drop): swing hip drops during single-leg stance
  // (during right stance phase≈0.25, left side drops → negative lean)
  const pelvisLean = -hipList * Math.sin(tau);

  // Trunk counter-rotation & stabilization
  const spineTwist = -cfg.chestTwist * Math.cos(tau);
  const spineBend = -0.82 * pelvisLean;
  const spineFlex = cfg.leanFwd + (cfg.jog ? 2.5 : 1.0) * Math.cos(bobPhase);

  // Head stabilization: keep gaze steady forward and level
  const neckTwist = -0.65 * (pelvisTurn + spineTwist);
  const neckBend = -0.55 * (pelvisLean + spineBend);
  const neckFlex = -0.45 * spineFlex;

  const p = pose({
    body: {
      tip: cfg.leanFwd * 0.45,
      lean: pelvisLean,
      turn: pelvisTurn,
      x: 0,
      y: bob,
      z: swayZ,
    },
    spine: { flex: spineFlex, bend: spineBend, twist: spineTwist },
    neck: { flex: neckFlex, bend: neckBend, twist: neckTwist },
    arms: { R: evalArm(uL), L: evalArm(uR) },
    legs: { R: evalLeg(uR), L: evalLeg(uL) },
  });

  // Phase-locked biomechanical EMG muscle excitation per side
  const e = cfg.effort;
  const loads: MuscleLoads = {};
  for (const [side, u, uOpp] of [["R", uR, uL], ["L", uL, uR]] as const) {
    // Loading response (heel strike → flat foot): quadriceps, gluteus maximus, tibialis anterior
    loads[`quadriceps.${side}`] = e * (0.92 * pulse(u, 0.08, 0.2) + 0.35 * pulse(u, 0.78, 0.18));
    loads[`gluteus.${side}`] = e * (0.95 * pulse(u, 0.08, 0.24) + 0.52 * pulse(u, 0.42, 0.2));
    loads[`gastrocnemius.${side}`] = e * (1.0 * pulse(u, 0.44, 0.2));
    loads[`tibialis.${side}`] = e * (0.72 * pulse(u, 0.96, 0.16) + 0.55 * pulse(u, 0.72, 0.18));
    loads[`hamstring.${side}`] = e * (0.85 * pulse(u, 0.88, 0.18) + 0.48 * pulse(u, 0.52, 0.18));
    loads[`adductor.${side}`] = e * (0.58 * pulse(u, 0.22, 0.22) + 0.35 * pulse(u, 0.62, 0.16));

    // Upper body reciprocal arm swing & core counter-torque
    const fwdSwing = Math.max(0, Math.cos(2 * Math.PI * uOpp));
    const backSwing = Math.max(0, -Math.cos(2 * Math.PI * uOpp));
    loads[`deltoid.${side}`] = e * (0.48 * fwdSwing + 0.22 * backSwing);
    loads[`biceps.${side}`] = e * (cfg.jog ? 0.62 * fwdSwing + 0.25 : 0.38 * fwdSwing);
    loads[`triceps.${side}`] = e * (0.44 * backSwing);
    loads[`pectoralis.${side}`] = e * (0.36 * fwdSwing);
    loads[`latissimus.${side}`] = e * (0.42 * backSwing + 0.2 * pulse(u, 0.12, 0.2));
    loads[`oblique.${side}`] = e * (0.62 * pulse(u, 0.25, 0.25) + 0.25);
    loads[`rectusAbdominis.${side}`] = e * (0.34 + 0.26 * Math.abs(Math.sin(bobPhase)));
    loads[`trapezius.${side}`] = e * (0.22 + 0.22 * fwdSwing);
    loads[`forearmFlexor.${side}`] = e * (cfg.jog ? 0.42 : 0.18 * fwdSwing);
  }

  const contact = {
    R: pulse(uR, 0.24, 0.32),
    L: pulse(uL, 0.24, 0.32),
  };

  return {
    pose: p,
    loads,
    phase,
    speedCmPerSec: cfg.speed * speedScale * (1 - 0.08 * female),
    contact,
  };
}
