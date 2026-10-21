#define _POSIX_C_SOURCE 200809L
#include "ring.h"
int main(int argc, char **argv) {
  if (argc < 5) { fprintf(stderr, "uso: producer <name> <cap> <n> <batch>\n"); return 1; }
  const char *name = argv[1];
  uint64_t cap = strtoull(argv[2], NULL, 10);
  uint64_t n = strtoull(argv[3], NULL, 10);
  size_t batch = (size_t)strtoull(argv[4], NULL, 10);
  Ring r = ring_open(name, cap, 0);
  uint64_t *buf = calloc(batch, 8);
  uint64_t next = 0;
  while (next < n) {
    size_t k = batch; if (next + k > n) k = (size_t)(n - next);
    for (size_t i = 0; i < k; i++) buf[i] = next + i;
    size_t off = 0;
    while (off < k) {
      size_t w = ring_push(&r, buf + off, k - off);
      if (!w) __builtin_ia32_pause();
      off += w;
    }
    next += k;
  }
  free(buf);
  return 0;
}
