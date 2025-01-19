OUTPUT_ARCH(riscv)
ENTRY(entry)
SECTIONS
{
    .text 0xC0000000: {
        *(.text*) 
        . = ALIGN(8);
    }
    .rodata : {
        . = ALIGN(8);
        *(.rodata*)
        *(.srodata*)
        . = ALIGN(8);
    }
    .data : { 
        . = ALIGN(8);
        *(.sdata*)
        *(.data*)
        . = ALIGN(8);
    } 
    .bss : {
        . = ALIGN(8);
        *(.sbss*)
        *(.bss*)
        *(COMMON)
        . = ALIGN(8);
    }
    . = ALIGN(4096);
    .kstack : {
        . = ALIGN(8);
        . += 5*4096;
        _stack_start = .;
    }
    _end = .;
}
