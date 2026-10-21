//! Benchmark de la máquina de enrutamiento: P productores → router → C consumidores.
//! Uso: route_bench <productores> <consumidores> <msgs_por_productor> <lote>
use shm_ring_bus::{pack, unpack, Ring};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let n_in: usize = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
    let n_out: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(2);
    let per: u64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(5_000_000);
    let batch: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(256);
    let cap: u64 = 1 << 16;
    let total = per * n_in as u64;
    let prefix = format!("fnbr-rt-{}", std::process::id());

    // limpiar leftovers
    for i in 0..n_in {
        Ring::unlink(&format!("{prefix}-in-{i}"));
    }
    for j in 0..n_out {
        Ring::unlink(&format!("{prefix}-out-{j}"));
    }

    let router_bin = std::env::current_exe()
        .unwrap()
        .with_file_name("router");
    let prod_bin = std::env::current_exe()
        .unwrap()
        .with_file_name("route_prod");

    let mut router = Command::new(&router_bin)
        .args([
            &cap.to_string(),
            &n_in.to_string(),
            &n_out.to_string(),
            &prefix,
            &total.to_string(),
        ])
        .spawn()
        .expect("router");

    let ready = std::env::temp_dir().join(format!("{prefix}.router-ready"));
    for _ in 0..500 {
        if ready.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(ready.exists(), "router no listo");

    // consumidores primero (abren egress en modo open false — el router ya creó)
    let cons_bin = std::env::current_exe()
        .unwrap()
        .with_file_name("route_cons");
    let mut consumers: Vec<Child> = (0..n_out)
        .map(|j| {
            Command::new(&cons_bin)
                .args([
                    &format!("{prefix}-out-{j}"),
                    &cap.to_string(),
                    &total.div_ceil(n_out as u64).to_string(), // upper bound; cons para por idle
                    &batch.to_string(),
                    &j.to_string(),
                ])
                .spawn()
                .expect("cons")
        })
        .collect();

    let t0 = Instant::now();
    let mut producers: Vec<Child> = (0..n_in)
        .map(|i| {
            Command::new(&prod_bin)
                .args([
                    &format!("{prefix}-in-{i}"),
                    &cap.to_string(),
                    &per.to_string(),
                    &batch.to_string(),
                    &n_out.to_string(),
                    &i.to_string(),
                ])
                .spawn()
                .expect("prod")
        })
        .collect();

    for p in &mut producers {
        p.wait().unwrap();
    }
    let st = router.wait().unwrap();
    assert!(st.success(), "router falló");

    // dar tiempo a drenar
    std::thread::sleep(Duration::from_millis(50));
    for c in &mut consumers {
        let _ = c.kill();
        let _ = c.wait();
    }
    let dt = t0.elapsed().as_secs_f64();
    println!(
        "route_bench productores={n_in} consumidores={n_out} total={total} segundos={dt:.3} msg_por_seg={:.1}M",
        total as f64 / dt / 1e6
    );

    for i in 0..n_in {
        Ring::unlink(&format!("{prefix}-in-{i}"));
    }
    for j in 0..n_out {
        Ring::unlink(&format!("{prefix}-out-{j}"));
    }
    let _ = pack(0, 0);
    let _ = unpack(0);
}
