CXXFLAGS=-O3 -std=c++17 -Wall -Wextra
all: producer
producer: producer.cpp ../c/ring.h
	$(CXX) $(CXXFLAGS) -o $@ producer.cpp -lrt
clean:
	rm -f producer
