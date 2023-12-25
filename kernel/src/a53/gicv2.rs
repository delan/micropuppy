use num::AsUsize;

use crate::gicv2::InterruptId;
use crate::memory_mapped_register as reg;
use crate::reg::memory_mapped::{PaddingBytes, Register};
use crate::reg::prelude::*;

#[repr(C)]
pub struct DistributorRegisterBlock {
    /// 0x000: GICD_CTLR (Distributor Control Register)
    pub ctlr: Register<GICD_CTLR>,
    /// 0x004: GICD_TYPER (Interrupt Controller Type Register)
    pub typer: Register<u32>,
    /// 0x008: GICD_IIDR (Distributor Implementer Identification Register)
    pub iidr: Register<u32>,
    /// 0x00C-0x01C: Reserved
    _0: PaddingBytes<0x14>,
    /// 0x020-0x03C: IMPLEMENTATION DEFINED registers
    _1: PaddingBytes<0x20>,
    /// 0x040-0x07C: Reserved
    _2: PaddingBytes<0x40>,
    /// 0x080: GICD_IGROUPRnb (Interrupt Group Registers)
    pub igroupr: [Register<u32>; 32],
    /// 0x100-0x17C: GICD_ISENABLERn (Interrupt Set-Enable Registers)
    pub isenabler: [Register<GICD_ISENABLER>; 32],
    /// 0x180-0x1FC: GICD_ICENABLERn (Interrupt Clear-Enable Registers)
    pub icenabler: [Register<u32>; 32],
    /// 0x200-0x27C: GICD_ISPENDRn (Interrupt Set-Pending Registers)
    pub ispender: [Register<u32>; 32],
    /// 0x280-0x2FC: GICD_ICPENDRn (Interrupt Clear-Pending Registers)
    pub icpendr: [Register<u32>; 32],
    /// 0x300-0x37C: GICD_ISACTIVERn (GICv2 Interrupt Set-Active Registers)
    pub isactiver: [Register<u32>; 32],
    /// 0x380-0x3FC: GICD_ICACTIVERn (Interrupt Clear-Active Registers)
    pub icactiver: [Register<u32>; 32],
    /// 0x400-0x7F8: GICD_IPRIORITYRn (Interrupt Priority Registers)
    pub ipriorityr: [Register<u32>; 255],
    /// 0x7FC: Reserved
    _3: PaddingBytes<0x4>,
    /// 0x800-0x81C: GICD_ITARGETSRn (Interrupt Processor Targets Registers)
    pub itargetsr: [Register<u32>; 255],
    /// 0xBFC: Reserved
    _4: PaddingBytes<0x4>,
    /// 0xC00-0xCFC: GICD_ICFGRn (Interrupt Configuration Registers)
    pub icfgr: [Register<u32>; 64],
    /// 0xD00-0xDFC: IMPLEMENTATION DEFINED registers
    _5: PaddingBytes<0x100>,
    /// 0xE00-0xEFC: GICD_NSACRn (Non-secure Access Control Registers, optional)
    pub nsacr: [Register<u32>; 64],
    /// 0xF00: GICD_SGIR (Software Generated Interrupt Register)
    pub sgir: Register<u32>,
    /// 0xF04-0xF0C: Reserved
    _6: PaddingBytes<0xa>,
    /// 0xF10-0xF1C: GICD_CPENDSGIRn (SGI Clear-Pending Registers)
    pub cpendsgir: [Register<u32>; 4],
    /// 0xF20-0xF2C: GICD_SPENDSGIRn (SGI Set-Pending Registers)
    pub spendsgi: [Register<u32>; 4],
    /// 0xF30-0xFCC: Reserved
    _7: PaddingBytes<0xa0>,
    /// 0xFD0-0xFFC:  - ( Identification registers on page 4-119)
    _8: PaddingBytes<0x20>,
}

reg! { GICD_CTLR(u32), rwi=0x0000_0000 }

#[allow(dead_code)]
impl RegisterReader<GICD_CTLR> {
    pub fn enable(&self) -> bool {
        self.bit(0)
    }
}

#[allow(dead_code)]
impl RegisterWriter<GICD_CTLR> {
    pub fn enable(&mut self, enable: bool) {
        unsafe { self.bit(0, enable) }
    }
}

reg! { GICD_ISENABLER(u32), wi=0x0000_0000 }

#[allow(dead_code)]
impl RegisterWriter<GICD_ISENABLER> {
    pub fn set_enable(&mut self, m: usize) {
        unsafe { self.bit(m, true) }
    }
}

#[repr(C)]
pub struct CpuInterfaceRegisterBlock {
    /// 0x0000: GICC_CTLR (CPU Interface Control Register)
    pub ctlr: Register<GICC_CTLR>,
    /// 0x0004: GICC_PMR (Interrupt Priority Mask Register)
    pub pmr: Register<GICC_PMR>,
    /// 0x0008: GICC_BPR (Binary Point Register)
    pub bpr: Register<u32>,
    /// 0x000C: GICC_IAR (Interrupt Acknowledge Register)
    pub iar: Register<GICC_IAR>,
    /// 0x0010: GICC_EOIR (End of Interrupt Register)
    pub eoir: Register<GICC_EOIR>,
    /// 0x0014: GICC_RPR (Running Priority Register)
    pub rpr: Register<u32>,
    /// 0x0018: GICC_HPPIR (Highest Priority Pending Interrupt Register)
    pub hppir: Register<u32>,
    /// 0x001C: GICC_ABPR (Aliased Binary Point Register)
    pub abpr: Register<u32>,
    /// 0x0020: GICC_AIAR (Aliased Interrupt Acknowledge Register)
    pub aiar: Register<u32>,
    /// 0x0024: GICC_AEOIR (Aliased End of Interrupt Register)
    pub aeoir: Register<u32>,
    /// 0x0028: GICC_AHPPIR (Aliased Highest Priority Pending Interrupt Register)
    pub ahppir: Register<u32>,
    /// 0x002C-0x003C: Reserved
    _0: PaddingBytes<0x14>,
    /// 0x0040-0x00CF: IMPLEMENTATION DEFINED registers
    _1: PaddingBytes<0x90>,
    /// 0x00D0-0x00DC: GICC_APRn (Active Priorities Registers)
    pub apr: [Register<u32>; 4],
    /// 0x00E0-0x00EC: GICC_NSAPRn (Non-secure Active Priorities Registers)
    pub nsapr: [Register<u32>; 4],
    /// 0x00ED-0x00F8: Reserved
    _2: PaddingBytes<0xf>,
    /// 0x00FC: GICC_IIDR (CPU Interface Identification Register)
    pub iidr: Register<u32>,
    // for some reason, this gap is left unmentioned
    _3: PaddingBytes<0xf00>,
    /// 0x1000: GICC_DIR (Deactivate Interrupt Register)
    pub dir: Register<u32>,
}

reg! { GICC_CTLR(u32), rwi=0x0000_0000 }

#[allow(dead_code)]
impl RegisterReader<GICC_CTLR> {
    pub fn enable(&self) -> bool {
        self.bit(0)
    }
}

#[allow(dead_code)]
impl RegisterWriter<GICC_CTLR> {
    pub fn enable(&mut self, enable: bool) {
        unsafe { self.bit(0, enable) }
    }
}

reg! { GICC_PMR(u32), rwi=0x0000_0000 }

#[allow(dead_code)]
impl RegisterReader<GICC_PMR> {
    pub fn priority(&self) -> u8 {
        self.field(0..=7) as _
    }
}

#[allow(dead_code)]
impl RegisterWriter<GICC_PMR> {
    pub fn priority(&mut self, priority: u8) {
        unsafe { self.field(0..=7, priority as _) }
    }
}

reg! { GICC_IAR(u32), r }

#[allow(dead_code)]
impl RegisterReader<GICC_IAR> {
    pub fn entire(&self) -> u32 {
        self.bits()
    }
    pub fn cpuid(&self) -> u8 {
        self.field(10..=12) as _
    }
    pub fn interrupt_id(&self) -> InterruptId {
        self.field(0..=9).as_usize().try_into().unwrap()
    }
}

// IHI 0048B.b § 4.4.5 “If software writes the ID of a spurious interrupt to the
// GICC_EOIR, the GIC ignores that write.”
reg! { GICC_EOIR(u32), wi=0x000003FF }

#[allow(dead_code)]
impl RegisterWriter<GICC_EOIR> {
    pub fn entire_iar(&mut self, iar: u32) {
        unsafe { self.bits(iar) }
    }
}
