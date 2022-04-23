	.section	__TEXT,__text,regular,pure_instructions
	.build_version macos, 12, 0
	.globl	_main
	.p2align	4, 0x90
_main:
	.cfi_startproc
	movl	$1, -4(%rsp)
	movl	$1, %eax
	retq
	.cfi_endproc

.subsections_via_symbols
