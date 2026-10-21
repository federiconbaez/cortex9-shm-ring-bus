#!/usr/bin/env bash
# RTT Go -> C++ -> Go por anillos shm. Uso: lat/correr.sh [n] [calentamiento]
set -euo pipefail
cd "$(dirname "$0")"
N=${1:-200000}; W=${2:-10000}; CAP=1024; TOT=$((N+W))
g++ -O2 -std=c++17 -o pong pong.cpp -lrt
(cd . && go build -o ping ping.go)
rm -f /dev/shm/lat_ping.ready
./ping lat_ping lat_pong $CAP $N $W &
PID=$!
while [ ! -e /dev/shm/lat_ping.ready ]; do sleep 0.01; done
./pong lat_ping lat_pong $CAP $TOT
wait $PID
