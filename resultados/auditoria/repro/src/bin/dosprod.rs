//! H3: la API segura permite dos productores sobre el mismo SPSC (no hay registro de rol):
//! carrera de datos sobre los slots y mensajes perdidos, sin error.
use shm_ring_bus::Ring;
fn main() {
    let name = format!("audit10-dosprod-{}", std::process::id());
    let mut cons = Ring::open(&name, 1 << 10, true).unwrap();
    const N: u64 = 2_000_000;
    let hs: Vec<_> = (0..2u64).map(|id| {
        let name = name.clone();
        std::thread::spawn(move || {
            let mut p = Ring::open(&name, 1 << 10, false).unwrap();
            let mut i = 0;
            while i < N { let v = (id << 32) | i; if p.push_batch(&[v]) == 1 { i += 1; } }
        })
    }).collect();
    let (mut got, mut out) = (0u64, [0u64; 256]);
    let mut seen = [0u64; 2];
    let t0 = std::time::Instant::now();
    while t0.elapsed().as_secs() < 5 {
        let n = cons.pop_batch(&mut out);
        for v in &out[..n] { seen[((v >> 32) & 1) as usize] += 1; }
        got += n as u64;
        if hs.iter().all(|h| h.is_finished()) && n == 0 { break; }
    }
    for h in hs { let _ = h.join(); }
    println!("enviados={} recibidos={} (por productor {:?}) perdidos={}", 2 * N, got, seen, (2 * N) as i64 - got as i64);
    Ring::unlink(&name);
}
