#pragma once
#include <stdint.h>
#include <stdatomic.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define RING_HDR 192
typedef struct {
  _Alignas(64) _Atomic uint64_t head;
  _Alignas(64) _Atomic uint64_t tail;
  _Alignas(64) _Atomic uint64_t cap;
} RingHdr;

typedef struct {
  uint8_t *base;
  size_t len;
  uint64_t cap, mask;
  uint64_t local_head, local_tail, cached_other;
} Ring;

static inline Ring ring_open(const char *name, uint64_t cap, int create) {
  Ring r; memset(&r, 0, sizeof r);
  char path[256];
  snprintf(path, sizeof path, "/%s", name);
  size_t len = RING_HDR + (size_t)cap * 8;
  int flags = create ? (O_CREAT|O_RDWR) : O_RDWR;
  int fd = shm_open(path, flags, 0600);
  if (fd < 0) { perror("shm_open"); exit(1); }
  if (create && ftruncate(fd, (off_t)len) != 0) { perror("ftruncate"); exit(1); }
  void *p = mmap(NULL, len, PROT_READ|PROT_WRITE, MAP_SHARED, fd, 0);
  close(fd);
  if (p == MAP_FAILED) { perror("mmap"); exit(1); }
  r.base = p; r.len = len; r.cap = cap; r.mask = cap - 1;
  RingHdr *h = (RingHdr*)p;
  if (create) {
    atomic_store_explicit(&h->head, 0, memory_order_relaxed);
    atomic_store_explicit(&h->tail, 0, memory_order_relaxed);
    atomic_store_explicit(&h->cap, cap, memory_order_release);
  } else if (atomic_load_explicit(&h->cap, memory_order_acquire) != cap) {
    fprintf(stderr, "cap mismatch\n"); exit(2);
  }
  return r;
}
static inline uint64_t *slots(Ring *r) { return (uint64_t*)(r->base + RING_HDR); }
static inline size_t ring_push(Ring *r, const uint64_t *buf, size_t n) {
  RingHdr *h = (RingHdr*)r->base;
  uint64_t hd = r->local_head;
  uint64_t free = r->cap - (hd - r->cached_other);
  if (free < n) {
    r->cached_other = atomic_load_explicit(&h->tail, memory_order_acquire);
    free = r->cap - (hd - r->cached_other);
  }
  size_t m = free < n ? (size_t)free : n;
  for (size_t i = 0; i < m; i++) slots(r)[(hd + i) & r->mask] = buf[i];
  if (m) {
    r->local_head = hd + m;
    atomic_store_explicit(&h->head, r->local_head, memory_order_release);
  }
  return m;
}
static inline size_t ring_pop(Ring *r, uint64_t *buf, size_t n) {
  RingHdr *h = (RingHdr*)r->base;
  uint64_t tl = r->local_tail;
  uint64_t avail = r->cached_other - tl;
  if (avail < n) {
    r->cached_other = atomic_load_explicit(&h->head, memory_order_acquire);
    avail = r->cached_other - tl;
  }
  size_t m = avail < n ? (size_t)avail : n;
  for (size_t i = 0; i < m; i++) buf[i] = slots(r)[(tl + i) & r->mask];
  if (m) {
    r->local_tail = tl + m;
    atomic_store_explicit(&h->tail, r->local_tail, memory_order_release);
  }
  return m;
}
static inline void ring_unlink(const char *name) {
  char path[256]; snprintf(path, sizeof path, "/%s", name); shm_unlink(path);
}
