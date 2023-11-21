.global _start
.section .text.start
_start:
    lui	a0,0x80000
    j _start
