#!/usr/bin/env bash
# Build CHIMERA.clap and CHIMERA.vst3 for Windows (x64), in Git Bash, into target/bundled. Both hold
# the same DLL: it exports clap_entry, and clap-wrapper adds the VST3 entry point. Unsigned.
#
#   crates/chimera-plugin/windows/build.sh [--install] [--package]
#   --install  copy into Common Files\CLAP and Common Files\VST3 (needs an administrator, as CI is)
#   --package  out/CHIMERA-windows.zip: a CHIMERA folder with both, the licences, a how-to, and an
#              installer to double-click ("Install CHIMERA.cmd", which runs install.ps1)
set -euo pipefail
cd "$(dirname "$0")/../../.."

# The page the editor shows is embedded by build.rs, so it has to exist first.
bun install --frozen-lockfile
bun run build
cargo build -p chimera-plugin --profile plugin

out=target/bundled
rm -rf "$out"
mkdir -p "$out/CHIMERA.vst3/Contents/x86_64-win" "$out/CHIMERA.vst3/Contents/Resources"
bun tools/third-party-notices.ts > "$out/THIRD-PARTY-LICENSES.txt"
cp LICENSE "$out/"
cp target/plugin/chimera_plugin.dll "$out/CHIMERA.clap"
cp target/plugin/chimera_plugin.dll "$out/CHIMERA.vst3/Contents/x86_64-win/CHIMERA.vst3"
cp LICENSE "$out/THIRD-PARTY-LICENSES.txt" "$out/CHIMERA.vst3/Contents/Resources/"

for arg in "$@"; do
  case "$arg" in
    --install)
      common=$(cygpath -u "${COMMONPROGRAMFILES:-C:\\Program Files\\Common Files}")
      mkdir -p "$common/CLAP" "$common/VST3"
      rm -rf "$common/VST3/CHIMERA.vst3"
      cp "$out/CHIMERA.clap" "$common/CLAP/"
      cp -R "$out/CHIMERA.vst3" "$common/VST3/"
      echo "installed into $common"
      ;;
    --package)
      # a CHIMERA folder, as on the other systems, with an installer to double-click (windows/install.ps1).
      # The cmd files and the how-to get Windows line endings.
      version=$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)
      pkg=$(mktemp -d)
      mkdir "$pkg/CHIMERA"
      cp -R "$out/CHIMERA.clap" "$out/CHIMERA.vst3" "$out/LICENSE" "$out/THIRD-PARTY-LICENSES.txt" \
        crates/chimera-plugin/windows/install.ps1 "$pkg/CHIMERA/"
      crlf() { sed 's/$/\r/' >"$1"; }
      crlf "$pkg/CHIMERA/Install CHIMERA.cmd" <<'CMD'
@echo off
rem CHIMERA's CLAP and VST3 into Common Files\CLAP and Common Files\VST3 (asks for an administrator)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" %*
CMD
      crlf "$pkg/CHIMERA/Uninstall CHIMERA.cmd" <<'CMD'
@echo off
rem removes CHIMERA's CLAP and VST3 from Common Files (asks for an administrator)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" -Uninstall %*
CMD
      crlf "$pkg/CHIMERA/How to install.txt" <<TXT
CHIMERA $version, a voice instrument: CLAP and VST3 for Windows 10 and 11 (x64).

Double-click "Install CHIMERA.cmd" and say yes when Windows asks for an administrator. It copies

  CHIMERA.clap  into  C:\\Program Files\\Common Files\\CLAP
  CHIMERA.vst3  into  C:\\Program Files\\Common Files\\VST3

where Windows DAWs look for plugins. Then rescan plugins in your DAW and put CHIMERA on an
instrument track.

The plugins aren't signed yet, so Windows may say "Windows protected your PC" when you run the
installer: click "More info", then "Run anyway". Or copy the two plugins into those folders yourself.

To remove CHIMERA, close your DAW and double-click "Uninstall CHIMERA.cmd".

More: https://chimera.audio/plugin/
TXT
      mkdir -p out
      rm -f out/CHIMERA-windows.zip
      zip=$(cygpath -w "$PWD/out/CHIMERA-windows.zip")
      (cd "$pkg" && 7z a -tzip -bso0 "$zip" CHIMERA)
      rm -rf "$pkg"
      ;;
  esac
done
