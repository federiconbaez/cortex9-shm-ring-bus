#!/usr/bin/env bash
# H6: el productor muere (SIGKILL) con el anillo a medias; el consumidor no lo detecta y gira al 100 % para siempre.
cd "$(dirname "$0")/../../.."
./target/release/shm_ring_bus 3000000000 256 & P=$!
sleep 0.3
C=$(pgrep -P $P -f -- '--producer')
echo "consumidor pid=$P productor pid=$C"; kill -9 $C
sleep 5
if kill -0 $P 2>/dev/null; then echo "5 s después del SIGKILL el consumidor sigue vivo: $(ps -o pcpu=,stat= -p $P) (%CPU, estado)"; kill -9 $P; echo "matado a mano"; else echo "consumidor terminó"; fi
ls /dev/shm | grep -c fnbr-bench && echo "segmento /dev/shm huérfano queda (no hay limpieza ante crash)"; rm -f /dev/shm/fnbr-bench-$P
