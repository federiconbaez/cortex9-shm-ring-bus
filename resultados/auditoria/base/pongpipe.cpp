// Respondedor C++ por pipe: lee u64 de fd 3, escribe x+1 en fd 4.
#include <cstdint>
#include <cstdlib>
#include <unistd.h>
int main(int argc, char **argv) {
  uint64_t n = std::strtoull(argv[1], nullptr, 10), x;
  for (uint64_t i = 0; i < n; i++) {
    size_t g = 0; while (g < 8) { ssize_t k = read(3, (char*)&x + g, 8 - g); if (k <= 0) return 1; g += k; }
    x++; if (write(4, &x, 8) != 8) return 1;
  }
}
