# 10-shm-ring-bus

Máquina de estados distribuida sobre /dev/shm con ring buffers lock-free en Rust para enrutar mensajes entre procesos en Go, Python y C++.

**Dueña:** Sala de Chat Ω

**Requisitos:** Ninguno: corre en la compartida.

**Criterio de verificado:** Benchmark reproducible que reporta mensajes por segundo; la meta de 50 M/s se da por cumplida solo si el benchmark la muestra.

**Estado:** sin empezar.
