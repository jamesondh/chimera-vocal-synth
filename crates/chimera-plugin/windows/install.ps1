# Install CHIMERA's CLAP and VST3 from this folder into Common Files\CLAP and Common Files\VST3, the
# folders every Windows DAW scans. Those need an administrator, so it asks for one. Shipped in
# CHIMERA-windows.zip beside "Install CHIMERA.cmd" and "Uninstall CHIMERA.cmd", which run it.
#
#   install.ps1 [-Uninstall] [-NoPause]
#   -NoPause  don't wait for Enter at the end (a double-clicked window would otherwise close at once)
param([switch]$Uninstall, [switch]$NoPause)
$ErrorActionPreference = 'Stop'

function Done([int]$code) {
  if (-not $NoPause) { Read-Host 'Press Enter to close' | Out-Null }
  exit $code
}

# C:\Program Files\Common Files, also from a 32-bit PowerShell (whose CommonProgramFiles is the x86 one)
$common = if ($env:CommonProgramW6432) { $env:CommonProgramW6432 } else { $env:CommonProgramFiles }
$plugins = @(
  @{ Name = 'CHIMERA.clap'; Folder = Join-Path $common 'CLAP' },
  @{ Name = 'CHIMERA.vst3'; Folder = Join-Path $common 'VST3' }
)

$identity = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $identity.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  # again as an administrator, in a window of its own that says how it went
  $again = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$PSCommandPath`"")
  if ($Uninstall) { $again += '-Uninstall' }
  if ($NoPause) { $again += '-NoPause' }
  try {
    $p = Start-Process powershell -Verb RunAs -Wait -PassThru -ArgumentList $again
    exit $p.ExitCode
  } catch {
    Write-Host "CHIMERA installs into $common, which needs an administrator: nothing was changed."
    Done 1
  }
}

if ($Uninstall) {
  foreach ($p in $plugins) {
    $dest = Join-Path $p.Folder $p.Name
    if (Test-Path -LiteralPath $dest) {
      try { Remove-Item -LiteralPath $dest -Recurse -Force } catch {
        Write-Host "Couldn't remove $dest. Close your DAW, then run this again."
        Done 1
      }
      Write-Host "removed $dest"
    }
  }
  Write-Host 'CHIMERA is uninstalled. Rescan plugins in your DAW.'
  Done 0
}

foreach ($p in $plugins) {
  if (-not (Test-Path -LiteralPath (Join-Path $PSScriptRoot $p.Name))) {
    Write-Host "This folder has no $($p.Name): nothing was installed. Unzip the whole download, then run this from its folder."
    Done 1
  }
}

# Each new plugin is copied in beside the old one first, under a name no DAW scans, and the old ones
# are swapped out only once every copy has worked. Windows won't let a DAW's loaded plugin be removed,
# so that's the one step that can fail, and it says what to do.
$staged = @()
try {
  foreach ($p in $plugins) {
    New-Item -ItemType Directory -Force -Path $p.Folder | Out-Null
    $s = Join-Path $p.Folder ".$($p.Name).installing"
    if (Test-Path -LiteralPath $s) { Remove-Item -LiteralPath $s -Recurse -Force }
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot $p.Name) -Destination $s -Recurse
    $staged += $s
    # downloaded files carry the internet's mark, which some hosts and Windows' checks hold against them
    Get-ChildItem -LiteralPath $s -Recurse -File -Force | Unblock-File
  }
  foreach ($p in $plugins) {
    $dest = Join-Path $p.Folder $p.Name
    if (Test-Path -LiteralPath $dest) { Remove-Item -LiteralPath $dest -Recurse -Force }
    Move-Item -LiteralPath (Join-Path $p.Folder ".$($p.Name).installing") -Destination $dest
  }
} catch {
  foreach ($s in $staged) { if (Test-Path -LiteralPath $s) { Remove-Item -LiteralPath $s -Recurse -Force -ErrorAction SilentlyContinue } }
  Write-Host "Couldn't install: $($_.Exception.Message)"
  Write-Host 'If your DAW is open with CHIMERA loaded, close it, then run this again.'
  Done 1
}

Write-Host 'CHIMERA is installed:'
foreach ($p in $plugins) { Write-Host "  $(Join-Path $p.Folder $p.Name)" }
Write-Host 'Rescan plugins in your DAW, and put CHIMERA on an instrument track.'
Done 0
