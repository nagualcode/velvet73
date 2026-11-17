#!/bin/bash
# Velvet73 · macOS installer. Jardim Sonoro, 2026.
set -e
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AU="$HOME/Library/Audio/Plug-Ins/Components"
VST3="$HOME/Library/Audio/Plug-Ins/VST3"
CLAP="$HOME/Library/Audio/Plug-Ins/CLAP"
PRESETS="$HOME/Library/Audio/Presets/Jardim Sonoro/Velvet73"
mkdir -p "$AU" "$VST3" "$CLAP" "$PRESETS"
echo "Velvet73: tuning seventy-three tines…"
for b in Velvet73.component:"$AU" Velvet73.vst3:"$VST3" Velvet73.clap:"$CLAP"; do
  name="${b%%:*}"; dest="${b#*:}"
  if [ -d "$DIR/$name" ]; then
    rm -rf "$dest/$name"
    cp -R "$DIR/$name" "$dest/"
    xattr -cr "$dest/$name" 2>/dev/null || true
    echo "  -> $dest/$name"
  fi
done
if [ -d "$DIR/Presets" ]; then cp -R "$DIR"/Presets/* "$PRESETS/" 2>/dev/null || true; fi
killall -9 AudioComponentRegistrar 2>/dev/null || true
echo "Done. Restart your DAW and play it in."
