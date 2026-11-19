#!/usr/bin/env bash
# Auditoría #10: re-corrida de los benches de throughput de BENCH.md con los mismos parámetros, 5 reps.
set -u
cd "$(dirname "$0")/../.."
OUT=resultados/auditoria/throughput-crudo.txt; : > $OUT
stamp(){ echo "## $1 $(TZ=ART3 date '+%F %T ART') load=$(cut -d' ' -f1-3 /proc/loadavg) nproc=$(nproc)" >> $OUT; }
for r in 1 2 3 4 5; do
  stamp "rep=$r rust-rust"
  timeout 120 ./target/release/shm_ring_bus 100000000 1 >> $OUT 2>&1
  timeout 120 ./target/release/shm_ring_bus 300000000 256 >> $OUT 2>&1
  for l in rust c cpp go python; do stamp "rep=$r cross $l"; timeout 120 ./clients/cross_bench.sh $l 30000000 256 >> $OUT 2>&1; done
  stamp "rep=$r route"
  timeout 120 ./target/release/route_bench 4 2 10000000 256 >> $OUT 2>&1
  timeout 120 ./target/release/route_bench 2 2 20000000 256 >> $OUT 2>&1
  timeout 120 ./target/release/route_bench 1 1 50000000 256 >> $OUT 2>&1
done
stamp fin
