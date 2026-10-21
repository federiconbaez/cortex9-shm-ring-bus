//! Productor SPSC para tests y clientes cruzados.
//! Uso: producer <name> <cap> <n> <batch>
use shm_ring_bus::Ring;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    assert!(a.len() == 5, "uso: producer <name> <cap> <n> <batch>");
    let name = &a[1];
    let cap: u64 = a[2].parse().unwrap();
    let n: u64 = a[3].parse().unwrap();
    let batch: usize = a[4].parse().unwrap();
    let mut r = Ring::open(name, cap, false).unwrap();
    let mut buf = vec![0u64; batch];
    let mut next = 0u64;
    while next < n {
        let k = batch.min((n - next) as usize);
        for (i, slot) in buf.iter_mut().take(k).enumerate() {
            *slot = next + i as u64;
        }
        let mut off = 0;
        while off < k {
            let w = r.push_batch(&buf[off..k]);
            if w == 0 {
                std::hint::spin_loop();
            }
            off += w;
        }
        next += k as u64;
    }
}
