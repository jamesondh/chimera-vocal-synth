/**
 * A square video of one 3D head reciting or speaking Arabic, with its sound, made frame by frame:
 * the engine renders the audio and records every state of the voice (`chimera render arabic …
 * --tele`), the clip page (`?clip=<id>`) shows the head at each frame's time, turning slowly, and
 * ffmpeg puts the frames and the audio together (H.264 + AAC, for social video).
 *
 * Usage: bun tools/clip.ts <out.mp4> "<vowelled Arabic>" [--body male|female|child] [--speak]
 *        [--size 1080] [--fps 30] [--url http://127.0.0.1:5317/] [--id clip] [--cpu]
 * Needs the dev server (bun run dev) or a preview of a build, ffmpeg, and the release CLI
 * (cargo build --release -p chimera-cli). The work files go in public/shows/<id>/ (gitignored).
 */
import { chromium } from "playwright-core";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const argv = process.argv.slice(2);
const opt = (name: string, d: string) => {
  const i = argv.indexOf(`--${name}`);
  return i >= 0 ? argv.splice(i, 2)[1] : d;
};
const flag = (name: string) => {
  const i = argv.indexOf(`--${name}`);
  return i >= 0 ? (argv.splice(i, 1), true) : false;
};
const body = opt("body", "male");
const size = Number(opt("size", "1080"));
const fps = Number(opt("fps", "30"));
const url = opt("url", "http://127.0.0.1:5317/");
const id = opt("id", "clip");
const speak = flag("speak");
// the GPU when there is one (--cpu: SwiftShader, for a machine without)
const cpu = flag("cpu");
const [out, text] = argv;
if (!out || !text) throw new Error('usage: bun tools/clip.ts <out.mp4> "<vowelled Arabic>" [--body …] [--speak]');

const dir = `public/shows/${id}`;
rmSync(dir, { recursive: true, force: true });
mkdirSync(`${dir}/frames`, { recursive: true });
// 1. the sound and every state of the voice, from one render
execFileSync("target/release/chimera", ["render", "arabic", `${dir}/voice`, text, body, "--tele", `${dir}/voice.tele`, "--words", `${dir}/words.json`, ...(speak ? [] : ["--recite"])], { stdio: "inherit" });
writeFileSync(`${dir}/clip.json`, JSON.stringify({ tele: "voice.tele", words: "words.json", turn: 16, centre: 0.7, sweep: 0.55 }));

// 2. the head at each frame's time
const browser = await chromium.launch({ headless: true, args: cpu ? ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"] : ["--use-angle=metal", "--enable-gpu", "--ignore-gpu-blocklist"] });
const page = await browser.newPage({ viewport: { width: size, height: size }, deviceScaleFactor: 1 });
page.on("pageerror", (e) => console.error("page:", e.message));
await page.goto(new URL(`?clip=${id}`, url).href, { waitUntil: "networkidle" });
await page.waitForFunction(() => window.__clipReady === true, null, { timeout: 30000 });
const duration = await page.evaluate(() => window.__clipDuration ?? 0);
const n = Math.ceil(duration * fps);
const canvas = page.locator(".clip canvas").first();
// let the head settle and the glow warm up before the first frame
await page.evaluate(() => (window.__clipTime = 0));
await page.waitForTimeout(1500);
for (let i = 0; i < n; i++) {
  await page.evaluate((t) => {
    window.__clipTime = t;
    // two animation frames: one to draw the new state, one for it to reach the screen
    return new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r())));
  }, i / fps);
  await canvas.screenshot({ path: `${dir}/frames/${String(i).padStart(5, "0")}.png` });
  if (i % fps === 0) process.stdout.write(`\rframe ${i}/${n}`);
}
await browser.close();
console.log(`\r${n} frames at ${fps} fps (${duration.toFixed(2)} s)`);

// 3. frames + sound
execFileSync("ffmpeg", ["-y", "-loglevel", "error", "-framerate", String(fps), "-i", `${dir}/frames/%05d.png`, "-i", `${dir}/voice.wav`, "-c:v", "libx264", "-preset", "slow", "-crf", "18", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "192k", "-ar", "48000", "-shortest", "-movflags", "+faststart", out], { stdio: "inherit" });
console.log(`wrote ${out}`);
