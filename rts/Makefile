CFLAGS=-S -emit-llvm -O2

all: io.ll rc.ll

io.ll: io.c rc.h
	clang $(CFLAGS) io.c

rc.ll: rc.c
	clang $(CFLAGS) rc.c
