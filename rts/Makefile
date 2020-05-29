CFLAGS=-S -O2 -emit-llvm -Wall -Wextra

all: io.ll rc.ll strings.ll lists.ll closures.ll

io.ll: io.c rc.h
	clang $(CFLAGS) io.c

rc.ll: rc.c rc.h
	clang $(CFLAGS) rc.c

strings.ll: strings.c rc.h strings.h
	clang $(CFLAGS) strings.c

lists.ll: lists.c rc.h lists.h strings.h
	clang $(CFLAGS) lists.c

closures.ll: closures.c closures.h rc.h
	clang $(CFLAGS) closures.c
