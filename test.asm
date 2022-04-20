	.text
	.file	"test"
	.globl	main
	.p2align	4, 0x90
	.type	main,@function
main:
	.cfi_startproc
	movl	$1, -4(%rsp)
	movl	$0, -44(%rsp)
	movl	-4(%rsp), %eax
	movl	%eax, -40(%rsp)
	movl	-4(%rsp), %eax
	addl	$1, %eax
	movl	%eax, -36(%rsp)
	imull	$3, -4(%rsp), %eax
	movl	%eax, -32(%rsp)
	movl	-32(%rsp), %eax
	addl	$1, %eax
	movl	%eax, -28(%rsp)
.LBB0_1:
	cmpl	$0, -4(%rsp)
	jl	.LBB0_3
	movl	$6, %eax
	movl	%eax, %ecx
	subl	-4(%rsp), %ecx
	subl	-4(%rsp), %eax
	movslq	%ecx, %rdx
	movl	%eax, -44(%rsp,%rdx,4)
	movl	-4(%rsp), %eax
	subl	$1, %eax
	movl	%eax, -4(%rsp)
	jmp	.LBB0_1
.LBB0_3:
	cmpl	$6, -20(%rsp)
	jne	.LBB0_5
	movl	$7, -16(%rsp)
.LBB0_5:
	movl	$1, %eax
	retq
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc

	.section	".note.GNU-stack","",@progbits
