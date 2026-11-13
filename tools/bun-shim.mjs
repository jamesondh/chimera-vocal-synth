/**
 * Shim so tools/layout-lint.ts can run under Node instead of Bun on this machine:
 * Bun's spawn of the Playwright browser hangs on this Windows setup, while Node's works.
 * Only Bun.spawnSync (used to render the lint's duet show) is provided.
 */
import { spawnSync as nodeSpawnSync } from "node:child_process";

globalThis.Bun = {
  spawnSync(argv, opts) {
    const r = nodeSpawnSync(argv[0], argv.slice(1), { ...opts, encoding: "buffer" });
    return { exitCode: r.status, stdout: r.stdout, stderr: r.stderr };
  },
};
