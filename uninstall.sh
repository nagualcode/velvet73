#!/bin/bash
# Velvet73 · macOS uninstaller. The dampers come down; the tines fall still.
rm -rf "$HOME/Library/Audio/Plug-Ins/Components/Velvet73.component" \
       "$HOME/Library/Audio/Plug-Ins/VST3/Velvet73.vst3" \
       "$HOME/Library/Audio/Plug-Ins/CLAP/Velvet73.clap" \
       "$HOME/Library/Audio/Presets/Jardim Sonoro/Velvet73"
killall -9 AudioComponentRegistrar 2>/dev/null || true
echo "Velvet73 has gone quiet."
