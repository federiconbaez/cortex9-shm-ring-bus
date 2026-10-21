//! Benchmark entre procesos: `shm_ring_bus [mensajes] [lote]`.
use shm_ring_bus::Ring;
use std::time::Instant;
const CAP: u64 = 1 << 16;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() > 1 && a[1] == "--producer" {
        let (name, n, batch): (&str, u64, usize) = (&a[2], a[3].parse().unwrap(), a[4].parse().unwrap());
        let mut r = Ring::open(name, CAP, false).unwrap();
        let mut buf = vec![0u64; batch]; let mut next = 0u64;
        while next < n {
            let k = batch.min((n - next) as usize);
            for (i, slot) in buf.iter_mut().take(k).enumerate() { *slot = next + i as u64; }
            let mut off = 0;
            while off < k { let w = r.push_batch(&buf[off..k]); if w == 0 { std::hint::spin_loop(); } off += w; }
            next += k as u64;
        }
        return;
    }
    let n: u64 = a.get(1).map(|s| s.parse().unwrap()).unwrap_or(200_000_000);
    let batch: usize = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(256);
    let name = format!("fnbr-bench-{}", std::process::id());
    let mut r = Ring::open(&name, CAP, true).unwrap();
    let t0 = Instant::now();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--producer", &name, &n.to_string(), &batch.to_string()]).spawn().unwrap();
    let mut out = vec![0u64; batch]; let mut expect = 0u64; let mut errores = 0u64;
    while expect < n {
        let k = r.pop_batch(&mut out);
        if k == 0 { std::hint::spin_loop(); continue; }
        for v in &out[..k] { if *v != expect { errores += 1; } expect += 1; }
    }
    let dt = t0.elapsed().as_secs_f64();
    child.wait().unwrap();
    Ring::unlink(&name);
    println!("mensajes={n} lote={batch} segundos={dt:.3} msg_por_seg={:.1}M errores_de_secuencia={errores}", n as f64 / dt / 1e6);
}
