.data
hello: .string "hello world\n"
hello_exc: .string "hello from exception\n"

.text
trap:
    li a0, 0x10000000
    la a1, hello_exc
    call print_str
    csrr t1, mepc
    addi t1, t1, 4
    csrw mepc, t1
    mret

.global _start
.section .text.start
_start:
    li a0, 0x10000000
    la a1, hello
    call print_str
    la t1, trap
    csrw mtvec, t1
    ecall
    j end

.text
print_str:
    lbu t0, 0(a1)
    beqz t0, print_str_ret
    sb t0, 0(a0)
    addi a1, a1, 1
    j print_str
print_str_ret:
    ret

end:
    nop
    .rept 4096 / 4
        nop
    .endr
    j _start
