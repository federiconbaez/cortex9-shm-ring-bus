#!/usr/bin/env bash
# Auditoría #10: RTT anillo vs pipe vs socket Unix (latencia.c) y Go<->C++ (clients/lat), 5 reps cada uno.
set -u
cd "$(dirname "$0")/../.."
OUT=resultados/auditoria/rtt-crudo.txt
: > $OUT
for r in 1 2 3 4 5; do
  echo "## rep=$r latencia.c $(TZ=ART3 date '+%F %T ART') load=$(cut -d' ' -f1-3 /proc/loadavg) nproc=$(nproc)" >> $OUT
  /tmp/latencia-audit10 >> $OUT 2>&1
  echo "## rep=$r go<->c++ $(TZ=ART3 date '+%F %T ART') load=$(cut -d' ' -f1-3 /proc/loadavg)" >> $OUT
  rm -f /dev/shm/lat_ping.ready
  (cd clients/lat; timeout 120 ./ping lat_ping lat_pong 1024 200000 10000 & PID=$!; while [ ! -e /dev/shm/lat_ping.ready ]; do sleep 0.01; done; timeout 120 ./pong lat_ping lat_pong 1024 210000; wait $PID) >> $OUT 2>&1
done
echo "## fin $(TZ=ART3 date '+%F %T ART') load=$(cut -d' ' -f1-3 /proc/loadavg)" >> $OUT
