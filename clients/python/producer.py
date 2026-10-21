#!/usr/bin/env python3
"""Productor SPSC compatible con el ABI de shm_ring_bus (header 192 bytes)."""
import mmap, os, sys, time, ctypes

HDR = 192
libc = ctypes.CDLL("libc.so.6", use_errno=True)
libc.shm_open.argtypes = [ctypes.c_char_p, ctypes.c_int, ctypes.c_int]
libc.shm_open.restype = ctypes.c_int

O_RDWR = 0o2

def open_ring(name: str, cap: int):
    path = f"/{name}".encode()
    fd = libc.shm_open(path, O_RDWR, 0o600)
    if fd < 0:
        raise OSError(ctypes.get_errno(), "shm_open")
    length = HDR + cap * 8
    mm = mmap.mmap(fd, length)
    os.close(fd)
    buf = (ctypes.c_uint64 * (length // 8)).from_buffer(mm)
    if buf[128 // 8] != cap:
        raise ValueError(f"cap mismatch {buf[128 // 8]} != {cap}")
    return mm, buf

def push_batch(buf, cap, local_head, cached_tail, values):
    mask = cap - 1
    free = cap - (local_head - cached_tail)
    if free < len(values):
        cached_tail = int(buf[64 // 8])  # tail
        free = cap - (local_head - cached_tail)
    n = min(int(free), len(values))
    base = HDR // 8
    for i in range(n):
        buf[base + ((local_head + i) & mask)] = values[i]
    if n:
        local_head += n
        buf[0] = local_head  # head
    return n, local_head, cached_tail

def main():
    name, cap, n, batch = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
    mm, buf = open_ring(name, cap)
    local_head, cached_tail, next_v = 0, 0, 0
    while next_v < n:
        k = min(batch, n - next_v)
        values = list(range(next_v, next_v + k))
        off = 0
        while off < k:
            w, local_head, cached_tail = push_batch(buf, cap, local_head, cached_tail, values[off:])
            if w == 0:
                time.sleep(0)
            off += w
        next_v += k
    del buf
    mm.close()

if __name__ == "__main__":
    main()
