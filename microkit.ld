/* Copyright (c) 2026 Edison Lepiten / AIEONYX */
/* Microkit 1.4.1 — exactly ONE loadable ELF segment */
ENTRY(init)

PHDRS {
    load PT_LOAD FLAGS(7);
}

SECTIONS {
    . = 0x400000;
    .text   : { *(.text .text.*) }   :load
    .rodata : { *(.rodata .rodata.*) } :load
    .data   : { *(.data .data.*) }   :load
    .bss    : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } :load

    /DISCARD/ : {
        *(.note.gnu.build-id)
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.comment)
        *(.ARM.exidx*)
        *(.ARM.extab*)
    }
}
