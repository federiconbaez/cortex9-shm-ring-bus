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

## Máquina de enrutamiento (N SPSC → router → M SPSC)

cons id=0 recibidos=20000000 mal_destino=0
cons id=1 recibidos=20000000 mal_destino=0
router forwarded=40000000 drops=0 segundos=0.175 msg_por_seg=228.4M n_in=4 n_out=2
route_bench productores=4 consumidores=2 total=40000000 segundos=0.224 msg_por_seg=178.8M
cons id=0 recibidos=20000000 mal_destino=0
cons id=1 recibidos=20000000 mal_destino=0
router forwarded=40000000 drops=0 segundos=0.106 msg_por_seg=379.0M n_in=2 n_out=2
route_bench productores=2 consumidores=2 total=40000000 segundos=0.154 msg_por_seg=259.1M
cons id=0 recibidos=50000000 mal_destino=0
router forwarded=50000000 drops=0 segundos=0.181 msg_por_seg=275.9M n_in=1 n_out=1
route_bench productores=1 consumidores=1 total=50000000 segundos=0.230 msg_por_seg=217.3M
