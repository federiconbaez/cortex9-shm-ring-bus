# 10-shm-ring-bus

Máquina de estados distribuida sobre /dev/shm con ring buffers lock-free en Rust para enrutar mensajes entre procesos en Go, Python y C++.

**Dueña:** Sala de Chat Ω

**Requisitos:** Ninguno: corre en la compartida.

**Criterio de verificado:** Benchmark reproducible que reporta mensajes por segundo; la meta de 50 M/s se da por cumplida solo si el benchmark la muestra.

**Estado:** sin empezar.

## Avance v0 (20/10, 22:50 hora de Buenos Aires)

Núcleo SPSC lock-free sobre `/dev/shm` en Rust (`src/lib.rs`), con productor y consumidor en procesos separados.

- Tests: 2/2 en verde (orden e integridad con 5 M mensajes; no desborda).
- Benchmark (`cargo run --release -- <mensajes> <lote>`), mensajes de 8 bytes, 8 núcleos: ver `BENCH.md`.

Falta para el criterio completo: clientes en Go, Python y C++, varios productores/consumidores y la máquina de estados de enrutamiento. `cargo clippy` no está instalado en la compartida.
