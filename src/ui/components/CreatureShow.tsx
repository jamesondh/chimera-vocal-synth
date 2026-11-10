/**
 * A creature show (`?show=<id>` whose show.json names a `"creature"`): a pre-rendered Bestiary
 * creature (`chimera render creature-show … --tele`) replayed in the Bestiary's own screens — its
 * drawing, the waterfall of the sound and the unrolled airway — at the transport's playhead, with
 * the current call's name as the caption. Nothing is synthesized: the frames are the engine's.
 */
import { useMemo } from "react";
import { Screen } from "./Screen.tsx";
import { CreatureView } from "./CreatureView.tsx";
import { CreatureTubes } from "./CreatureTubes.tsx";
import { Spectrogram, type SpectrogramTrack } from "./Spectrogram.tsx";
import type { CreatureFeed } from "../creatureLive.ts";
import type { CreatureTeleTrack } from "../../show/creatureTele.ts";
import type { Transport } from "../../show/transport.ts";
import type { CreatureDef } from "../../wasm/types.ts";

export function CreatureShow(props: { def: CreatureDef; track: CreatureTeleTrack; transport: Transport; audio: AudioBuffer }) {
  const { def, track, transport, audio } = props;
  const feed = useMemo<CreatureFeed>(
    () => ({
      def,
      frame: () => {
        const t = transport.time();
        const cue = track.cueAt(t);
        return { tele: track.telemetry(track.indexAt(t)), running: transport.isPlaying, caption: cue && { name: cue.name, hint: cue.hint } };
      },
    }),
    [def, track, transport],
  );
  const sound = useMemo<SpectrogramTrack>(() => ({ samples: audio.getChannelData(0), sampleRate: audio.sampleRate, time: () => transport.time() }), [audio, transport]);
  return (
    <main className="duet-creature">
      <Screen className="creature-show-main" focus title="Creature" sub="Replayed from the render" label={`${def.name}, drawn from the recording`}>
        <CreatureView feed={feed} />
      </Screen>
      <Screen className="creature-show-wf" title="Spectrogram" sub="Waterfall of the sound">
        <Spectrogram track={sound} />
      </Screen>
      <Screen className="creature-show-airway" title="Airway" sub="Each tube unrolled">
        <CreatureTubes feed={feed} />
      </Screen>
    </main>
  );
}
