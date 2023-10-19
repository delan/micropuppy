ENTRY(_start)
SECTIONS
{
    . = 0x40080000;
    .text.boot : { *(.text.boot) }
    .text : { *(.text*) }
    .data : { *(.data*) }
    .rodata : { *(.rodata*) }
    .bss : { *(.bss*) }

    . = ALIGN(8);
    . = . + 0x4000;
    LD_STACK_PTR = .;

    /* Debugging: DWARF */
    .debug_abbrev : { *(.debug_abbrev) }
    .debug_info : { *(.debug_info) }
    .debug_aranges : { *(.debug_aranges) }
    .debug_str : { *(.debug_str) }
    .debug_pubnames : { *(.debug_pubnames) }
    .debug_pubtypes : { *(.debug_pubtypes) }
    .debug_frame : { *(.debug_frame) }
    .debug_line : { *(.debug_line) }
    .debug_ranges : { *(.debug_ranges) }
    .debug_loc : { *(.debug_loc) }
    /* Standard ELF sections */
    .symtab : { *(.symtab) }
    .shstrtab : { *(.shstrtab) }
    .strtab : { *(.strtab) }
    .comment : { *(.comment) }
}
