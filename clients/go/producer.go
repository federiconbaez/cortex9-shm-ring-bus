package main

import (
	"fmt"
	"os"
	"runtime"
	"strconv"
	"syscall"
	"unsafe"
)

const hdr = 192

type ring struct {
	base        []byte
	cap, mask   uint64
	localHead   uint64
	cachedOther uint64
}

func openRing(name string, cap uint64) (*ring, error) {
	fd, err := syscall.Open("/dev/shm/"+name, syscall.O_RDWR, 0)
	if err != nil {
		return nil, err
	}
	length := hdr + int(cap)*8
	b, err := syscall.Mmap(fd, 0, length, syscall.PROT_READ|syscall.PROT_WRITE, syscall.MAP_SHARED)
	syscall.Close(fd)
	if err != nil {
		return nil, err
	}
	c := *(*uint64)(unsafe.Pointer(&b[128]))
	if c != cap {
		return nil, fmt.Errorf("cap mismatch %d != %d", c, cap)
	}
	return &ring{base: b, cap: cap, mask: cap - 1}, nil
}

func (r *ring) push(buf []uint64) int {
	h := r.localHead
	free := r.cap - (h - r.cachedOther)
	if int(free) < len(buf) {
		r.cachedOther = *(*uint64)(unsafe.Pointer(&r.base[64]))
		free = r.cap - (h - r.cachedOther)
	}
	n := int(free)
	if n > len(buf) {
		n = len(buf)
	}
	for i := 0; i < n; i++ {
		off := hdr + int((h+uint64(i))&r.mask)*8
		*(*uint64)(unsafe.Pointer(&r.base[off])) = buf[i]
	}
	if n > 0 {
		r.localHead = h + uint64(n)
		*(*uint64)(unsafe.Pointer(&r.base[0])) = r.localHead
	}
	return n
}

func main() {
	if len(os.Args) < 5 {
		fmt.Fprintln(os.Stderr, "uso: producer <name> <cap> <n> <batch>")
		os.Exit(1)
	}
	name := os.Args[1]
	cap, _ := strconv.ParseUint(os.Args[2], 10, 64)
	nMsg, _ := strconv.ParseUint(os.Args[3], 10, 64)
	batch, _ := strconv.Atoi(os.Args[4])
	r, err := openRing(name, cap)
	if err != nil {
		panic(err)
	}
	buf := make([]uint64, batch)
	var next uint64
	for next < nMsg {
		k := batch
		if next+uint64(k) > nMsg {
			k = int(nMsg - next)
		}
		for i := 0; i < k; i++ {
			buf[i] = next + uint64(i)
		}
		off := 0
		for off < k {
			w := r.push(buf[off:k])
			if w == 0 {
				runtime.Gosched()
				continue
			}
			off += w
		}
		next += uint64(k)
	}
}
