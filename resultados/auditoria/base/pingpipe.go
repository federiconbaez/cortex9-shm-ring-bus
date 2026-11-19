// Línea base de auditoría: RTT Go -> C++ -> Go por pipes (misma carga que clients/lat: 8 B, 200k tras 10k).
package main

import (
	"encoding/binary"
	"fmt"
	"os"
	"os/exec"
	"sort"
	"time"
)

func main() {
	n, w := 200000, 10000
	r1, w1, _ := os.Pipe() // go -> c++
	r2, w2, _ := os.Pipe() // c++ -> go
	cmd := exec.Command(os.Args[1], fmt.Sprint(n+w))
	cmd.ExtraFiles = []*os.File{r1, w2} // fd 3 = leer, fd 4 = escribir
	cmd.Stderr = os.Stderr
	if err := cmd.Start(); err != nil { panic(err) }
	r1.Close(); w2.Close()
	lat := make([]int64, 0, n)
	errs := 0
	b := make([]byte, 8)
	for i := 0; i < n+w; i++ {
		t := time.Now()
		binary.LittleEndian.PutUint64(b, uint64(i))
		if _, err := w1.Write(b); err != nil { panic(err) }
		got := 0
		for got < 8 { k, err := r2.Read(b[got:]); if err != nil { panic(err) }; got += k }
		d := time.Since(t).Nanoseconds()
		if binary.LittleEndian.Uint64(b) != uint64(i)+1 { errs++ }
		if i >= w { lat = append(lat, d) }
	}
	cmd.Wait()
	sort.Slice(lat, func(a, b int) bool { return lat[a] < lat[b] })
	us := func(q int) float64 { return float64(lat[q]) / 1e3 }
	fmt.Printf("go->c++->go pipe n=%d p50=%.2fus p99=%.2fus p99.9=%.2fus max=%.2fus errores=%d\n",
		n, us(n/2), us(n*99/100), us(n*999/1000), us(n-1), errs)
}
