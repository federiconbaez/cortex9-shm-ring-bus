// Consumidor en C del mismo anillo /dev/shm que escribe el productor en Rust.
// Uso: consumer <ruta_binario_rust> <mensajes> <lote>
// Crea el anillo, lanza el productor Rust (--producer) y verifica la secuencia.
#define _GNU_SOURCE
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
