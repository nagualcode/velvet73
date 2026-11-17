#!/bin/bash
# H2 re fundamental (F3, DI) and Twin speaker volts for a chord, by MIDI velocity.
for v in 40 64 80 100 127; do
  vel=$(python3 -c "print($v/127)")
  ./target/release/velvet73-render renders/t.wav renders/di.preset f3 1.5 48000 $vel 2>/dev/null
  h=$(python3 -I tools/spectrum.py renders/t.wav 174.6 0.3 0.15 | tail -1 | awk '{print $4, $5}')
  tip=$(VELVET_PROBE=1 ./target/release/velvet73-render renders/t.wav renders/di.preset f3 0.5 48000 $vel 2>&1 | sed -n 3p | sed 's/.*tipF3= *\([0-9.]*\)mm.*/\1/')
  spk=$(VELVET_PROBE=1 ./target/release/velvet73-render renders/t.wav factory/Stage/Factory.preset chord 1.0 48000 $vel 2>&1 | awk -F'amp=' 'NF>1{split($2,a,"V"); if(a[1]+0>m)m=a[1]+0} END{print m}')
  echo "vel $v: tip F3 ${tip} mm | H2 H3 (dB re f0): $h | Twin chord peak ${spk} V"
done
