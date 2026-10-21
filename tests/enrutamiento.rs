use shm_ring_bus::{pack, unpack, Ring};
use std::process::Command;
use std::time::Duration;

#[test]
fn maquina_enruta_sin_perder() {
    let cap = 1u64 << 12;
    let n_in = 2usize;
    let n_out = 2usize;
    let per = 50_000u64;
    let total = per * n_in as u64;
    let prefix = format!("fnbr-it-rt-{}", std::process::id());
    for i in 0..n_in {
        Ring::unlink(&format!("{prefix}-in-{i}"));
    }
    for j in 0..n_out {
        Ring::unlink(&format!("{prefix}-out-{j}"));
    }

    let router = env!("CARGO_BIN_EXE_router");
    let prod = env!("CARGO_BIN_EXE_route_prod");
    let mut child = Command::new(router)
        .args([
            &cap.to_string(),
            &n_in.to_string(),
            &n_out.to_string(),
            &prefix,
            &total.to_string(),
        ])
        .spawn()
        .unwrap();
    let ready = std::env::temp_dir().join(format!("{prefix}.router-ready"));
    for _ in 0..500 {
        if ready.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(ready.exists());

    // open egress as consumers in-process
    let mut outs: Vec<Ring> = (0..n_out)
        .map(|j| Ring::open(&format!("{prefix}-out-{j}"), cap, false).unwrap())
        .collect();

    let mut prods = Vec::new();
    for i in 0..n_in {
        prods.push(
            Command::new(prod)
                .args([
                    &format!("{prefix}-in-{i}"),
                    &cap.to_string(),
                    &per.to_string(),
                    "64",
                    &n_out.to_string(),
                    &i.to_string(),
                ])
                .spawn()
                .unwrap(),
        );
    }

    let mut got = vec![0u64; n_out];
    let mut bad = 0u64;
    let mut buf = [0u64; 128];
    let t0 = std::time::Instant::now();
    while got.iter().sum::<u64>() < total {
        assert!(t0.elapsed() < Duration::from_secs(30), "timeout");
        for (j, r) in outs.iter_mut().enumerate() {
            let n = r.pop_batch(&mut buf);
            for &m in &buf[..n] {
                let (dst, _) = unpack(m);
                if dst as usize != j {
                    bad += 1;
                }
                got[j] += 1;
            }
        }
    }
    for mut p in prods {
        p.wait().unwrap();
    }
    child.wait().unwrap();
    assert_eq!(bad, 0, "mensajes en destino incorrecto");
    assert_eq!(got.iter().sum::<u64>(), total);
    for i in 0..n_in {
        Ring::unlink(&format!("{prefix}-in-{i}"));
    }
    for j in 0..n_out {
        Ring::unlink(&format!("{prefix}-out-{j}"));
    }
    let _ = pack;
}
