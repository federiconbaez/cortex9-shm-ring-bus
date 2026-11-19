//! H5: par hostil/corrupto escribe head arbitrario. El consumidor no valida head-tail <= cap:
//! entrega datos viejos como nuevos (sin acceso fuera de rango gracias a la máscara).
use shm_ring_bus::Ring;
fn main() {
    let name = format!("audit10-hostil-{}", std::process::id());
    let mut cons = Ring::open(&name, 8, true).unwrap();
    let path = format!("/dev/shm/{name}");
    let f = std::fs::OpenOptions::new().read(true).write(true).open(&path).unwrap();
    use std::os::unix::fs::FileExt;
    f.write_all_at(&1_000_000u64.to_le_bytes(), 0).unwrap(); // head = 1e6 con cap = 8
    let mut out = [0u64; 64];
    let n = cons.pop_batch(&mut out);
    println!("consumidor aceptó {n} mensajes con head-tail = 1e6 > cap = 8: {:?}", &out[..n]);
    Ring::unlink(&name);
}
