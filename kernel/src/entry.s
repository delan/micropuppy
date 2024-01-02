.extern INITIAL_SP // defined in linker.ld
.equ PSCI_SYSTEM_OFF, 0x84000008

.section ".start", "ax"

.globl _start
_start:
    mov x0, #0x9000000
    mov w1, #'u'
    mov w2, #'p'
    strb w1, [x0]               // “u”
    strb w2, [x0]               // “p”
    strb w1, [x0]               // “u”
    strb w2, [x0]               // “p”
    mov w1, #'!'
    mov w2, #'\n'

    // lower VA range, level 0
    ldr x0, =tt_lower_level0
    msr TTBR0_EL1, x0
    mov x1, #0 // index (TODO: use ubfx and VA)
    ldr x2, =tt_lower_level1
    orr x2, x2, #0b11 // D_Table
    str x2, [x0, x1, lsl #3]

    // lower VA range, level 1 (index 0)
    ldr x0, =tt_lower_level1
    mov x1, #0 // index (TODO: use ubfx and VA)
    mov x2, #0x0000000000000000 // TODO: use VA
    mov x3, #(0b1 << 10) | (0b01 << 0) // AF | D_Block
    orr x2, x2, x3
    str x2, [x0, x1, lsl #3]

    // lower VA range, level 1 (index 1)
    ldr x0, =tt_lower_level1
    mov x1, #1 // index (TODO: use ubfx and VA)
    mov x2, #0x0000000040000000 // TODO: use VA
    mov x3, #(0b1 << 10) | (0b01 << 0) // AF | D_Block
    orr x2, x2, x3
    str x2, [x0, x1, lsl #3]


    // 4KiB translation granule
    //   level -1: IA[51:48] (4-bit)
    //   level  0: IA[47:39] (9-bit)
    //   level  1: IA[38:30] (9-bit)
    //   level  2: IA[29:21] (9-bit)
    //   level  3: IA[20:12] (9-bit)

    // === populate levels 0 to 2 ===
    // level 0: D_Table pointing to level 1
    ldr x0, =tt_upper_level0
    ldr x1, =_kernel_va
    ldr x2, =tt_upper_level1
    ubfx x3, x1, #39, #9 // IA[47:39]
    orr x4, x2, #0b11 // D_Table
    str x4, [x0, x3, lsl #3]

    msr TTBR1_EL1, x0

    // level 1: D_Table pointing to level 2
    mov x0, x2
    ldr x2, =tt_upper_level2
    ubfx x3, x1, #30, #9 // IA[38:30]
    orr x4, x2, #0b11 // D_Table
    str x4, [x0, x3, lsl #3]

    // level 2: D_Table pointing to level 3
    mov x0, x2
    ldr x2, =tt_upper_level3
    ubfx x3, x1, #21, #9 // IA[29:21]
    orr x4, x2, #0b11 // D_Table
    str x4, [x0, x3, lsl #3]

    // === populate level 3 ===
    mov x0, x2
    ldr x2, =_kernel_pa
    ldr x5, =_ekernel_va
    mov x6, #(1 << 10) | 0b11 // AF | D_Page
.populate_level3:
    ubfx x3, x1, #12, #9 // IA[20:12]
    orr x4, x2, x6
    str x4, [x0, x3, lsl #3]

    add x1, x1, #0x1000
    add x2, x2, #0x1000
    cmp x1, x5
    blt .populate_level3

    // need to barrier after writing to any translation tables (R[SPVBD]),
    // or the new contents may not be observable by the mmu.
    dsb sy

    // need to set both half regions to 2^48, because values that are too small
    // (that is, regions that are too big) may yield L0TF exceptions (R[SXWGM]).
    mrs x5, TCR_EL1
    ldr x5, =((16 << 16) | (16 << 0))   // T1SZ = 16, T0SZ = 16
    msr TCR_EL1, x5

    mrs x5, SCTLR_EL1
    orr x5, x5, #1              // mmu enable
.enable_mmu:
    msr SCTLR_EL1, x5

    ldr x30, =_estack_va
    mov sp, x30
    bl kernel_main

    ldr x0, =PSCI_SYSTEM_OFF
    hvc #0

.align 12
tt_lower_level0:
    .fill 512, 8, 0
.align 12
tt_lower_level1:
    .fill 512, 8, 0
.align 12
tt_upper_level0:
    .fill 512, 8, 0
.align 12
tt_upper_level1:
    .fill 512, 8, 0
.align 12
tt_upper_level2:
    .fill 512, 8, 0
.align 12
tt_upper_level3:
    .fill 512, 8, 0

.section ".vectors", "ax"

.macro define_vector_trampoline, addr:req, source:req, type:req
    .org \addr
    vector_\source\()_\type\()_trampoline:
        b vector_\source\()_\type\()_wrapper
.endm

define_vector_trampoline 0x000, el1_sp0, synchronous
define_vector_trampoline 0x080, el1_sp0, irq
define_vector_trampoline 0x100, el1_sp0, fiq
define_vector_trampoline 0x180, el1_sp0, serror

define_vector_trampoline 0x200, el1_sp1, synchronous
define_vector_trampoline 0x280, el1_sp1, irq
define_vector_trampoline 0x300, el1_sp1, fiq
define_vector_trampoline 0x380, el1_sp1, serror

define_vector_trampoline 0x400, el0_a64, synchronous
define_vector_trampoline 0x480, el0_a64, irq
define_vector_trampoline 0x500, el0_a64, fiq
define_vector_trampoline 0x580, el0_a64, serror

define_vector_trampoline 0x600, el0_a32, synchronous
define_vector_trampoline 0x680, el0_a32, irq
define_vector_trampoline 0x700, el0_a32, fiq
define_vector_trampoline 0x780, el0_a32, serror

.section ".text"

// **These macros MUST be kept in sync with the `Context` struct defined in `task.rs`.**
.macro task_save
    sub sp, sp, #0x110

    // GPRs x0 through x29 => context.gprs[0] through context.gprs[29]
    stp x0, x1, [sp, #0x00]
    stp x2, x3, [sp, #0x10]
    stp x4, x5, [sp, #0x20]
    stp x6, x7, [sp, #0x30]
    stp x8, x9, [sp, #0x40]
    stp x10, x11, [sp, #0x50]
    stp x12, x13, [sp, #0x60]
    stp x14, x15, [sp, #0x70]
    stp x16, x17, [sp, #0x80]
    stp x18, x19, [sp, #0x90]
    stp x20, x21, [sp, #0xa0]
    stp x22, x23, [sp, #0xb0]
    stp x24, x25, [sp, #0xc0]
    stp x26, x27, [sp, #0xd0]
    stp x28, x29, [sp, #0xe0]

    // GPR x30 => context.gprs[30]
    // PSTATE => context.psr (we can clobber x0, since it's already been saved)
    mrs x0, SPSR_EL1
    stp x30, x0, [sp, #0xf0]

    // PC => context.pc
    // SP => context.sp
    mrs x0, ELR_EL1
    mrs x1, SP_EL0
    stp x0, x1, [sp, #0x100]
.endm

.macro task_restore
    // context.sp => SP
    // context.pc => PC
    ldp x0, x1, [sp, #0x100]
    msr SP_EL0, x1
    msr ELR_EL1, x0

    // context.psr => PSTATE
    // GPR x30 => context.gprs[30] (we can clobber x0 since it hasn't been restored yet)
    ldp x30, x0, [sp, #0xf0]
    msr SPSR_EL1, x0

    // context.gprs[0] through context.gprs[29] => GPRs x0 through x29
    ldp x28, x29, [sp, #0xe0]
    ldp x26, x27, [sp, #0xd0]
    ldp x24, x25, [sp, #0xc0]
    ldp x22, x23, [sp, #0xb0]
    ldp x20, x21, [sp, #0xa0]
    ldp x18, x19, [sp, #0x90]
    ldp x16, x17, [sp, #0x80]
    ldp x14, x15, [sp, #0x70]
    ldp x12, x13, [sp, #0x60]
    ldp x10, x11, [sp, #0x50]
    ldp x8, x9, [sp, #0x40]
    ldp x6, x7, [sp, #0x30]
    ldp x4, x5, [sp, #0x20]
    ldp x2, x3, [sp, #0x10]
    ldp x0, x1, [sp, #0x00]

    add sp, sp, #0x110
.endm

.macro define_vector_stub, source:req, type:req
    vector_\source\()_\type\()_wrapper:
        eret
.endm

.macro define_vector_task, source:req, type:req
    vector_\source\()_\type\()_wrapper:
        task_save
        mov x0, sp

        bl vector_\source\()_\type

        mov sp, x0
        task_restore
        eret
.endm

// Exception taken from EL1 with SP_EL0
define_vector_stub el1_sp0, synchronous
define_vector_stub el1_sp0, irq
define_vector_stub el1_sp0, fiq
define_vector_stub el1_sp0, serror

// Exception taken from EL1 with SP_EL1
define_vector_stub el1_sp1, synchronous
define_vector_stub el1_sp1, irq
define_vector_stub el1_sp1, fiq
define_vector_stub el1_sp1, serror

// Exception taken from EL0 using AArch64
define_vector_task el0_a64, synchronous
define_vector_task el0_a64, irq
define_vector_task el0_a64, fiq
define_vector_task el0_a64, serror

// Exception taken from EL0 using AArch32
define_vector_stub el0_a32, synchronous
define_vector_stub el0_a32, irq
define_vector_stub el0_a32, fiq
define_vector_stub el0_a32, serror

.global task_start
task_start:
    mov sp, x0
    task_restore
    eret
