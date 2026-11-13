/**
 * Entry: load the engine (wasm) first — UI modules call into it as they load (Sing mode builds
 * its syllables at import time) — then import and render the app. A failed load can be retried.
 */
import "./ui/styles.css";
import { initChimera } from "./wasm/chimera.ts";
// the texts' language: zustand only, safe before the engine loads
import { t } from "./ui/i18n/index.ts";

const root = document.getElementById("root")!;

async function boot(): Promise<void> {
  try {
    await initChimera();
  } catch (err) {
    root.replaceChildren();
    const box = document.createElement("div");
    box.className = "boot-error";
    const msg = document.createElement("p");
    msg.textContent = `${t("The voice engine could not load")}: ${err instanceof Error ? err.message : String(err)}`;
    const retry = document.createElement("button");
    retry.className = "primary";
    retry.textContent = t("Try again");
    retry.onclick = () => void boot();
    box.append(msg, retry);
    root.append(box);
    return;
  }
  const { mount } = await import("./ui/mount.tsx");
  mount(root);
}

void boot();
