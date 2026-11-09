	.file	"adler2.84a311d043e08519-cgu.0"
	.section	.text._RNvCsbo1sFMHgwtx_6adler213adler32_slice,"ax",@progbits
	.globl	_RNvCsbo1sFMHgwtx_6adler213adler32_slice
	.p2align	4
	.type	_RNvCsbo1sFMHgwtx_6adler213adler32_slice,@function
_RNvCsbo1sFMHgwtx_6adler213adler32_slice:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	%rsi, %rdx
	movq	%rdi, %rsi
	movl	$1, 4(%rsp)
	leaq	4(%rsp), %rdi
	callq	*_RNvMs_Csbo1sFMHgwtx_6adler2NtB4_7Adler3211write_slice@GOTPCREL(%rip)
	movl	4(%rsp), %eax
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end0:
	.size	_RNvCsbo1sFMHgwtx_6adler213adler32_slice, .Lfunc_end0-_RNvCsbo1sFMHgwtx_6adler213adler32_slice
	.cfi_endproc

	.section	.rodata.cst16,"aM",@progbits,16
	.p2align	4, 0x0
.LCPI1_0:
	.long	2147975281
	.long	2147975281
	.long	2147975281
	.long	2147975281
.LCPI1_1:
	.long	65521
	.long	65521
	.long	65521
	.long	65521
	.section	.text._RNvMs_Csbo1sFMHgwtx_6adler2NtB4_7Adler3211write_slice,"ax",@progbits
	.globl	_RNvMs_Csbo1sFMHgwtx_6adler2NtB4_7Adler3211write_slice
	.p2align	4
	.type	_RNvMs_Csbo1sFMHgwtx_6adler2NtB4_7Adler3211write_slice,@function
_RNvMs_Csbo1sFMHgwtx_6adler2NtB4_7Adler3211write_slice:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	pushq	%r15
	.cfi_def_cfa_offset 24
	pushq	%r14
	.cfi_def_cfa_offset 32
	pushq	%r13
	.cfi_def_cfa_offset 40
	pushq	%r12
	.cfi_def_cfa_offset 48
	pushq	%rbx
	.cfi_def_cfa_offset 56
	.cfi_offset %rbx, -56
	.cfi_offset %r12, -48
	.cfi_offset %r13, -40
	.cfi_offset %r14, -32
	.cfi_offset %r15, -24
	.cfi_offset %rbp, -16
	movq	%rdx, %rcx
	movzwl	(%rdi), %r9d
	movzwl	2(%rdi), %r10d
	movabsq	$9223372036854775804, %r8
	andq	%rdx, %r8
	movabsq	$3402281327715882719, %rdx
	movq	%r8, %rax
	mulq	%rdx
	andl	$3, %ecx
	movq	%rcx, -8(%rsp)
	shrq	$12, %rdx
	imulq	$22208, %rdx, %r11
	movq	%r8, %rax
	subq	%r11, %rax
	xorl	%r14d, %r14d
	cmpq	$22208, %r11
	movl	%r9d, -12(%rsp)
	jae	.LBB1_11
	xorl	%r13d, %r13d
	xorl	%ebx, %ebx
	xorl	%edx, %edx
	xorl	%r15d, %r15d
	xorl	%r12d, %r12d
	xorl	%r9d, %r9d
	xorl	%ebp, %ebp
	testq	%rax, %rax
	jne	.LBB1_4
	jmp	.LBB1_6
.LBB1_11:
	imull	$22208, %r9d, %edx
	pxor	%xmm1, %xmm1
	movdqa	.LCPI1_0(%rip), %xmm2
	movdqa	.LCPI1_1(%rip), %xmm3
	movl	$2147975281, %r9d
	movq	%rsi, %rbx
	movq	%r11, %r14
	pxor	%xmm0, %xmm0
	pxor	%xmm4, %xmm4
	.p2align	4
.LBB1_12:
	movq	$-22208, %r15
	.p2align	4
.LBB1_13:
	movd	22208(%rbx,%r15), %xmm5
	punpcklbw	%xmm1, %xmm5
	punpcklwd	%xmm1, %xmm5
	paddd	%xmm4, %xmm5
	paddd	%xmm5, %xmm0
	movd	22212(%rbx,%r15), %xmm4
	punpcklbw	%xmm1, %xmm4
	punpcklwd	%xmm1, %xmm4
	paddd	%xmm5, %xmm4
	paddd	%xmm4, %xmm0
	movd	22216(%rbx,%r15), %xmm5
	punpcklbw	%xmm1, %xmm5
	punpcklwd	%xmm1, %xmm5
	paddd	%xmm4, %xmm5
	paddd	%xmm5, %xmm0
	movd	22220(%rbx,%r15), %xmm4
	punpcklbw	%xmm1, %xmm4
	punpcklwd	%xmm1, %xmm4
	paddd	%xmm5, %xmm4
	paddd	%xmm4, %xmm0
	addq	$16, %r15
	jne	.LBB1_13
	addq	$22208, %rbx
	addq	$-22208, %r14
	addl	%edx, %r10d
	movdqa	%xmm4, %xmm5
	pmuludq	%xmm2, %xmm5
	pshufd	$237, %xmm5, %xmm5
	pshufd	$245, %xmm4, %xmm6
	pmuludq	%xmm2, %xmm6
	pshufd	$237, %xmm6, %xmm6
	punpckldq	%xmm6, %xmm5
	psrld	$15, %xmm5
	pshufd	$245, %xmm5, %xmm6
	pmuludq	%xmm3, %xmm5
	pshufd	$232, %xmm5, %xmm5
	pmuludq	%xmm3, %xmm6
	pshufd	$232, %xmm6, %xmm6
	punpckldq	%xmm6, %xmm5
	psubd	%xmm5, %xmm4
	movdqa	%xmm0, %xmm5
	pmuludq	%xmm2, %xmm5
	pshufd	$237, %xmm5, %xmm5
	pshufd	$245, %xmm0, %xmm6
	pmuludq	%xmm2, %xmm6
	pshufd	$237, %xmm6, %xmm6
	punpckldq	%xmm6, %xmm5
	psrld	$15, %xmm5
	pshufd	$245, %xmm5, %xmm6
	pmuludq	%xmm3, %xmm5
	pshufd	$232, %xmm5, %xmm5
	pmuludq	%xmm3, %xmm6
	pshufd	$232, %xmm6, %xmm6
	punpckldq	%xmm6, %xmm5
	psubd	%xmm5, %xmm0
	movq	%r10, %rcx
	imulq	%r9, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %r10d
	cmpq	$22208, %r14
	jae	.LBB1_12
	pshufd	$255, %xmm0, %xmm1
	movd	%xmm1, %ebx
	pshufd	$238, %xmm0, %xmm1
	movd	%xmm1, %r13d
	pshufd	$85, %xmm0, %xmm1
	movd	%xmm1, %r14d
	movd	%xmm0, %r9d
	pshufd	$255, %xmm4, %xmm0
	movd	%xmm0, %edx
	pshufd	$238, %xmm4, %xmm0
	movd	%xmm0, %r15d
	pshufd	$85, %xmm4, %xmm0
	movd	%xmm0, %r12d
	movd	%xmm4, %ebp
	testq	%rax, %rax
	je	.LBB1_6
	.p2align	4
.LBB1_4:
	movzbl	(%rsi,%r11), %ecx
	addl	%ecx, %ebp
	movzbl	1(%rsi,%r11), %ecx
	addl	%ecx, %r12d
	movzbl	2(%rsi,%r11), %ecx
	addl	%ecx, %r15d
	movzbl	3(%rsi,%r11), %ecx
	addl	%ecx, %edx
	addl	%ebp, %r9d
	addl	%r12d, %r14d
	addl	%r15d, %r13d
	addl	%edx, %ebx
	addq	$4, %r11
	cmpq	%r11, %r8
	jne	.LBB1_4
	movl	%ebp, %ecx
	movl	$2147975281, %r11d
	imulq	%r11, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %ebp
	movl	%r12d, %ecx
	imulq	%r11, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %r12d
	movl	%r15d, %ecx
	imulq	%r11, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %r15d
	movl	%edx, %ecx
	imulq	%r11, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %edx
	movl	%r9d, %ecx
	imulq	%r11, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %r9d
	movl	%r14d, %ecx
	imulq	%r11, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %r14d
	movl	%r13d, %ecx
	imulq	%r11, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %r13d
	movl	%ebx, %ecx
	imulq	%r11, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %ebx
.LBB1_6:
	addl	%r13d, %r14d
	addl	%r9d, %ebx
	movl	$65521, %ecx
	subl	%edx, %ecx
	leal	(%rcx,%rcx,2), %ecx
	movl	-12(%rsp), %r11d
	imull	%r11d, %eax
	addl	%eax, %r10d
	movl	$2147975281, %eax
	movq	%r10, %r9
	imulq	%rax, %r9
	shrq	$47, %r9
	imull	$65521, %r9d, %r9d
	subl	%r9d, %r10d
	leal	(%r10,%r14,4), %r9d
	addl	%ecx, %r9d
	leal	(%r12,%r15,2), %ecx
	subl	%ecx, %r9d
	leal	(%r9,%rbx,4), %r9d
	addl	$196563, %r9d
	addl	%r15d, %edx
	addl	%r12d, %edx
	addl	%ebp, %edx
	addl	%r11d, %edx
	movq	-8(%rsp), %r10
	testq	%r10, %r10
	je	.LBB1_10
	movzbl	(%rsi,%r8), %ecx
	addl	%ecx, %edx
	addl	%edx, %r9d
	cmpl	$1, %r10d
	je	.LBB1_10
	movzbl	1(%rsi,%r8), %ecx
	addl	%ecx, %edx
	addl	%edx, %r9d
	cmpl	$2, %r10d
	je	.LBB1_10
	movzbl	2(%rsi,%r8), %ecx
	addl	%ecx, %edx
	addl	%edx, %r9d
.LBB1_10:
	movl	%edx, %ecx
	imulq	%rax, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %ecx
	subl	%ecx, %edx
	movw	%dx, (%rdi)
	movl	%r9d, %ecx
	imulq	%rax, %rcx
	shrq	$47, %rcx
	imull	$65521, %ecx, %eax
	subl	%eax, %r9d
	movw	%r9w, 2(%rdi)
	popq	%rbx
	.cfi_def_cfa_offset 48
	popq	%r12
	.cfi_def_cfa_offset 40
	popq	%r13
	.cfi_def_cfa_offset 32
	popq	%r14
	.cfi_def_cfa_offset 24
	popq	%r15
	.cfi_def_cfa_offset 16
	popq	%rbp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end1:
	.size	_RNvMs_Csbo1sFMHgwtx_6adler2NtB4_7Adler3211write_slice, .Lfunc_end1-_RNvMs_Csbo1sFMHgwtx_6adler2NtB4_7Adler3211write_slice
	.cfi_endproc

	.section	.text._RNvXs1_Csbo1sFMHgwtx_6adler2NtB5_7Adler32NtNtCsgxBkk5gSRhY_4core4hash6Hasher5write,"ax",@progbits
	.globl	_RNvXs1_Csbo1sFMHgwtx_6adler2NtB5_7Adler32NtNtCsgxBkk5gSRhY_4core4hash6Hasher5write
	.p2align	4
	.type	_RNvXs1_Csbo1sFMHgwtx_6adler2NtB5_7Adler32NtNtCsgxBkk5gSRhY_4core4hash6Hasher5write,@function
_RNvXs1_Csbo1sFMHgwtx_6adler2NtB5_7Adler32NtNtCsgxBkk5gSRhY_4core4hash6Hasher5write:
	.cfi_startproc
	jmpq	*_RNvMs_Csbo1sFMHgwtx_6adler2NtB4_7Adler3211write_slice@GOTPCREL(%rip)
.Lfunc_end2:
	.size	_RNvXs1_Csbo1sFMHgwtx_6adler2NtB5_7Adler32NtNtCsgxBkk5gSRhY_4core4hash6Hasher5write, .Lfunc_end2-_RNvXs1_Csbo1sFMHgwtx_6adler2NtB5_7Adler32NtNtCsgxBkk5gSRhY_4core4hash6Hasher5write
	.cfi_endproc

	.ident	"rustc version 1.98.1 (48a229cea 2026-09-01)"
	.section	".note.GNU-stack","",@progbits
