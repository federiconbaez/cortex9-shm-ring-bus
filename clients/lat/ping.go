// Emisor Go: crea dos anillos, envía i por <ping>, espera i+1 por <pong> y mide RTT.
// Uso: ping <ping> <pong> <cap> <n> <calentamiento>
package main

import (
	"fmt"
	"os"
	"sort"
	"strconv"
	"sync/atomic"
	"syscall"
	"time"
	"unsafe"
)

const hdr = 192

type ring struct {
	b    []byte
	mask uint64
	loc  uint64
}

func (r *ring) at(off int) *uint64   { return (*uint64)(unsafe.Pointer(&r.b[off])) }
func (r *ring) slot(i uint64) *uint64 { return r.at(hdr + int(i&r.mask)*8) }

func create(name string, cap uint64) *ring {
	path := "/dev/shm/" + name
	fd, err := syscall.Open(path, syscall.O_CREAT|syscall.O_RDWR|syscall.O_TRUNC, 0600)
	if err != nil {
		panic(err)
	}
	n := hdr + int(cap)*8
	if err := syscall.Ftruncate(fd, int64(n)); err != nil {
		panic(err)
	}
	b, err := syscall.Mmap(fd, 0, n, syscall.PROT_READ|syscall.PROT_WRITE, syscall.MAP_SHARED)
	syscall.Close(fd)
	if err != nil {
		panic(err)
	}
	r := &ring{b: b, mask: cap - 1}
	atomic.StoreUint64(r.at(0), 0)
	atomic.StoreUint64(r.at(64), 0)
	atomic.StoreUint64(r.at(128), cap)
	return r
}

func main() {
	if len(os.Args) < 6 {
		fmt.Fprintln(os.Stderr, "uso: ping <ping> <pong> <cap> <n> <calentamiento>")
		os.Exit(1)
	}
	cap, _ := strconv.ParseUint(os.Args[3], 10, 64)
	n, _ := strconv.Atoi(os.Args[4])
	w, _ := strconv.Atoi(os.Args[5])
	out, in := create(os.Args[1], cap), create(os.Args[2], cap)
	// Señal de listo para el lanzador.
	os.WriteFile("/dev/shm/"+os.Args[1]+".ready", nil, 0600)
	lat := make([]int64, 0, n)
	errs := 0
	for i := 0; i < n+w; i++ {
		t := time.Now()
		for out.loc-atomic.LoadUint64(out.at(64)) >= cap {
		}
		*out.slot(out.loc) = uint64(i)
		out.loc++
		atomic.StoreUint64(out.at(0), out.loc)
		for atomic.LoadUint64(in.at(0)) == in.loc {
		}
		y := *in.slot(in.loc)
		in.loc++
		atomic.StoreUint64(in.at(64), in.loc)
		d := time.Since(t).Nanoseconds()
		if y != uint64(i)+1 {
			errs++
		}
		if i >= w {
			lat = append(lat, d)
		}
	}
	sort.Slice(lat, func(a, b int) bool { return lat[a] < lat[b] })
	us := func(q int) float64 { return float64(lat[q]) / 1e3 }
	fmt.Printf("go->c++->go anillo n=%d p50=%.2fus p99=%.2fus p99.9=%.2fus max=%.2fus errores=%d\n",
		n, us(n/2), us(n*99/100), us(n*999/1000), us(n-1), errs)
	os.Remove("/dev/shm/" + os.Args[1])
	os.Remove("/dev/shm/" + os.Args[2])
	os.Remove("/dev/shm/" + os.Args[1] + ".ready")
}
