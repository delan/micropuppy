ENTRY(_start)
MEMORY
{
    /* 4MiB from the start of RAM - space required at start of RAM for FDT */
    ram (a) : ORIGIN = 0x40400000, LENGTH = 0x4000000

    /* virtual memory, with kernel at VA[55]=1, starting at 2^55 */
    /* vm (a) : ORIGIN = 0x80000000000000, LENGTH = 0x4000000 */
    vm (a) : ORIGIN =  0xFFFF000000000000, LENGTH = 0x4000000
}
SECTIONS
{
    .data.tt0l0 : {
        TT0L0 = .;
        . = . + 0x1000;
        KEEP(*(.data.tt0l0))
    } >ram AT >ram
    .data.tt0l1 : {
        TT0L1 = .;
        . = . + 0x1000;
        KEEP(*(.data.tt0l1))
    } >ram AT >ram
    .data.tt0l2 : {
        TT0L2 = .;
        . = . + 0x1000;
        KEEP(*(.data.tt0l2))
    } >ram AT >ram
    .data.tt0l3 : {
        TT0L3 = .;
        . = . + 0x1000;
        KEEP(*(.data.tt0l3))
    } >ram AT >ram
    .data.tt1l0 : {
        TT1L0 = .;
        . = . + 0x1000;
        KEEP(*(.data.tt1l0))
    } >ram AT >ram
    .data.tt1l1 : {
        TT1L1 = .;
        . = . + 0x1000;
        KEEP(*(.data.tt1l1))
    } >ram AT >ram
    .data.tt1l2 : {
        TT1L2 = .;
        . = . + 0x1000;
        KEEP(*(.data.tt1l2))
    } >ram AT >ram
    .data.tt1l3 : {
        TT1L3 = .;
        . = . + 0x1000;
        KEEP(*(.data.tt1l3))
    } >ram AT >ram
    .text.startup : { *(.text.startup) } >ram

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
    } >vm AT >ram

    .text : { *(.text*) } >vm AT >ram
    .data : { *(.data*) } >vm AT >ram
    .rodata : { *(.rodata*) } >vm AT >ram
    .bss : { *(.bss*) } >vm AT >ram

    /* sp must be aligned to 16 bytes at a public interface or when used to access memory */
    .stack ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        INITIAL_SP = .;
    } >vm AT >ram
    .task1 ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK1_INITIAL_SP = .;
    } >vm AT >ram
    .task1_kernel ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK1_KERNEL_INITIAL_SP = .;
    } >vm AT >ram
    .task2 ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK2_INITIAL_SP = .;
    } >vm AT >ram
    .task2_kernel ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK2_KERNEL_INITIAL_SP = .;
    } >vm AT >ram
    BUDDY_ALLOC_TREE = .;

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
