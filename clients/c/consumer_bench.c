#define _POSIX_C_SOURCE 200809L
#include "ring.h"
#include <sys/wait.h>
int main(int argc, char **argv) {
  uint64_t n = argc > 1 ? strtoull(argv[1], NULL, 10) : 100000000;
  size_t batch = argc > 2 ? (size_t)strtoull(argv[2], NULL, 10) : 256;
  uint64_t cap = 1ull << 16;
  char name[64]; snprintf(name, sizeof name, "fnbr-c-%d", getpid());
  Ring r = ring_open(name, cap, 1);
  char prod[512];
  snprintf(prod, sizeof prod, "%s/producer", getenv("RING_C_BIN") ? getenv("RING_C_BIN") : ".");
  pid_t pid = fork();
  if (pid == 0) {
    char cap_s[32], n_s[32], b_s[32];
    snprintf(cap_s, sizeof cap_s, "%llu", (unsigned long long)cap);
    snprintf(n_s, sizeof n_s, "%llu", (unsigned long long)n);
    snprintf(b_s, sizeof b_s, "%zu", batch);
    execl(prod, prod, name, cap_s, n_s, b_s, (char*)NULL);
    perror("execl"); _exit(127);
  }
  struct timespec t0, t1; clock_gettime(CLOCK_MONOTONIC, &t0);
  uint64_t *out = calloc(batch, 8);
  uint64_t expect = 0, err = 0;
  while (expect < n) {
    size_t k = ring_pop(&r, out, batch);
    if (!k) { __builtin_ia32_pause(); continue; }
    for (size_t i = 0; i < k; i++) { if (out[i] != expect) err++; expect++; }
  }
  clock_gettime(CLOCK_MONOTONIC, &t1);
  waitpid(pid, NULL, 0);
  double dt = (t1.tv_sec - t0.tv_sec) + (t1.tv_nsec - t0.tv_nsec) * 1e-9;
  printf("c_c mensajes=%llu lote=%zu segundos=%.3f msg_por_seg=%.1fM errores=%llu\n",
         (unsigned long long)n, batch, dt, n / dt / 1e6, (unsigned long long)err);
  ring_unlink(name); free(out);
  return err ? 1 : 0;
}
