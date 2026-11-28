#!/usr/bin/env bash
# What CI runs on Windows, in Git Bash: the core's tests, a build, the validators, then the plugin
# in REAPER. A failure anywhere fails the run.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
cd "$here/../.."
tools=$(mktemp -d)

echo "::group::tests"
cargo test -p chimera-plugin --release
echo "::endgroup::"

echo "::group::build"
"$here/windows/build.sh" --package
echo "::endgroup::"

# The zip's installer, run as a tester would (CI is already an administrator, so no prompt), twice:
# onto nothing, then over itself. The host test below plays what it installed.
common=$(cygpath -u "${COMMONPROGRAMFILES:-C:\\Program Files\\Common Files}")
installer() { # "Install CHIMERA.cmd" or "Uninstall CHIMERA.cmd"
  powershell.exe -NoProfile -Command "& '$(cygpath -w "$tools/zip/CHIMERA/$1")' -NoPause; exit \$LASTEXITCODE" | tr -d '\r'
}
echo "::group::the zip's installer"
unzip -q out/CHIMERA-windows.zip -d "$tools/zip"
installer "Install CHIMERA.cmd"
installer "Install CHIMERA.cmd"
cmp "$tools/zip/CHIMERA/CHIMERA.clap" "$common/CLAP/CHIMERA.clap"
cmp "$tools/zip/CHIMERA/CHIMERA.vst3/Contents/x86_64-win/CHIMERA.vst3" "$common/VST3/CHIMERA.vst3/Contents/x86_64-win/CHIMERA.vst3"
echo "::endgroup::"

echo "::group::clap-validator"
gh release download -R free-audio/clap-validator -p '*windows.zip' -D "$tools/clap-validator"
(cd "$tools/clap-validator" && unzip -q ./*.zip && for t in ./*.tar.gz; do [ -e "$t" ] && tar xzf "$t"; done; true)
validator=$(find "$tools/clap-validator" -iname 'clap-validator.exe' | head -1)
# the same exclusions as macos/validate.sh, for the same reasons
"$validator" validate --only-failed \
  -x 'state-reproducibility-.*' -x state-invalid-random -x process-varying-sample-rates \
  "$(cygpath -w "$PWD/target/bundled/CHIMERA.clap")"
echo "::endgroup::"

echo "::group::pluginval (VST3)"
gh release download -R Tracktion/pluginval -p 'pluginval_Windows.zip' -D "$tools/pluginval"
(cd "$tools/pluginval" && unzip -q pluginval_Windows.zip)
"$(find "$tools/pluginval" -iname 'pluginval.exe' | head -1)" --strictness-level 5 --skip-gui-tests \
  --validate "$(cygpath -w "$PWD/target/bundled/CHIMERA.vst3")"
echo "::endgroup::"

echo "::group::REAPER"
"$here/host-test/run.sh"
echo "::endgroup::"

echo "::group::the zip's uninstaller"
installer "Uninstall CHIMERA.cmd"
if [ -e "$common/CLAP/CHIMERA.clap" ] || [ -e "$common/VST3/CHIMERA.vst3" ]; then
  echo "the uninstaller left CHIMERA in $common" >&2
  exit 1
fi
echo "::endgroup::"
