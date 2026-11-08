/** Page addresses: the Arabic page lives at /arabic; every other mode is on / (they switch in place). */
import type { Mode } from "./store.ts";

/** The mode a page address opens in: /arabic is the Arabic page, everything else starts in Play. */
export function modeOfPath(path: string): Mode {
  return /^\/arabic\/?$/.test(path) ? "arabic" : "play";
}

/** The address of a mode (only the Arabic page has its own). */
export function pathOfMode(m: Mode): string {
  return m === "arabic" ? "/arabic" : "/";
}
