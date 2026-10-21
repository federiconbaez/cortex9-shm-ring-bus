//! Enrutador lock-free: N anillos SPSC de ingreso → M de egreso.
//! Estados: Idle → Drain → Scatter → Idle. Reenvío por lotes por destino.
//! Uso: router <cap> <n_in> <n_out> <prefijo> <mensajes_totales_esperados>
use shm_ring_bus::{unpack, Ring};
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    assert!(a.len() == 6, "uso: router <cap> <n_in> <n_out> <prefijo> <total>");
    let cap: u64 = a[1].parse().unwrap();
    let n_in: usize = a[2].parse().unwrap();
    let n_out: usize = a[3].parse().unwrap();
    let prefix = &a[4];
    let total: u64 = a[5].parse().unwrap();

    let mut ingress: Vec<Ring> = (0..n_in)
        .map(|i| Ring::open(&format!("{prefix}-in-{i}"), cap, true).unwrap())
        .collect();
    let mut egress: Vec<Ring> = (0..n_out)
        .map(|j| Ring::open(&format!("{prefix}-out-{j}"), cap, true).unwrap())
        .collect();

    let ready = std::env::temp_dir().join(format!("{prefix}.router-ready"));
    std::fs::write(&ready, b"1").ok();

    let mut buf = [0u64; 1024];
    let mut bins: Vec<Vec<u64>> = (0..n_out).map(|_| Vec::with_capacity(1024)).collect();
    let mut forwarded = 0u64;
    let t0 = Instant::now();
    let mut rr = 0usize;

    while forwarded < total {
        let i = rr % n_in;
        rr = rr.wrapping_add(1);
        let n = ingress[i].pop_batch(&mut buf);
        if n == 0 {
            std::hint::spin_loop();
            continue;
        }
        for b in bins.iter_mut() {
            b.clear();
        }
        for &m in &buf[..n] {
            let (dst, _) = unpack(m);
            bins[(dst as usize) % n_out].push(m);
        }
        for (j, batch) in bins.iter().enumerate() {
            if batch.is_empty() {
                continue;
            }
            let mut off = 0;
            while off < batch.len() {
                let w = egress[j].push_batch(&batch[off..]);
                if w == 0 {
                    std::hint::spin_loop();
                    continue;
                }
                off += w;
                forwarded += w as u64;
            }
        }
    }
    let dt = t0.elapsed().as_secs_f64();
    println!(
        "router forwarded={forwarded} drops=0 segundos={dt:.3} msg_por_seg={:.1}M n_in={n_in} n_out={n_out}",
        forwarded as f64 / dt / 1e6
    );
    let _ = std::fs::remove_file(&ready);
}
