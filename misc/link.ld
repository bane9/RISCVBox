OUTPUT_ARCH( "riscv" )
ENTRY(_start)

MEMORY
{
  RAM (rwx)  : ORIGIN = 0x80000000, LENGTH = 64K
  TOHOST (rw) : ORIGIN = 0x01000000, LENGTH = 1k
}


SECTIONS
{
   .text :
  {
    . = ALIGN(8);
    *(.text.init*)
    *(.text)
    *(.text*)
    . = ALIGN(8);
    _etext = .;
  } >RAM

  .rodata : {
      *(.rdata)
      *(.rodata .rodata.*)
  } >RAM

  .bss :
  {
    _sbss = .;
    *(.bss)
    *(.bss*)
    . = ALIGN(8);
    _ebss = .;
  } >RAM

  _sidata = LOADADDR(.data);

  .data :
  {
    . = ALIGN(8);
    _sdata = .;
    *(.data)
    *(.data*)
    . = ALIGN(8);
    _edata = .;
    PROVIDE(__global_pointer$ = . + 0x800);
    *(.sdata .sdata.* .sdata2.*)
    PROVIDE(_end = .);
  } >RAM

  .tohost :
  { 
    *(.tohost)
  } >TOHOST AT >RAM

  /DISCARD/ : {
    *(.riscv.attributes)
  }
}
