//! Productor para route_bench: escribe `pack(dst, seq)` en un ingress.
//! uso: route_prod <name> <cap> <n> <batch> <n_out> <prod_id>
use shm_ring_bus::{pack, Ring};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let name = &a[1];
    let cap: u64 = a[2].parse().unwrap();
    let n: u64 = a[3].parse().unwrap();
    let batch: usize = a[4].parse().unwrap();
    let n_out: u16 = a[5].parse().unwrap();
    let prod_id: u64 = a[6].parse().unwrap();
    let mut r = Ring::open(name, cap, false).unwrap();
    let mut buf = vec![0u64; batch];
    let mut next = 0u64;
    // seq global por productor: (prod_id << 32) | local  — cabe en 48 bits si local < 2^32
    while next < n {
        let k = batch.min((n - next) as usize);
        for (i, slot) in buf.iter_mut().take(k).enumerate() {
            let local = next + i as u64;
            let dst = ((local + prod_id) % n_out as u64) as u16;
            let seq = (prod_id << 32) | (local & 0xffff_ffff);
            *slot = pack(dst, seq);
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
