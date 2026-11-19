//! H2: open(create=true) sobre un nombre existente (sin O_EXCL) pone head/tail en 0 debajo de pares vivos.
//! Con head > cap, el productor calcula free = cap - (head - 0) con resta envolvente y sobrescribe datos no leídos.
use shm_ring_bus::Ring;
fn main() {
    let name = format!("audit10-recreate-{}", std::process::id());
    let mut cons = Ring::open(&name, 8, true).unwrap();
    let mut prod = Ring::open(&name, 8, false).unwrap();
    let mut out = [0u64; 64];
    let mut seq = 0u64;
    for _ in 0..2 { // dos vueltas completas: head = tail = 16
        let v: Vec<u64> = (seq..seq + 8).collect(); seq += 8;
        assert_eq!(prod.push_batch(&v), 8);
        assert_eq!(cons.pop_batch(&mut out), 8);
    }
    let _otro = Ring::open(&name, 8, true).unwrap(); // otro proceso crea con el mismo nombre
    let a: Vec<u64> = (seq..seq + 8).collect(); seq += 8;
    let b: Vec<u64> = (seq..seq + 8).collect();
    let wa = prod.push_batch(&a);
    let wb = prod.push_batch(&b); // el consumidor todavía no leyó `a`: debería devolver 0
    println!("con el anillo lleno (cap=8) el productor escribió {wa} + {wb} = {} mensajes", wa + wb);
    let n = cons.pop_batch(&mut out);
    println!("consumidor leyó {n}: {:?}  (esperado 16..24 y después 24..32)", &out[..n]);
    Ring::unlink(&name);
}
