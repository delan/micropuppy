ENTRY(_start)
SECTIONS
{
    /* 4MiB from the start of RAM - space required at start of RAM for FDT */
    . = 0x40400000;

    /* VBAR_EL1 is 16 vectors of 0x80 bytes */
    . = ALIGN(0x800);
    .text.vectors : { *(.text.vectors) }

    .text.startup : { *(.text.startup) }
    .text : { *(.text*) }
    .data : { *(.data*) }
    .rodata : { *(.rodata*) }
    .bss : { *(.bss*) }

    /* sp must be aligned to 16 bytes at a public interface or when used to access memory */
    .stack ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        INITIAL_SP = .;
    }

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
