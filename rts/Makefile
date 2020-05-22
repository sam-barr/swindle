CFLAGS=-S -O2 -emit-llvm -Wall -Wextra

all: io.ll rc.ll strings.ll lists.ll

io.ll: io.c rc.h
	clang $(CFLAGS) io.c

rc.ll: rc.c
	clang $(CFLAGS) rc.c

strings.ll: strings.c
	clang $(CFLAGS) strings.c

lists.ll: lists.c
	clang $(CFLAGS) lists.c
