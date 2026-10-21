// Respondedor C++: lee de <ping>, suma 1, escribe en <pong>. Uso: pong <ping> <pong> <cap> <n>
#include <atomic>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <fcntl.h>
#include <sys/mman.h>
#include <unistd.h>
static constexpr size_t HDR = 192;
struct Ring { uint8_t *b; uint64_t mask; uint64_t loc; };
static std::atomic<uint64_t> &at(Ring &r, size_t off) { return *reinterpret_cast<std::atomic<uint64_t> *>(r.b + off); }
static Ring open_ring(const char *n, uint64_t cap) {
  char p[256]; std::snprintf(p, sizeof p, "/%s", n);
  int fd = shm_open(p, O_RDWR, 0600); if (fd < 0) { perror("shm_open"); std::exit(1); }
  void *m = mmap(nullptr, HDR + cap * 8, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0); close(fd);
  if (m == MAP_FAILED) { perror("mmap"); std::exit(1); }
  Ring r{static_cast<uint8_t *>(m), cap - 1, 0};
  if (at(r, 128).load(std::memory_order_acquire) != cap) { std::fprintf(stderr, "cap mismatch\n"); std::exit(2); }
  return r;
}
int main(int argc, char **argv) {
  if (argc < 5) { std::fprintf(stderr, "uso: pong <ping> <pong> <cap> <n>\n"); return 1; }
  uint64_t cap = std::strtoull(argv[3], nullptr, 10), n = std::strtoull(argv[4], nullptr, 10);
  Ring in = open_ring(argv[1], cap), out = open_ring(argv[2], cap);
  auto *si = reinterpret_cast<uint64_t *>(in.b + HDR); auto *so = reinterpret_cast<uint64_t *>(out.b + HDR);
  for (uint64_t i = 0; i < n; i++) {
    while (at(in, 0).load(std::memory_order_acquire) == in.loc) {}
    uint64_t x = si[in.loc & in.mask]; in.loc++; at(in, 64).store(in.loc, std::memory_order_release);
    while (out.loc - at(out, 64).load(std::memory_order_acquire) >= cap) {}
    so[out.loc & out.mask] = x + 1; out.loc++; at(out, 0).store(out.loc, std::memory_order_release);
  }
}
