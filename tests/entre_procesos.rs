//! Integridad entre PROCESOS: productor hijo, consumidor padre.
use shm_ring_bus::Ring;
use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn orden_e_integridad_entre_procesos() {
    let name = format!("fnbr-itest-{}", std::process::id());
    let cap = 1u64 << 14;
    let n = 500_000u64;
    let batch = 64usize;
    let mut cons = Ring::open(&name, cap, true).unwrap();

    let prod = env!("CARGO_BIN_EXE_producer");
    let mut child = Command::new(prod)
        .args([&name, &cap.to_string(), &n.to_string(), &batch.to_string()])
        .spawn()
        .expect("spawn producer");

    let mut out = vec![0u64; batch];
    let mut expect = 0u64;
    let mut errores = 0u64;
    let t0 = Instant::now();
    while expect < n {
        let k = cons.pop_batch(&mut out);
        if k == 0 {
            assert!(t0.elapsed() < Duration::from_secs(30), "timeout esperando mensajes");
            std::hint::spin_loop();
            continue;
        }
        for v in &out[..k] {
            if *v != expect {
                errores += 1;
            }
            expect += 1;
        }
    }
    let status = child.wait().unwrap();
    Ring::unlink(&name);
    assert!(status.success(), "productor falló: {status}");
    assert_eq!(errores, 0, "errores de secuencia entre procesos");
}
