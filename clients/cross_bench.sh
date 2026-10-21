#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LANG=${1:?c|cpp|go|python|rust}
N=${2:-50000000}
BATCH=${3:-256}
CAP=$((1<<16))
NAME="fnbr-xbench-$$-$RANDOM"
export NAME CAP N BATCH LANG_LABEL="$LANG"
READY="/tmp/${NAME}.ready"
rm -f "$READY"
# start consumer first (creates ring)
"$ROOT/target/release/cross_cons" &
CPID=$!
# wait until ready file exists
for i in $(seq 1 200); do [[ -f "$READY" ]] && break; sleep 0.01; done
[[ -f "$READY" ]] || { kill $CPID; echo "consumer no listo"; exit 1; }
case "$LANG" in
  rust) "$ROOT/target/release/producer" "$NAME" "$CAP" "$N" "$BATCH" ;;
  c)    "$ROOT/clients/c/producer" "$NAME" "$CAP" "$N" "$BATCH" ;;
  cpp)  "$ROOT/clients/cpp/producer" "$NAME" "$CAP" "$N" "$BATCH" ;;
  go)   "$ROOT/clients/go/producer" "$NAME" "$CAP" "$N" "$BATCH" ;;
  python) python3 "$ROOT/clients/python/producer.py" "$NAME" "$CAP" "$N" "$BATCH" ;;
  *) echo "lang?"; kill $CPID; exit 2 ;;
esac
wait $CPID
