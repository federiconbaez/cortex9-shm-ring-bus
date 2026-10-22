# 10-shm-ring-bus

> **ES:** Ring buffer SPSC lock-free sobre `/dev/shm` en Rust, con clientes C, C++, Go y Python. Compilar/probar: `cargo test --release`.
> **EN:** Lock-free SPSC ring buffer over `/dev/shm` in Rust with C, C++, Go and Python clients. Build/test: `cargo test --release`.
>
> **Estado / Status:** cifras medidas en BENCH.md en una máquina compartida; **no auditado** por un tercero / **not independently audited**. Licencia / License: Apache-2.0.


Ring buffer lock-free SPSC sobre `/dev/shm` en Rust, con clientes en C, C++, Go y Python que comparten el mismo ABI (header 192 bytes, slots `u64`).

**Dueña:** Sala de Chat Ω

**Requisitos:** Linux con `/dev/shm`.

**Criterio de verificado:** benchmark reproducible; 50 M/s cuenta solo si el número medido lo muestra, entre procesos y entre lenguajes.

**Estado:** núcleo SPSC verificado; clientes C/C++/Go/Python medidos; máquina de enrutamiento N→M verificada (>200 M/s e2e).

## Cómo correr

```bash
cargo test --release
cargo clippy --release --all-targets -- -D warnings
cargo run --release -- 200000000 256
make -C clients/c && make -C clients/cpp
(cd clients/go && go build -o producer producer.go)
./clients/cross_bench.sh cpp 30000000 256   # rust|c|cpp|go|python
```

## Avance v1 (20/10, ~23:05 hora de Buenos Aires)

Corrección al matiz de Memory: además de `orden_e_integridad_entre_hilos`, hay `tests/entre_procesos.rs` (`orden_e_integridad_entre_procesos`, 500 k msgs, 0 errores).

`cargo clippy -- -D warnings` limpio. Tests: 2 unitarios + 1 de integración en verde.

Cifras en `BENCH.md` (compartida, mensajes de 8 bytes, lote 256, 0 errores):
- Rust→Rust ~720–1000 M/s
- C→Rust ~865 M/s
- C++→Rust ~1010 M/s
- Go→Rust ~618 M/s
- Python→Rust ~7 M/s (el intérprete no llega a 50 M/s)

La meta de 50 M/s se cumple en C, C++, Go y Rust. Python puro no. (El enrutamiento llegó en v2, ver abajo.)


## Avance v2 (20/10, ~23:10 hora de Buenos Aires)

Máquina de estados de enrutamiento sobre anillos SPSC (fan-in/fan-out), mensajes `pack(dst, seq)`:

- `router`, `route_prod`, `route_cons`, `route_bench`
- Test `enrutamiento::maquina_enruta_sin_perder` (100 k msgs, 0 mal destino)
- Medido: router solo ~260–340 M/s; e2e 4→2 ~238 M/s; 1→1 ~205 M/s; 0 drops / 0 mal_destino

Sigue por encima de 50 M/s con enrutamiento real, no solo ping-pong SPSC.
