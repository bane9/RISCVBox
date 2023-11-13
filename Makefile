all:
	riscv32-unknown-elf-gcc -march=rv32i_zicsr -mno-riscv-attribute -mabi=ilp32 -nostdlib -ffreestanding -nostartfiles -T link.ld -o test.elf test.s
	riscv32-unknown-elf-objcopy -O binary test.elf test.bin
	riscv32-unknown-elf-objdump --disassemble-all -Mno-aliases test.elf > test.dump

debug_jit_ptr:
	cargo b
	lldb -s lldb.txt

.PHONY: all debug_jit_ptr
