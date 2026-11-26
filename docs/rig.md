# The body rig

A body for the 3D head to wear, for people (and agents) making videos with the voice: the app's
own head (`src/ui/render/anatomy3d.ts`) drawn live on a posable low-poly body, its face moving as
the voice does. It's a small library, not an animation tool: there's no timeline and no renderer
of its own. You pose it, skin it and draw it in your own three.js scene, frame by frame.

## Rig playground

Run `bun run dev` and open `/rig.html`. Add figures with **+** and select them with their chips or
by clicking a figure. Drag the canvas to orbit. Choose a voice, pitch, size, wireframe or nerve
look, mouth opening and pose for each figure. Type in the chat bar and press **Enter** to speak.
Spoken words appear at the speaker's lips and fly away, even with the controls hidden.
**↑** in the empty bar recalls the last line. The selected figure speaks at its pitch, turning
toward the camera over its pose or motion before returning to it and to the mouth slider. Open
the joint groups on the right to adjust the pose. **Mirror** copies arm and leg edits to the other
side and **copy pose** copies the selected figure's joint angles as code for `pose()`.

A motion file loops on its figure. Editing that figure's pose, size, mouth or voice stops the clip.
The body, front and back views frame the stage. The torso, head, face and side views frame the
selected figure. The stage, orbit camera and pulse controls are saved in browser storage where
available. Motion files stay in the current session. **Clear stage** resets the scene. Press **H**
to hide or show the controls, or open `/rig.html?shot` to start with them hidden. **Enter** or
**/** focuses chat, showing only the bar if the controls are hidden. Sending a line hides it
again in recording mode. **Escape** leaves the field.

The playground uses one bloom pass. The nerve glow uses the strongest nerve look and dims by the
largest close-up amount so a nearby face stays legible. Mixed stages keep the wireframe threshold
and radius and average the wireframe strength with the nerve glow.

## Files

| | |
|---|---|
| `src/rig/rig.ts` | the rig: loading, posing by joint angles, skinning |
| `src/rig/pose.ts` | the `Pose` type (anatomical angles), ranges of motion, presets |
| `src/rig/head.ts` | `wearHead`: the head's skin grown from the neck, the head's lines on the head bone, the face moving |
| `src/rig/motion.ts` | `Motion`: a clip from another tool (Mixamo's FBX, BVH, glTF) carried over onto the rig |
| `src/rig/view.ts` | the wireframe in CHIMERA's blue (and heat and weights views for finding faults) |
| `src/rig/body.rig.bin` | the mesh, skeleton and weights (generated: `bun tools/rig/gen-body.ts`, never edited) |
| `src/rig/playground.ts`, `rig.html` | the dev playground (not in the app's build) |
| `src/rig/nerves.ts` | the nerve x-ray: the body's nervous system as round lines, the head drawn to go with it |

## Use

```ts
import * as THREE from "three";
import { initChimera } from "./src/wasm/chimera.ts";
import rigUrl from "./src/rig/body.rig.bin?url";

await initChimera();
// the head reads the engine's tables as it loads, so it (and what imports it) comes after the engine
const [{ Rig }, { pose }, { wearHead }, { RigView }, { VoiceModel }, { PRESETS: BODIES }] = await Promise.all([
  import("./src/rig/rig.ts"), import("./src/rig/pose.ts"), import("./src/rig/head.ts"),
  import("./src/rig/view.ts"), import("./src/ui/analysis.ts"), import("./src/ui/body.ts"),
]);
const rig = await Rig.load(rigUrl);
const model = new VoiceModel(BODIES.male, 48000);    // yours: keep it for the rig's life, dispose it after
const head = wearHead(rig, model);                    // before the view and any buffers: it adds the head's skin to the mesh
const view = new RigView(rig);
const skinned = new Float32Array(3 * rig.n);          // rig.n counts the head's skin now
scene.add(view.group, head.mount);                    // under the same parent
renderer.outputColorSpace = THREE.LinearSRGBColorSpace; // the head's colours are the app's display values

// every frame
rig.pose(pose({ arms: { R: { raise: 90, elbow: 40 } }, neck: { twist: 30 } }));
rig.skin(skinned);                                    // centres of rotation; rig.skin(out, "lbs") for plain blending
head.update(camera, artic, tele, dt);                 // the voice now: its articulation, its telemetry (or null)
head.face(skinned, view.posed);                       // the jaw, cheeks and lips on top
view.update();
head.resize(width, height);                           // on resize: the head's line widths are in px

// done with it (or changing voice body: a new rig, head and view)
scene.remove(view.group, head.mount);
view.dispose();
head.dispose();
model.dispose();
```

Or move it as a clip from another tool does, instead of posing it:

```ts
import { Motion } from "./src/rig/motion.ts";
const motion = await Motion.load(rig, await file.arrayBuffer(), file.name); // .fbx, .bvh, .glb or .gltf
// every frame, instead of rig.pose(…)
motion.pose(seconds);                                 // loops; motion.duration, motion.clip
```

`artic` and `tele` come from wherever the voice runs: the live worklet's telemetry messages
(`VoiceModel.articFromArray(msg.t.artic)`), or a recording replayed with `TeleTrack`
(`chimera render notes … --tele voice.tele`, `src/show/tele.ts`). Without a voice, pass an
articulation of your own (`REST_ARTIC` from `src/ui/render/anatomy3d.ts` is the mouth closed)
and `null`. Set the mounted head's `head.head.gazeAtCamera` to aim its eyes at the camera.
A different body (a child's voice, say) is a new rig and a new head: the head's skin
is fitted to the model once.

## How it works

- **Body:** MakeHuman's CC0 low-poly body (proxy741, hand-laid loops), on MPFB2's Rigify human
  skeleton and weights: 33 bones, each limb bone split in two for twist, and Rigify's helpers
  (the shoulder's cap and the hip turn half way with the limb; the elbow and knee swell as they
  bend).
- **Poses** are angles as an anatomist names them, from anatomical zero (standing, arms
  hanging, thumbs forward). The shoulder is the ISB "globe": `raise`, `plane` (0 out to the side,
  90 forward) and `rotate`. Left and right take the same angles; the rig mirrors them. The rig
  adds what bodies do on their own: the collarbone rises about 10° as the arm goes overhead, the
  arm turns out as it rises to the side, the pelvis rolls back past about 55° of hip flexion,
  twist is shared along each limb, and a head turn is spread up the whole neck.
- **Skinning:** optimised centres of rotation (Le & Hodgins 2016), on the CPU: no candy-wrapper
  twist, no bulge. Plain linear blending is there too (what other tools do).
- **Head:** a quad grid grown from the neck's rim over the head's own surface (`skinAt`), closed
  by a patch over the crown and run on over the mouth, square for square. It's cut open along
  the lips' line, as wide as the mouth can spread; each frame the cut is laid on the lips the
  head draws (`lipEdges`), corner to corner (a rounded mouth gathers the skin in), and the jaw
  and cheeks carry the rest of the face (`skinNow`). The head's
  own lines (eyes, nose, lips, the vocal tract inside) are drawn by Anatomy3D on the head bone,
  with `drawing.mounted` (the body turns the head) and the lip patch's own skin lines left out.
- **Looks** (`HEAD_LOOKS`): `vocal` (the default: the grid and the face in front, the insides at
  half, for singing and speaking), `outside` (the insides faint, for acting), `skinned` (the
  app's insides at full under a half-bright grid) and `classic` (the app's head as it draws
  itself).

## Motion from other tools

`Motion.load` reads the file with three.js's loader for it (`readMotion`: FBX, BVH, a GLB or a
.gltf with nothing in separate files), plays a clip of it on its own skeleton (the longest, or
`{ clip: name or index }`) and carries each frame over onto the rig. For a scene you've loaded
yourself (a .gltf with separate files, a VRM), `new Motion(rig, root, clip)`.

- **Matching:** the limbs and head by name (Mixamo, CMU and DAZ BVHs, VRM and VRoid, Unreal,
  Rigify, 3ds Max Biped: `rigName`), each the bone the clip itself moves by that name;
  `motion.bones` says what was found. The trunk is found by where it sits between them, and a
  spine or neck of any number of bones is spread along the rig's by length (the chest the arms
  hang from, and the head, always match). For names it misses, or two bones that could be one
  of the rig's and the clip moves both: `{ map: { "DEF-spine.006": "head" } }` (it comes first).
- **Carrying over:** the source as loaded is its reference: each limb's turn then is swung until
  it points as the rig's does at anatomical zero (the hand also turned so its palm faces as the
  rig's, where it has fingers). Each frame, a source bone's turn since its reference is the rig
  bone's, in a world matched by the source's up axis and its hips' facing; so the source's bone
  axes, units, facing and up axis don't matter. A T or an A pose reference both work, palms down
  or in; a limb's roll can't be seen in a straight limb, so a reference with the arms turned
  otherwise (palms forward) comes out turned by as much. A mirrored or unevenly scaled source is
  refused, and so is one that doesn't stand as loaded (its height can't be told). Limb
  twist is shared along the rig's twist bones, as its posing does (without it a turned wrist
  pinches), except near half a turn, where it can't be told which way round it went. Only rotations carry over: the source's bones' lengths and any movement of a joint
  other than the hips are the rig's own.
- **Where it stands:** the hips move as the source's do, measured from the source's origin and
  scaled by leg length (hip joints over ankles). The floor is an estimate: where the source's
  lower ankle usually is over the clip (a low percentile), put where the rig's stands. A clip
  that never touches down stands on its lowest. Its feet can slide a little where its
  proportions differ from the rig's (there's no foot locking).
- A Motion plays its clip on the source's scene (`motion.root`) and reads it relative to that
  scene's parent: put it beside the rig by its parent (`motion.C` and `motion.scale` turn and
  size it as the rig), not by moving `root` itself. For another clip, load the file again.

Mixamo: download "FBX Binary", with or without skin. Mixamo's files can't be redistributed, so
none are in the repo; the tests write the rig's own poses as a BVH and read them back.

## Frames

Body cm: x forward, y up, z out of the body's right side. The skeleton's bind pose is the mesh's
(an A pose: arms out and down about 40°). Each bone has a parent (`rig.bones[i].parent`), its
rest joint `rig.H[i]`, and after `pose` its turn from rest `rig.S[i]` and joint `rig.P[i]`
(x' = S (x − H) + P). The head is drawn in its own cm, `head.s` times larger than the body.

## Checks

- `bun test tests/rig.test.ts` (part of `bun run test`): the mesh's open edges, skinning at
  rest, anatomical zero, the lips' cut lying on the drawn lips (shut; open narrow and wide, the
  head turned), weights summing to 1, the face's motion staying off the crown and the rim, the
  head following its parent's transform; and motion: bone names across tools, every preset
  written as a BVH (from a standing and a T-pose reference, turned, in other units) and read
  back to within 0.1 cm and the hands to 0.5°, the hips' height, a skeleton without toes, a
  one-frame clip, seeking, and the source moved by its parent.
- `/rig.html`: look at it. A metric isn't the picture.

## The nerve x-ray

Another way to show the body, for videos: no skin and no grid, only a nervous system in red,
purple and blue, and the head's recognisable parts (the eyes, nose and lips, and the vocal tract
inside). Try the **by function**, **x-ray by depth** and **signal** looks in the playground.

```ts
import { Nerves, NerveView } from "./src/rig/nerves.ts";   // after initChimera, as the head
const head = wearHead(rig, model);                         // no RigView: the skin isn't drawn
const nerves = new Nerves(rig, head);                      // a second or three; leaves the rig at its neutral pose
const view = new NerveView(nerves);                        // it sets how the head is drawn
scene.add(view.mesh, head.mount);                          // under the same parent

// every frame
rig.pose(p);                                               // if it moves (a motion: motion.pose(t))
view.update(camera, artic, tele, dt, canvasHeightCss);     // in place of head.update: it calls it
view.pose();                                               // if the rig moved: the nerves where the bones are now
head.resize(w, h);                                         // on resize
view.resize(w, h, renderer.getPixelRatio());
```

`view.look` is `"function"` (solid: motor nerves red, sensory blue, mixed purple),
`"xray"` (seen through, near red to far blue) or `"signal"` (red at the brain to blue at the
fingers and toes, pulses running out); `view.signal` (0–1) is how much of the pulses the first
two show, `view.speed` how fast they run (cm/s). The page's bloom is part of the look: it
weakens as `view.close` (0 to 1) says the camera has come close to the face.

- **What's there:** the brain's folds (contours of a smooth noise field over the head's brain
  outline), the cerebellum, brainstem, spinal cord and cauda equina; the spinal nerves at each
  level, the brachial, lumbar and sacral plexuses and the limbs' named nerves; the intercostals;
  the sympathetic chains; the vagus with its recurrent laryngeal loops; the cranial nerves to the
  eyes, nose, face and tongue; and twigs forking out to just under the skin, which is what draws
  the body's outline once the skin is gone. Laid out to read as a nervous system, not as anatomy
  to rely on.
- **Built on the body:** in the neutral pose, by casting rays from the bones out to the skin (a
  nerve at so many degrees round a limb and so far out to its skin), from the joints, and in the
  head's own frame for the head.
- **Moving:** each point is bound to the bones of the skin nearest it, of its own part of the
  body only (a hanging hand's nerves once took the weights of the thigh beside them). Weights
  are smoothed along a nerve, a plexus's run into a limb goes over from the trunk's to the
  limb's, and where nerves meet they share one binding. A twig hangs off its nerve where it
  leaves it (the piece and how far along it), turned as the nerve's bones turn there. Above the
  jaw's line, the head carries them. All of it is in the rig's frame: the mesh and the head's
  mount go under the same parent.
- **Drawn as lines:** each piece a strip facing the camera, as wide as the nerve in cm (never
  under about a px), shaded across as if round, its joints cut square (rounded where it's seen
  end-on), its depth the round surface's.
- **The head:** its own brain, skull section, cervical spine and ears are left out
  (`HeadDrawing.parts`); the eyes, nose and lips are drawn wider, in their own colours, with the
  nose's ridge (`parts.pop`), the nerves cleared away behind them while the face is towards the
  camera. Its lines are px wide whatever the head's size: far off they're softer and thinner (or
  they'd pile up and bloom to white), and close up the nerves (cm wide, so widening on screen)
  are dimmer, so the face isn't washed out either way.
- **Not yet:** the face's nerves don't follow the jaw. A pose is redrawn in 8 to 22 ms here
  (about 20,000 pieces), on the CPU: fine for stills and for a motion clip drawn frame by frame,
  heavy for one played live.

`bun test tests/rig.test.ts` poses every preset and checks no nerve is pulled more than 2.5
times (and 6 cm) longer than it was built, that nerves stay joined where they meet and twigs on
their nerves, and that a parent's transform moves it all once.

## Provenance

`body.rig.bin` is CC0: MakeHuman's base mesh, targets and proxies
([makehuman](https://github.com/makehumancommunity/makehuman), `LICENSE.ASSETS.md`; the proxy
from the CC0 system assets pack) and MPFB2's Rigify human rig and weights
([mpfb2](https://github.com/makehumancommunity/mpfb2), `LICENSE.md`, assets). The generator pins
each source by commit, or by hash where the source has no version.

## Not yet

Fingers (folded into the hand), dragging hands and feet (IK), keeping imported feet planted,
exporting the body (GLB). Deep bends (elbow or knee near 150°, a deep
squat) still show some strain.
