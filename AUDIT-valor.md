# AUDIT-valor — sistema 10 (`shm-ring-bus`)

**Auditor:** Cortex-9 Nexus (Grok Bot) · **Fecha:** 2026-11-09, 07:13–07:27 ART (America/Buenos_Aires, UTC-03:00)
**Commit auditado:** `acbd2ac` (padre `176a06e`). Solo box; no se tocó `host-de-pruebas` ni máquinas de Federico.
**Clon aparte:** `/workspace/audit-clones/10-shm-ring-bus-acbd2ac`. Crudos en `resultados/auditoria/`.

## Umbral #10 (literal)

Fuente: `/workspace/mycelium/proyectos/objetivo-valor-10-sistemas.md`, tabla «Umbrales de éxito (Research, 2026-10-21 00:02 ART)»,
fijada en el commit `7c3912d` de `mycelium` (2026-10-21 00:02:45 ART):

> | 10 | RTT p50 ≤ **0,20×** pipe (misma carga, 0 errores) **y** p99 ≤ **0,25×** pipe; si se mide en app real, además p99 de la app ≤ pipe | Solo gana en msg/s sintéticos sin latencia, o con errores |

Experimento acotado por la dueña (`c88d64a` de `mycelium`, 2026-10-20 23:49:28 ART):
«#10 (Sala): parte que se mide acá = RTT p50/p99/p99,9 entre Go y C++ contra pipe y socket Unix con la misma carga.»
Criterio original del repo (`83d0b51`, README, 2026-10-20 22:43 ART): «Benchmark reproducible que reporta mensajes por segundo;
la meta de 50 M/s se da por cumplida solo si el benchmark la muestra.»

**Orden (no es pre-registro numérico):** la latencia en C (`a935c77`, 23:55:44 ART) y la de Go↔C++ (`eb26f30`, 23:56:40 ART) se commitearon
**antes** de que se fijaran los números 0,20×/0,25× (`7c3912d`, 00:02:45 ART). Ese mismo commit dice «el 10 ya cumple […] RTT ≪ pipe».
El umbral numérico es post-hoc. Lo que sí precede a la medición es el diseño del experimento (`c88d64a`). El margen es amplio
(ver abajo), así que el orden no cambia el veredicto. Sí queda como falla de proceso.

## Veredicto

**PASA el umbral #10 (literal), con salvedades.** En el rerun (5 reps × 200 k RTT, 0 errores):
- C: anillo/pipe **p50 0,049–0,060×** (≤ 0,20) y **p99 0,034–0,057×** (≤ 0,25).
- Go↔C++ contra un pipe Go↔C++ con la misma carga (línea base que BENCH.md no midió; la medí yo): **p50 ≤ 0,073×** y **p99 ≤ 0,066×**.

No hay «app real», así que la cláusula condicional no aplica y el nivel C no se alcanza. **B+ se confirma, no más.**

Salvedades que pesan:
1. La ventaja sale sobre todo de no dormir. Contra un pipe con espera activa (`O_NONBLOCK` + spin; informativo, no lo exige el umbral)
   el cociente es p50 0,21–0,27× y p99 0,16–0,28×: **no pasaría 0,20× en p50 en ninguna de las 5 reps.** BENCH.md ya lo advierte
   («la ventaja se paga en CPU»).
2. Hay fallas de corrección reales fuera del camino feliz (reinicio de un par, re-creación del segmento, crash, par hostil):
   corrupción o pérdida **silenciosa**. Ver §5. No afectan al número medido, pero sí a «app real».
3. La meta original de 50 M/s se cumple solo con lote 256. Con lote 1, Rust↔Rust da 39,7 M/s (BENCH) y **19,5–24,8 M/s** en el rerun.

## 1. Integridad
- HEAD `acbd2ac741760ae450c6277df40dcae344795b8a`, padre `176a06e`. Árbol limpio. Ignorados: solo binarios de clientes
  (`clients/c/{producer,consumer_bench}`, `clients/cpp/producer`, `clients/go/producer`, `clients/lat/{ping,pong}`).
- `git archive HEAD | sha256sum` = `540da7e8e25f07ff7a6aed8fbbf274797f9571f1b740d411e4717373a421e305` (igual en el repo y en el clon).
- **Historial parcialmente reescrito** (contra lo que dice el registro: «10 sin reescribir»). Según el reflog, `136b864` (licencia) y
  `bcc6552` (README) pasaron por `reset` a `176a06e`/`acbd2ac`, con autor «Federico Nicolas Báez Rizzo» y fecha 2026-10-22 00:21:57 ART.
  `git diff bcc6552 acbd2ac` está vacío: el **árbol es idéntico**. Cambiaron solo el autor, el mensaje y la fecha. Los commits
  `83d0b51`…`eb26f30` conservan su hash.
- `git diff eb26f30 acbd2ac`: solo `LICENSE` y `README.md`. **Ningún cambio de código después de las mediciones.** ✔
- Remoto `github` configurado (`federiconbaez/cortex9-shm-ring-bus`). No lo consulté.

## 2. Tests y clippy (`resultados/auditoria/clippy-test.txt`)
- `cargo clippy --all-targets -- -D warnings`: limpio. Con `--release`: limpio.
- `cargo test --release`: **4/4** (2 unitarios + `maquina_enruta_sin_perder` + `orden_e_integridad_entre_procesos`). ✔
- Los tests no cubren: secuencia por productor en el enrutamiento (solo destino y total), reinicio o crash de un par, re-creación,
  header corrupto. Ninguno de los hallazgos de §5 lo detecta un test.

## 3. Rerun (clon aparte, mismos parámetros, 5 reps)
Entorno (`entorno.txt`): kernel 6.12.113+, **nproc 8** (Xeon, KVM, 1 hilo por núcleo), rustc 1.99.1, gcc 14.2, go 1.27.3, Python 3.13.5.
**C++ compilado con clang 22.1.8 (vía zig 0.17.1)**, no con g++: en la box no hay g++ y `apt` falla (500 en el espejo). Los flags son los mismos.
**Contención:** load average 1,15–2,55 durante las corridas, con un proceso `node` ajeno al 100 % de un núcleo. Box compartida.
Scripts: `run_lat.sh`, `run_base.sh`, `run_tput.sh`. Resumen: `python3 resumen.py` → `resumen.txt`.

Hubo un primer intento a las 07:15 ART que abortó por un error de ruta en mi script (no encontraba `pong`). Su única línea de `latencia.c`
(anillo p50 0,48 / p99 0,61 / p99,9 6,20 µs; pipe 7,83 / 10,40; socket 7,21 / 13,56) quedó fuera del crudo porque el rerun lo sobrescribió.
Es coherente con el rerun, salvo la p99,9 del anillo.

### Tabla: reclamado contra rerun (rango min–max de 5 reps; latencias en µs, throughput en M msg/s)

| Medida (parámetros de BENCH.md) | BENCH.md / README | Rerun auditoría | ¿Reproduce? |
|---|---|---|---|
| RTT anillo shm C p50 / p99 / p99,9 | 0,44–0,49 / 0,56–0,62 / 0,69–0,88 (3 corridas) | 0,38–0,45 / 0,50–0,59 / 0,73–0,98 | ✔ |
| RTT pipe C p50 / p99 / p99,9 | 8,2–8,3 / 10,5–12,3 / 23,9–27,7 | 7,44–7,69 / 10,06–15,17 / 16,51–27,37 | ✔ (p50 algo menor) |
| RTT socket Unix C p50 / p99 / p99,9 | 7,7–7,9 / 9,9–10,5 / 17,7–19,9 | 7,14–7,39 / 9,47–11,15 / 15,39–16,91 | ✔ |
| **anillo/pipe C p50 · p99** | «13–15× menos» (≈0,05–0,07×) | **0,049–0,060× · 0,034–0,057×** | ✔ **pasa (≤0,20 · ≤0,25)** |
| RTT anillo Go↔C++ p50 / p99 / p99,9 | 0,54–0,61 / 0,66–0,77 / 0,96–1,14 | 0,52–0,60 / 0,68–0,72 / 0,90–1,22 (+1 outlier, ver nota) | ✔ |
| RTT **pipe Go↔C++** (misma carga) | **no medido** (se comparó contra el pipe en C) | 8,07–8,59 / 10,83–17,61 / 20,61–29,23 | medido por la auditoría |
| **anillo/pipe Go↔C++ p50 · p99** | — | **0,062–0,073× · 0,061–0,066×** (sin el outlier) | ✔ **pasa** |
| pipe con espera activa (C), informativo | — | 1,60–1,87 / 2,00–3,04 / 2,72–7,69 | anillo/pipe_spin p50 0,21–0,27×, p99 0,16–0,28× |
| Rust↔Rust 100 M, lote 1 | 39,7 | **19,5–24,8** | ✗ rerun 38–51 % por debajo; < 50 M/s en ambos |
| Rust↔Rust 300 M, lote 256 | 718,6 (README «~720–1000») | 1006,5–1297,9 | ✔ (supera) |
| Rust→Rust 30 M, lote 256 | 764,1 | 849,2–1211,5 | ✔ |
| C→Rust | 865,2 | 1143,7–1241,9 | ✔ |
| C++→Rust | 1010,7 | 1177,2–1274,0 (clang) | ✔ |
| Go→Rust | 618,2 | 802,0–874,7 | ✔ |
| Python→Rust | 6,7 | 10,9–12,0 | ✔ (< 50 M/s, como dice el README) |
| router 4→2 / 2→2 / 1→1 | 228,4 / 379,0 / 275,9 (README: «~260–340») | 345,9–419,7 / 292,5–371,7 / 272,2–332,1 | ✔ |
| e2e route_bench 4→2 / 2→2 / 1→1 | 178,8 / 259,1 / 217,3 (README: «4→2 ~238; 1→1 ~205») | 243,8–277,5 / 215,7–256,7 / 215,0–251,2 | ✔ |
| errores de secuencia / mal_destino | 0 | 0 en todas (25 consumidores de ruteo, 0 mal_destino) | ✔ |

Notas de la tabla:
- **Outlier Go↔C++ (rep 3):** p50 0,09 / p99 0,11 µs con 0 errores. No lo pude reproducir fijando afinidad
  (`rtt-afinidad-crudo.txt`: CPU 0 contra CPU 1…7 da p50 0,59–0,65 en todos los pares). Hipótesis no verificada: en ese momento el
  hipervisor ubicó las dos vCPU en núcleos hermanos. Solo baja el cociente; si se excluye, el veredicto no cambia.
- Las cifras del README (v1 «~720–1000», v2 «~260–340», «~238», «~205») **no coinciden con BENCH.md** y no tienen crudo. El rerun queda por
  encima de todas, así que no hay sobre-reclamo, pero son números sin fuente.
- La fila #10 de `objetivo-valor` («ya: 29,5 vs 2,6/1,5 M/s») **no tiene fuente** en el repo ni en `mycelium`. No la pude verificar.
- `route_bench`: su `segundos` incluye un `sleep(50 ms)` fijo y el spawn de procesos, así que el e2e queda subestimado.
  `router … drops=0` es un **literal impreso, no una medición**: el router reintenta hasta que entra, así que no puede descartar.

## 4. Lo que el umbral exige y lo que falta
- «Misma carga, 0 errores»: ✔ en C (`latencia.c` mide los tres canales en el mismo binario). En Go↔C++, BENCH.md comparó contra el pipe **en C**,
  no contra uno con la misma carga. La línea base faltante la medí (`base/pingpipe.go` + `base/pongpipe.cpp`) y también pasa.
- «Si se mide en app real…»: no hay app real (la fila del objetivo pedía «App real Go/Python/C++»). Queda **sin medir**. Nivel C no alcanzado.
- Python no entra en el RTT: el experimento acotado es Go↔C++.
- No se midieron socket Unix Go↔C++, eventfd/futex ni crossbeam. El umbral no los pide.

## 5. Corrección del ring (repros en `resultados/auditoria/repro/`, salida en `repro-salida.txt`)
Núcleo SPSC (Rust `src/lib.rs` y C `clients/c/ring.h`), revisado a mano: el productor escribe los slots en forma plana y después hace un
`store(head, Release)`; el consumidor hace `load(head, Acquire)`, lee y después `store(tail, Release)`; el productor hace `load(tail, Acquire)`
antes de reutilizar. **El ordenamiento es correcto.** ABA: no aplica (contadores monotónicos de 64 bits; a 1,2·10⁹ msg/s el desborde tarda unos 487 años).
El índice envuelve con una máscara (cap potencia de 2, `assert!`). Con productor o consumidor lento hay contrapresión (`push` devuelve 0)
y no hay pérdida.
- **TSan sobre `ring.h`** (`tsan_ring.c`, hilos sobre una sola proyección, 2 M mensajes): 0 reportes, 0 errores. **Control positivo**
  (`head` publicado con `relaxed`): TSan reporta la carrera en `ring_pop`. ✔
- No corrí loom, Miri ni TSan en Rust: `Ring::open` hace `mmap` por handle (TSan no ve entre proyecciones distintas), Miri no emula
  `shm_open`/`mmap` compartido y loom exige reescribir con sus atómicos. Queda fuera por costo.

Hallazgos (todos reproducidos en `acbd2ac`):

| # | Hallazgo | Repro | Resultado | Severidad |
|---|---|---|---|---|
| H1 | **Reinicio de un par:** `open(create=false)` arranca con `local_head=0` aunque el `head` compartido avanzó; no hay resincronización | `reattach` | Después de 5 mensajes y un productor reiniciado que escribe 1 (`200`), el consumidor entrega **8** (`[0,0,0,200,101,102,103,104]`) y vuelve a entregar 8 en el siguiente pop. Corrupción silenciosa | alta |
| H2 | **Re-creación sin `O_EXCL`:** `open(create=true)` sobre un nombre vivo pone `head/tail=0`; con `head>cap`, `free` envuelve | `recreate` | Con cap 8, el productor mete **16** sin que el consumidor lea; el consumidor recibe `24..31` dos veces y `16..23` se pierde. `latencia.c` (`lat_a`/`lat_b`) y `clients/lat` (`lat_ping`/`lat_pong`) usan **nombres fijos**: dos corridas simultáneas en la box compartida se pisan | alta |
| H3 | La API segura permite **dos productores** (o dos consumidores) en un SPSC: `Ring` es `Send` y `open` es `safe` | `dosprod` | 4 M enviados → el consumidor «recibe» **≈2,14·10⁹** (el `head` retrocede y `avail` envuelve). Carrera de datos sobre los slots, que en Rust seguro es UB | media |
| H4 | Carrera de arranque: el par abre entre `shm_open(O_CREAT)` y `ftruncate` | `sigbus` | Objeto de 0 bytes → `mmap` funciona → la lectura de `cap` da **SIGBUS** (rc 135) | media-baja |
| H5 | El header compartido no se valida (`head−tail ≤ cap`) | `hostil` | Con `head=10⁶` y cap 8, el consumidor acepta 64 «mensajes» viejos. Sin acceso fuera de rango (la máscara lo acota) | media (si un par no es de confianza) |
| H6 | **Crash con el ring a medias:** no hay latido, timeout ni limpieza | `crash.sh` | Productor con SIGKILL → 5 s después el consumidor (`shm_ring_bus`) sigue girando al **100 % de CPU** y el segmento `/dev/shm` queda huérfano | media |
| H7 | Clientes: el productor Go (`clients/go/producer.go`, el del 618 M/s) lee y escribe `head`/`tail` **sin atómicos**; Python sin barreras | lectura | Funciona en x86-TSO; según los modelos de memoria de Go y de C/C++ es una carrera de datos. **No portable a arm64** | baja en x86 |

## 6. Condiciones para subir de B+ a C
1. App real Go/Python/C++ con p99 de la app ≤ pipe, con un umbral **commiteado antes** de medir (este no lo fue).
2. Arreglar H1–H6: nombre exclusivo (`O_EXCL`) o versión/época en el header; resincronizar `local_*` desde el header al abrir; validar
   `head−tail ≤ cap`; `ftruncate` antes de publicar (o abrir con reintento sobre `fstat`); detectar un par muerto (pid/época + timeout).
   Agregar tests de cada uno.
3. Reportar también contra una línea base con espera activa (pipe/eventfd en spin) y el costo en CPU por mensaje.
4. Crudo versionado de cada cifra del README. Hoy las de v1/v2 no tienen fuente.

## Caveats
- Una sola tanda de 5 reps en la box compartida (load hasta 2,55, un núcleo ocupado por otro proceso), sin aislamiento de CPU.
- C++ compilado con clang/zig y no con g++. El throughput C++ y `pong` pueden diferir del original.
- El rerun de throughput mide ventanas de 25–40 ms (30 M msgs) que incluyen el arranque del productor: son sensibles al ruido.
- No reproduje el número de la fila del objetivo «29,5 vs 2,6/1,5 M/s» (sin fuente).
- No encontré instrucciones ocultas en los archivos, commits ni salidas revisados.
