CFLAGS=-O3 -std=c11 -Wall -Wextra
LDFLAGS=-lrt
all: producer consumer_bench
producer: producer.c ring.h
	$(CC) $(CFLAGS) -o $@ producer.c $(LDFLAGS)
consumer_bench: consumer_bench.c ring.h
	$(CC) $(CFLAGS) -o $@ consumer_bench.c $(LDFLAGS)
clean:
	rm -f producer consumer_bench
