# ferro font download script
# Run once before building: .\scripts\download_fonts.ps1
# Fonts are embedded at compile time via build.rs

$fontsDir = "$PSScriptRoot\..\assets\fonts"
New-Item -ItemType Directory -Force -Path $fontsDir | Out-Null

function Fetch($url, $out) {
    Write-Host "  Downloading $(Split-Path $out -Leaf) ..."
    try {
        $wr = [System.Net.WebClient]::new()
        $wr.Headers.Add("User-Agent", "ferro-font-downloader")
        $wr.DownloadFile($url, $out)
        Write-Host "    -> OK ($([Math]::Round((Get-Item $out).Length / 1KB)) KB)"
    } catch {
        Write-Warning "    -> FAILED: $_"
        Write-Warning "    Skipping — app works without this font."
        if (Test-Path $out) { Remove-Item $out }
    }
}

Write-Host "Downloading fonts to $(Resolve-Path $fontsDir -ErrorAction SilentlyContinue) ..."
Write-Host ""

# ── Material Symbols Rounded (Google, Apache-2.0) ────────────────────────────
# Direct raw URL — the filename contains brackets, so we build the Uri explicitly
$msName = "MaterialSymbolsRounded[FILL,GRAD,opsZ,wght].ttf"
$msEncoded = "MaterialSymbolsRounded%5BFILL%2CGRAD%2CopsZ%2Cwght%5D.ttf"
$msUrl = "https://raw.githubusercontent.com/google/material-design-icons/master/variablefont/$msEncoded"
Fetch $msUrl "$fontsDir\MaterialSymbolsRounded.ttf"

# ── JetBrains Mono Regular (JetBrains, OFL) ─────────────────────────────────
$jbUrl = "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/fonts/ttf/JetBrainsMono-Regular.ttf"
Fetch $jbUrl "$fontsDir\JetBrainsMono-Regular.ttf"

# ── IBM Plex Sans Regular (IBM, OFL) ────────────────────────────────────────
$plexUrl = "https://raw.githubusercontent.com/IBM/plex/master/packages/plex-sans/fonts/complete/ttf/IBMPlexSans-Regular.ttf"
Fetch $plexUrl "$fontsDir\IBMPlexSans-Regular.ttf"

Write-Host ""

$downloaded = (Get-ChildItem "$fontsDir\*.ttf" -ErrorAction SilentlyContinue).Count
if ($downloaded -gt 0) {
    Write-Host "$downloaded font(s) downloaded. Rebuild to embed them:"
    Write-Host "  cargo build --release"
} else {
    Write-Host "No fonts downloaded — app uses egui built-in fonts."
    Write-Host ""
    Write-Host "Manual download:"
    Write-Host "  Material Symbols Rounded -> https://fonts.google.com/icons (Download Family > Rounded)"
    Write-Host "  JetBrains Mono           -> https://www.jetbrains.com/lp/mono/"
    Write-Host "  IBM Plex Sans            -> https://www.ibm.com/plex/"
    Write-Host "  Save .ttf files to: $fontsDir"
}
