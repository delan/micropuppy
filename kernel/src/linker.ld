ENTRY(_start)

RAM_BASE = 0x40000000;
PHYS_BASE = 0xffff000000000000;
KERNEL_BASE = 0xffff800000000000;

MEMORY {
    /* intentionally offset from the start of RAM: space required for FDT */
    ram (rwxa) : ORIGIN = RAM_BASE + 4M, LENGTH = 64M

    /*
        higher half kernel: when TCR_EL1.T1SZ = 16, this entire region is translated by TTBR1_EL1
        since IA[55] = 1
    */
    kernel (rwxa) : ORIGIN = KERNEL_BASE, LENGTH = 8192M
}

SECTIONS {
    /* .start must be >ram because _start must be a physical address */
    .start : {
        _start_pa = LOADADDR(.start);
        *(.start*)
    } >ram

    /*
        the vector table is 16 entries of 0x80 bytes each; VBAR_ELn bits 10:0 are RES0, which
        mandates an 0x800 byte alignment of the vector table
    */
    .vectors : ALIGN(4K) {
        _kernel_va = .;
        _kernel_pa = LOADADDR(.vectors);
        /*
            KEEP is required as no symbols inside the section are directly referenced and thus
            --gc-sections yeets the entire vector table
        */
        VECTORS = .;
        KEEP(*(.vectors))
    } >kernel AT >ram

    .text : { *(.text*) } >kernel AT >ram
    .data : { *(.data*) } >kernel AT >ram
    .rodata : { *(.rodata*) } >kernel AT >ram
    .bss : { *(.bss*) } >kernel AT >ram

    /* sp must be aligned to 16 bytes at a public interface or when used to access memory */
    .stack ALIGN(16) (NOLOAD) : {
        . = . + 0x8000;
        _estack_pa = LOADADDR(.stack) + SIZEOF(.stack);
        _estack_va = .;
    } >kernel AT >ram
    .task1 ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK1_INITIAL_SP = .;
    } >kernel AT >ram
    .task1_kernel ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK1_KERNEL_INITIAL_SP = .;
    } >kernel AT >ram
    .task2 ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK2_INITIAL_SP = .;
    } >kernel AT >ram
    .task2_kernel ALIGN(16) (NOLOAD) : {
        . = . + 0x4000;
        TASK2_KERNEL_INITIAL_SP = .;
    } >kernel AT >ram
    /* TODO move this to rust, so we can calculate the correct space
       and map more pages if needed */
    .buddy_alloc_tree ALIGN(4K) (NOLOAD) : {
        _buddy_alloc_tree_va = .;
        _buddy_alloc_tree_pa = LOADADDR(.buddy_alloc_tree);
        . = . + 0x80000; /* 512KiB of space for tree */
    } >kernel AT >ram

    _ekernel_va = .;

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
