.data
hello: .string "hello world\n"

.global _start
.section .text.start
_start:
    li a0, 0x10000000
    la a1, hello
    j print_str

.text
print_str:
    lbu t0, 0(a1)
    beqz t0, end
    sb t0, 0(a0)
    addi a1, a1, 1
    j print_str

end:
    nop
