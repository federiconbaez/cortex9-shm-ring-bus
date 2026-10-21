#include <atomic>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

static constexpr size_t HDR = 192;

struct alignas(64) RingHdr {
  std::atomic<uint64_t> head;
  char pad0[64 - sizeof(std::atomic<uint64_t>)];
  std::atomic<uint64_t> tail;
  char pad1[64 - sizeof(std::atomic<uint64_t>)];
  std::atomic<uint64_t> cap;
  char pad2[64 - sizeof(std::atomic<uint64_t>)];
};
static_assert(sizeof(RingHdr) == HDR, "header must be 192");

struct Ring {
  uint8_t *base{};
  uint64_t cap{}, mask{}, local_head{}, cached_other{};
};

static Ring open_ring(const char *name, uint64_t cap) {
  char path[256];
  std::snprintf(path, sizeof path, "/%s", name);
  int fd = shm_open(path, O_RDWR, 0600);
  if (fd < 0) { perror("shm_open"); std::exit(1); }
  size_t len = HDR + (size_t)cap * 8;
  void *p = mmap(nullptr, len, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
  close(fd);
  if (p == MAP_FAILED) { perror("mmap"); std::exit(1); }
  auto *h = static_cast<RingHdr *>(p);
  if (h->cap.load(std::memory_order_acquire) != cap) {
    std::fprintf(stderr, "cap mismatch\n");
    std::exit(2);
  }
  return Ring{static_cast<uint8_t *>(p), cap, cap - 1, 0, 0};
}

static size_t push(Ring &r, const uint64_t *buf, size_t n) {
  auto *h = reinterpret_cast<RingHdr *>(r.base);
  uint64_t hd = r.local_head;
  uint64_t free = r.cap - (hd - r.cached_other);
  if (free < n) {
    r.cached_other = h->tail.load(std::memory_order_acquire);
    free = r.cap - (hd - r.cached_other);
  }
  size_t m = free < n ? (size_t)free : n;
  auto *slots = reinterpret_cast<uint64_t *>(r.base + HDR);
  for (size_t i = 0; i < m; i++) slots[(hd + i) & r.mask] = buf[i];
  if (m) {
    r.local_head = hd + m;
    h->head.store(r.local_head, std::memory_order_release);
  }
  return m;
}

int main(int argc, char **argv) {
  if (argc < 5) {
    std::fprintf(stderr, "uso: producer <name> <cap> <n> <batch>\n");
    return 1;
  }
  const char *name = argv[1];
  uint64_t cap = std::strtoull(argv[2], nullptr, 10);
  uint64_t n = std::strtoull(argv[3], nullptr, 10);
  size_t batch = (size_t)std::strtoull(argv[4], nullptr, 10);
  Ring r = open_ring(name, cap);
  auto *buf = static_cast<uint64_t *>(std::calloc(batch, 8));
  uint64_t next = 0;
  while (next < n) {
    size_t k = batch;
    if (next + k > n) k = (size_t)(n - next);
    for (size_t i = 0; i < k; i++) buf[i] = next + i;
    size_t off = 0;
    while (off < k) {
      size_t w = push(r, buf + off, k - off);
      if (!w) __builtin_ia32_pause();
      off += w;
    }
    next += k;
  }
  std::free(buf);
  return 0;
}
