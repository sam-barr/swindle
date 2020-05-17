all: io.ll rc.ll

io.ll: io.c
	clang -S -emit-llvm io.c

rc.ll: rc.c
	clang -S -emit-llvm rc.c
