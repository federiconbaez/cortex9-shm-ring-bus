# Benchmark (compartida, 20/10/2026, hora de Buenos Aires)

Mensajes de 8 bytes, cap 65536, lote 256 salvo indicación. Errores=0 en todas.

## Rust ↔ Rust
mensajes=100000000 lote=1 segundos=2.519 msg_por_seg=39.7M errores_de_secuencia=0
mensajes=300000000 lote=256 segundos=0.417 msg_por_seg=718.6M errores_de_secuencia=0

## Productor foráneo → consumidor Rust (30M, lote 256)
lang=rust mensajes=30000000 lote=256 segundos=0.039 msg_por_seg=764.1M errores=0
lang=c mensajes=30000000 lote=256 segundos=0.035 msg_por_seg=865.2M errores=0
lang=cpp mensajes=30000000 lote=256 segundos=0.030 msg_por_seg=1010.7M errores=0
lang=go mensajes=30000000 lote=256 segundos=0.049 msg_por_seg=618.2M errores=0
lang=python mensajes=30000000 lote=256 segundos=4.493 msg_por_seg=6.7M errores=0
