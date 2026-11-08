/**
 * The clip page (`?clip=<id>`): one 3D head replaying a rendered voice on a clock a capture tool
 * sets (`window.__clipTime`, seconds), turning on that same clock, for a video made frame by frame
 * (tools/clip.ts). The clip is a folder `public/shows/<id>/` holding `clip.json` ({"tele":
 * "voice.tele", "turn": seconds per back-and-forth, "sweep": radians each side of the front}) and
 * the telemetry from `chimera render … --tele`.
 */
import { useEffect, useRef, useState } from "react";
import { AnatomyView3D, type HeadFeed } from "./AnatomyView3D.tsx";
import { TeleTrack } from "../../show/tele.ts";
import { getModel } from "../useModel.ts";
import { VoiceModel } from "../analysis.ts";
import type { ViewFlags } from "../render/anatomy.ts";

interface ClipFile {
  tele: string;
  turn?: number;
  sweep?: number;
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
  const [error, setError] = useState<string | null>(null);
  const started = useRef(false);
  useEffect(() => {
    if (started.current) return;
    started.current = true;
    (async () => {
      const clip: ClipFile = await (await fetch(`${base}clip.json`)).json();
      const track = new TeleTrack(await (await fetch(base + clip.tele)).arrayBuffer());
      const turn = clip.turn ?? 12, sweep = clip.sweep ?? 0.9;
      // the front is 0.62 rad round from the opening three-quarter view; start on one side
      const front = -0.62;
      window.__clipDuration = track.duration;
      setFeed({
        model: getModel(track.body, track.sampleRate),
        view: VIEW,
        frame: () => {
          const msg = track.message(track.indexAt(window.__clipTime ?? 0));
          return { tele: msg, artic: VoiceModel.articFromArray(msg.t.artic), epoch: 0 };
        },
        yaw: () => front + sweep * Math.cos((2 * Math.PI * (window.__clipTime ?? 0)) / turn),
      });
      window.__clipReady = true;
    })().catch((e) => setError(String(e)));
  }, [base]);
  if (error) return <p className="duet-msg error">This clip could not load: {error}</p>;
  return <div className="clip">{feed && <AnatomyView3D feed={feed} readouts={false} />}</div>;
}
