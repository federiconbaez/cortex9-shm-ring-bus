//! Ring buffer SPSC lock-free sobre memoria compartida (/dev/shm).
//! Un productor y un consumidor en procesos distintos; mensajes de 8 bytes.
use std::ffi::CString;
use std::sync::atomic::{AtomicU64, Ordering};

#[repr(C, align(64))]
struct Pad(AtomicU64);

#[repr(C)]
struct Header {
    head: Pad,      // escrito por el productor
    tail: Pad,      // escrito por el consumidor
    cap: Pad,       // capacidad (potencia de 2)
}

pub struct Ring {
    base: *mut u8,
    len: usize,
    cap: u64,
    mask: u64,
    // cachés locales para evitar tocar la línea del otro lado
    local_head: u64,
    local_tail: u64,
    cached_other: u64,
}
unsafe impl Send for Ring {}

fn hdr_size() -> usize { std::mem::size_of::<Header>() }

impl Ring {
    /// Crea (o abre) el segmento `/dev/shm/<name>` con `cap` slots (potencia de 2).
    pub fn open(name: &str, cap: u64, create: bool) -> std::io::Result<Ring> {
        assert!(cap.is_power_of_two());
        let len = hdr_size() + (cap as usize) * 8;
        let cname = CString::new(format!("/{name}")).unwrap();
        unsafe {
            let flags = if create { libc::O_CREAT | libc::O_RDWR } else { libc::O_RDWR };
            let fd = libc::shm_open(cname.as_ptr(), flags, 0o600);
            if fd < 0 { return Err(std::io::Error::last_os_error()); }
            if create && libc::ftruncate(fd, len as libc::off_t) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let p = libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0);
            libc::close(fd);
            if p == libc::MAP_FAILED { return Err(std::io::Error::last_os_error()); }
            let r = Ring { base: p as *mut u8, len, cap, mask: cap - 1, local_head: 0, local_tail: 0, cached_other: 0 };
            if create {
                r.hdr().head.0.store(0, Ordering::Relaxed);
                r.hdr().tail.0.store(0, Ordering::Relaxed);
                r.hdr().cap.0.store(cap, Ordering::Release);
            } else if r.hdr().cap.0.load(Ordering::Acquire) != cap {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "capacidad distinta"));
            }
            Ok(r)
        }
    }
    pub fn unlink(name: &str) {
        let c = CString::new(format!("/{name}")).unwrap();
        unsafe { libc::shm_unlink(c.as_ptr()); }
    }
    fn hdr(&self) -> &Header { unsafe { &*(self.base as *const Header) } }
    fn slots(&self) -> *mut u64 { unsafe { self.base.add(hdr_size()) as *mut u64 } }

    /// Productor: escribe hasta `buf.len()` mensajes; devuelve cuántos entraron.
    pub fn push_batch(&mut self, buf: &[u64]) -> usize {
        let h = self.local_head;
        let mut free = self.cap - (h - self.cached_other);
        if (free as usize) < buf.len() {
            self.cached_other = self.hdr().tail.0.load(Ordering::Acquire);
            free = self.cap - (h - self.cached_other);
        }
        let n = (free as usize).min(buf.len());
        for (i, v) in buf[..n].iter().enumerate() {
            unsafe { *self.slots().add(((h + i as u64) & self.mask) as usize) = *v; }
        }
        if n > 0 {
            self.local_head = h + n as u64;
            self.hdr().head.0.store(self.local_head, Ordering::Release);
        }
        n
    }
    /// Consumidor: lee hasta `out.len()` mensajes; devuelve cuántos leyó.
    pub fn pop_batch(&mut self, out: &mut [u64]) -> usize {
        let t = self.local_tail;
        let mut avail = self.cached_other - t;
        if avail == 0 {
            self.cached_other = self.hdr().head.0.load(Ordering::Acquire);
            avail = self.cached_other - t;
        }
        let n = (avail as usize).min(out.len());
        for (i, o) in out[..n].iter_mut().enumerate() {
            *o = unsafe { *self.slots().add(((t + i as u64) & self.mask) as usize) };
        }
        if n > 0 {
            self.local_tail = t + n as u64;
            self.hdr().tail.0.store(self.local_tail, Ordering::Release);
        }
        n
    }
}
impl Drop for Ring {
    fn drop(&mut self) { unsafe { libc::munmap(self.base as *mut _, self.len); } }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Integridad SPSC entre HILOS del mismo proceso (no entre procesos).
    #[test]
    fn orden_e_integridad_entre_hilos() {
        let name = format!("fnbr-test-{}", std::process::id());
        let mut p = Ring::open(&name, 1024, true).unwrap();
        let mut c = Ring::open(&name, 1024, false).unwrap();
        const N: u64 = 5_000_000;
        let prod = std::thread::spawn(move || {
            let mut next = 0u64; let mut buf = [0u64; 37];
            while next < N {
                let k = buf.len().min((N - next) as usize);
                for (i, slot) in buf.iter_mut().take(k).enumerate() { *slot = next + i as u64; }
                let mut off = 0;
                while off < k { off += p.push_batch(&buf[off..k]); }
                next += k as u64;
            }
        });
        let mut expect = 0u64; let mut out = [0u64; 53];
        while expect < N {
            let n = c.pop_batch(&mut out);
            for v in &out[..n] { assert_eq!(*v, expect); expect += 1; }
        }
        prod.join().unwrap();
        Ring::unlink(&name);
    }
    #[test]
    fn nunca_desborda() {
        let name = format!("fnbr-test2-{}", std::process::id());
        let mut p = Ring::open(&name, 8, true).unwrap();
        assert_eq!(p.push_batch(&[1; 20]), 8);
        assert_eq!(p.push_batch(&[1]), 0);
        Ring::unlink(&name);
    }
}
