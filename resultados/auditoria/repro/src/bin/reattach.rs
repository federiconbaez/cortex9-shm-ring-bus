//! H1: un productor que se reinicia y reabre el anillo (create=false) arranca con local_head=0
//! aunque el head compartido ya avanzó. El consumidor recibe basura sin error.
use shm_ring_bus::Ring;
fn main() {
    let name = format!("audit10-reattach-{}", std::process::id());
    let mut cons = Ring::open(&name, 8, true).unwrap();
    {
        let mut p1 = Ring::open(&name, 8, false).unwrap();
        assert_eq!(p1.push_batch(&[100, 101, 102, 103, 104]), 5);
    } // "crash"/salida del productor 1
    let mut out = [0u64; 8];
    let n = cons.pop_batch(&mut out);
    println!("antes del reinicio: consumidor leyó {n}: {:?}", &out[..n]);
    let mut p2 = Ring::open(&name, 8, false).unwrap(); // productor reiniciado
    let w = p2.push_batch(&[200]);
    println!("productor reiniciado escribió {w} mensaje (200)");
    let n = cons.pop_batch(&mut out);
    println!("consumidor leyó {n} mensajes: {:?}  (esperado: 1 mensaje [200])", &out[..n]);
    let n2 = cons.pop_batch(&mut out);
    println!("siguiente pop: {n2} mensajes: {:?}", &out[..n2]);
    Ring::unlink(&name);
}
