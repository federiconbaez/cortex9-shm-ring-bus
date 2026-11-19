//! H4: carrera de arranque. Si el par abre (create=false) entre shm_open(O_CREAT) y ftruncate del creador,
//! mmap tiene éxito sobre un objeto de 0 bytes y la primera lectura del header mata al proceso con SIGBUS.
use shm_ring_bus::Ring;
fn main() {
    let name = format!("audit10-sigbus-{}", std::process::id());
    let path = format!("/dev/shm/{name}");
    std::fs::File::create(&path).unwrap(); // estado intermedio del creador: objeto existe, tamaño 0
    println!("abriendo {path} (0 bytes) con create=false...");
    let r = Ring::open(&name, 8, false);
    println!("no debería llegar acá: {:?}", r.is_ok());
}
