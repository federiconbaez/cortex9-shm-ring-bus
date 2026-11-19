#!/usr/bin/env bash
# Auditoría #10: líneas base extra (no exigidas por BENCH): pipe Go<->C++ (misma carga que el anillo Go<->C++) y pipe con espera activa en C.
set -u
cd "$(dirname "$0")/base"
OUT=../rtt-base-crudo.txt; : > $OUT
for r in 1 2 3 4 5; do
  echo "## rep=$r $(TZ=ART3 date '+%F %T ART') load=$(cut -d' ' -f1-3 /proc/loadavg) nproc=$(nproc)" >> $OUT
  timeout 120 ./pingpipe ./pongpipe >> $OUT 2>&1
  timeout 120 ./pipespin >> $OUT 2>&1
done
echo "## fin $(TZ=ART3 date '+%F %T ART') load=$(cut -d' ' -f1-3 /proc/loadavg)" >> $OUT
