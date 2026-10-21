//! Consumidor: drena un egreso hasta idle prolongado o alcanzar max.
//! uso: route_cons <name> <cap> <max> <batch> <id>
use shm_ring_bus::{unpack, Ring};
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let name = &a[1];
    let cap: u64 = a[2].parse().unwrap();
    let max: u64 = a[3].parse().unwrap();
    let batch: usize = a[4].parse().unwrap();
    let id: u16 = a[5].parse().unwrap();
    let mut r = Ring::open(name, cap, false).unwrap();
    let mut out = vec![0u64; batch];
    let mut got = 0u64;
    let mut bad_dst = 0u64;
    let mut last = Instant::now();
    while got < max {
        let k = r.pop_batch(&mut out);
        if k == 0 {
            if last.elapsed().as_millis() > 500 && got > 0 {
                break; // drenado
            }
            std::hint::spin_loop();
            continue;
        }
        last = Instant::now();
        for &m in &out[..k] {
            let (dst, _) = unpack(m);
            if dst != id {
                bad_dst += 1;
            }
            got += 1;
        }
    }
    println!("cons id={id} recibidos={got} mal_destino={bad_dst}");
}
