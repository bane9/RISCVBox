ENTRY(_start)

SECTIONS {
    . = 0x00000000;
    .text : {
        *(.text)
    }
    . = ALIGN(4);
    .data : {
        *(.data)
    }
}
