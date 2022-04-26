CC = ../../target/debug/cmm
CLANG = clang
IO_C = ../../src/io.c

test: *.c
	for cfile in $^ ; do \
		$(CC) --file $$cfile; \
		$(CLANG) $(IO_C) `echo $$cfile | sed 's/\.c//'`.s -o a.out;\
		echo $$cfile"================\n";\
		cat input.txt | ./a.out | diff - output.txt; \
	done
	

.PHONY: clean	
clean:
	rm *.s
	rm *.ll
	rm a.out
