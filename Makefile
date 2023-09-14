all:
	riscv32-unknown-elf-gcc -march=rv32i -mabi=ilp32 -nostdlib -ffreestanding -nostartfiles -o test.elf test.s
	riscv32-unknown-elf-objcopy -O binary test.elf test.bin
	riscv32-unknown-elf-objdump -d -Mno-aliases test.elf > test.dump

.PHONY: all
