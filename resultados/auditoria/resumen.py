#!/usr/bin/env python3
"""Resumen de los crudos de la auditoría #10: rango [min–max] y mediana por métrica."""
import re, statistics as st, collections
d = collections.defaultdict(list)
for f in ["rtt-crudo.txt", "rtt-base-crudo.txt"]:
    for l in open(f):
        m = re.match(r"(\S+(?: \S+)?)\s+n=\d+ p50=([\d.]+)us p99=([\d.]+)us p99.9=([\d.]+)us max=[\d.]+us errores=(\d+)", l.strip())
        if m:
            k = m.group(1).strip()
            for q, v in zip(("p50", "p99", "p999", "err"), m.groups()[1:]):
                d[(k, q)].append(float(v))
for l in open("throughput-crudo.txt"):
    m = re.search(r"mensajes=(\d+) lote=(\d+) segundos=\S+ msg_por_seg=([\d.]+)M errores_de_secuencia=(\d+)", l)
    if m: d[(f"rust-rust n={m.group(1)} lote={m.group(2)}", "Mps")].append(float(m.group(3))); d[(f"rust-rust n={m.group(1)} lote={m.group(2)}", "err")].append(float(m.group(4)))
    m = re.search(r"lang=(\w+) .*msg_por_seg=([\d.]+)M errores=(\d+)", l)
    if m: d[(f"cross {m.group(1)}", "Mps")].append(float(m.group(2))); d[(f"cross {m.group(1)}", "err")].append(float(m.group(3)))
    m = re.search(r"router forwarded=(\d+) .*msg_por_seg=([\d.]+)M n_in=(\d+) n_out=(\d+)", l)
    if m: d[(f"router {m.group(3)}->{m.group(4)}", "Mps")].append(float(m.group(2)))
    m = re.search(r"route_bench productores=(\d+) consumidores=(\d+) .*msg_por_seg=([\d.]+)M", l)
    if m: d[(f"e2e {m.group(1)}->{m.group(2)}", "Mps")].append(float(m.group(3)))
    m = re.search(r"cons id=\d+ recibidos=(\d+) mal_destino=(\d+)", l)
    if m: d[("route cons", "mal_destino")].append(float(m.group(2)))
for (k, q), v in d.items():
    print(f"{k:32s} {q:12s} n={len(v)} min={min(v):g} med={st.median(v):g} max={max(v):g}")
# cocientes por rep (mismo rep, misma corrida de latencia.c)
def per(k, q): return d[(k, q)]
for q in ("p50", "p99"):
    r = [a / b for a, b in zip(per("anillo_shm", q), per("pipe", q))]
    print(f"cociente anillo_shm/pipe {q}: " + " ".join(f"{x:.3f}" for x in r) + f"  max={max(r):.3f}")
    r = [a / b for a, b in zip(per("go->c++->go anillo", q), per("go->c++->go pipe", q))]
    print(f"cociente go-c++ anillo/pipe go-c++ {q} (reps emparejadas por índice, corridas distintas): " + " ".join(f"{x:.3f}" for x in r) + f"  max={max(r):.3f}")
    r = [a / b for a, b in zip(per("anillo_shm", q), per("pipe_spin", q))]
    print(f"cociente anillo_shm/pipe_spin {q}: " + " ".join(f"{x:.3f}" for x in r) + f"  max={max(r):.3f}")
