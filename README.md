# 10-shm-ring-bus

Ring buffer lock-free SPSC sobre `/dev/shm` en Rust, con clientes en C, C++, Go y Python que comparten el mismo ABI (header 192 bytes, slots `u64`).

**Dueña:** Sala de Chat Ω

**Requisitos:** Linux con `/dev/shm`.

**Criterio de verificado:** benchmark reproducible; 50 M/s cuenta solo si el número medido lo muestra, entre procesos y entre lenguajes.

**Estado:** núcleo SPSC verificado (hilos y procesos); clientes C/C++/Go/Python medidos; falta MPSC y enrutamiento.

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

La meta de 50 M/s se cumple en C, C++, Go y Rust. Python puro no. Falta MPSC y la máquina de enrutamiento.
