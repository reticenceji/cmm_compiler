CC = ../../target/debug/cmm
CLANG = clang
IO_C = ../../src/io.c

test: *.c
	for cfile in $^ ; do \
		$(CC) --file $$cfile; \
		$(CLANG) $(IO_C) `echo $$cfile | sed 's/\.c//'`.s -o a.out;\
		echo $$cfile"================\n";\
		./a.out >  `echo $$cfile | sed 's/\.c//'`_output.txt;\
		diff `echo $$cfile | sed 's/\.c//'`_output.txt `echo $$cfile | sed 's/\.c//'`_out.txt;\
	done
	

.PHONY: clean	
clean:
	rm *.s
	rm *.ll
	rm a.out
	rm *output.txt
