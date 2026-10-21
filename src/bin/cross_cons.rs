//! Consumidor Rust para bench cruzado. Env: NAME CAP N BATCH LANG_LABEL
//! Crea el ring y espera al productor externo.
use shm_ring_bus::Ring;
use std::time::Instant;

fn main() {
    let name = std::env::var("NAME").expect("NAME");
    let cap: u64 = std::env::var("CAP").expect("CAP").parse().unwrap();
    let n: u64 = std::env::var("N").expect("N").parse().unwrap();
    let batch: usize = std::env::var("BATCH").unwrap_or_else(|_| "256".into()).parse().unwrap();
    let label = std::env::var("LANG_LABEL").unwrap_or_else(|_| "?".into());
    let mut r = Ring::open(&name, cap, true).unwrap();
    // touch a ready file so producer can wait
    let ready = std::env::temp_dir().join(format!("{name}.ready"));
    std::fs::write(&ready, b"1").ok();
    let t0 = Instant::now();
    let mut out = vec![0u64; batch];
    let mut expect = 0u64;
    let mut err = 0u64;
    while expect < n {
        let k = r.pop_batch(&mut out);
        if k == 0 {
            std::hint::spin_loop();
            continue;
        }
        for v in &out[..k] {
            if *v != expect {
                err += 1;
            }
            expect += 1;
        }
    }
    let dt = t0.elapsed().as_secs_f64();
    let _ = std::fs::remove_file(&ready);
    Ring::unlink(&name);
    println!(
        "lang={label} mensajes={n} lote={batch} segundos={dt:.3} msg_por_seg={:.1}M errores={err}",
        n as f64 / dt / 1e6
    );
    if err > 0 {
        std::process::exit(1);
    }
}
