ENTRY(_start)
SECTIONS
{
    /* 4MiB from the start of RAM - space required at start of RAM for FDT */
    . = 0x40400000;

    /*
        the vector table is 16 entries of 0x80 bytes each; VBAR_ELn bits 10:0 are RES0, which
        mandates an 0x800 byte alignment of the vector table
    */
    .text.vectors ALIGN(0x800) : {
        VECTORS = .;
        /*
            KEEP is required as no symbols inside the section are directly referenced and thus
            --gc-sections yeets the entire vector table
        */
        KEEP(*(.text.vectors))
    }

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
    .task1 ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK1_INITIAL_SP = .;
    }
    .task1_kernel ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK1_KERNEL_INITIAL_SP = .;
    }
    .task2 ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK2_INITIAL_SP = .;
    }
    .task2_kernel ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK2_KERNEL_INITIAL_SP = .;
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
