/**
 * The clip page (`?clip=<id>`): one 3D head replaying a rendered voice on a clock a capture tool
 * sets (`window.__clipTime`, seconds), turning on that same clock, for a video made frame by frame
 * (tools/clip.ts). The clip is a folder `public/shows/<id>/` holding `clip.json` (see ClipFile), the
 * telemetry from `chimera render … --tele` and the words from `--words`. The head turns between a
 * near-front and a profile view, always facing the screen's right, where its words are written.
 */
import { useEffect, useRef, useState } from "react";
import { AnatomyView3D, type HeadFeed } from "./AnatomyView3D.tsx";
import { TeleTrack } from "../../show/tele.ts";
import { getModel } from "../useModel.ts";
import { VoiceModel } from "../analysis.ts";
import type { ViewFlags } from "../render/anatomy.ts";
import { mouthWords } from "../mouthWords.ts";
import { ARABIC } from "../render/vector.ts";
import { V } from "../colors.ts";
import type { OverlayFn } from "./AnatomyView.tsx";

interface ClipFile {
  tele: string;
  /** the words and when each is said (`chimera render arabic … --words`) */
  words?: string;
  /** seconds per back-and-forth turn, its centre and its half-width: the camera's angle round the
   *  head in radians, 0 at the profile and π/2 in front (the face looks toward the screen's right
   *  between them) */
  turn?: number;
  centre?: number;
  sweep?: number;
}

interface Word {
  text: string;
  t0: number;
  t1: number;
}

const VIEW: ViewFlags = { pressure: true, particles: true, grid: false, calipers: false };

declare global {
  interface Window {
    __clipTime?: number;
    __clipReady?: boolean;
    __clipDuration?: number;
  }
}

export function ClipPage(props: { id: string }) {
  const base = `${import.meta.env.BASE_URL}shows/${props.id}/`;
  const [feed, setFeed] = useState<HeadFeed | null>(null);
  const [overlay, setOverlay] = useState<OverlayFn | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);
  const started = useRef(false);
  useEffect(() => {
    if (started.current) return;
    started.current = true;
    (async () => {
      const clip: ClipFile = await (await fetch(`${base}clip.json`)).json();
      const track = new TeleTrack(await (await fetch(base + clip.tele)).arrayBuffer());
      const turn = clip.turn ?? 12, centre = clip.centre ?? 0.7, sweep = clip.sweep ?? 0.55;
      // the opening three-quarter view is 0.62 rad round from the profile
      const home = 0.62;
      const time = () => window.__clipTime ?? 0;
      if (clip.words) {
        const words: Word[] = await (await fetch(base + clip.words)).json();
        const now = () => {
          const t = time();
          const i = words.findIndex((w) => t >= w.t0 && t < w.t1 + 0.05);
          return i < 0 ? null : { key: i, text: words[i].text };
        };
        // large: the clip is watched small, on a phone
        setOverlay(() => mouthWords(now, V.white, 1, ARABIC, 1.6));
      }
      window.__clipDuration = track.duration;
      setFeed({
        clock: time,
        model: getModel(track.body, track.sampleRate),
        view: VIEW,
        frame: () => {
          const msg = track.message(track.indexAt(window.__clipTime ?? 0));
          return { tele: msg, artic: VoiceModel.articFromArray(msg.t.artic), epoch: 0 };
        },
        // start near the front, swing toward the profile and back
        yaw: () => centre - home + sweep * Math.cos((2 * Math.PI * time()) / turn),
      });
      window.__clipReady = true;
    })().catch((e) => setError(String(e)));
  }, [base]);
  if (error) return <p className="duet-msg error">This clip could not load: {error}</p>;
  return <div className="clip">{feed && <AnatomyView3D feed={feed} readouts={false} overlay={overlay} />}</div>;
}
