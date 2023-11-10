use crate::memory_mapped_register as reg;
use crate::reg::memory_mapped::{PaddingBytes, Register};
use crate::reg::prelude::*;

#[repr(C)]
pub struct Pl011RegisterBlock {
    /// 0x000: UARTDR (Data Register)
    pub dr: Register<UARTDR>,
    /// 0x004: UARTRSR/UARTECR (Receive Status Register/Error Clear Register)
    pub rsr_ecr: Register<u32>,
    /// 0x008-0x014: Reserved
    _0: PaddingBytes<0x10>,
    /// 0x018: UARTFR (Flag Register)
    pub fr: Register<u32>,
    /// 0x01C: Reserved
    _1: PaddingBytes<0x4>,
    /// 0x020: UARTILPR (IrDA Low-Power Counter Register)
    pub ilpr: Register<u32>,
    /// 0x024: UARTIBRD (Integer Baud Rate Register)
    pub ibrd: Register<u32>,
    /// 0x028: UARTFBRD (Fractional Baud Rate Register)
    pub fbrd: Register<u32>,
    /// 0x02C: UARTLCR_H (Line Control Register)
    pub lcr_h: Register<u32>,
    /// 0x030: UARTCR (Control Register)
    pub cr: Register<u32>,
    /// 0x034: UARTIFLS (Interrupt FIFO Level Select Register)
    pub ifls: Register<u32>,
    /// 0x038: UARTIMSC (Interrupt Mask Set/Clear Register)
    pub imsc: Register<u32>,
    /// 0x03C: UARTRIS (Raw Interrupt Status Register)
    pub ris: Register<u32>,
    /// 0x040: UARTMIS (Masked Interrupt Status Register)
    pub mis: Register<u32>,
    /// 0x044: UARTICR (Interrupt Clear Register)
    pub icr: Register<u32>,
    /// 0x048: UARTDMACR (DMA Control Register)
    pub dmacr: Register<u32>,
    /// 0x04C-0x07C: Reserved
    _2: PaddingBytes<0x34>,
    /// 0x080-0x08C: Reserved for test purposes
    _3: PaddingBytes<0x10>,
    /// 0x090-0xFCC: Reserved
    _4: PaddingBytes<0xf40>,
    /// 0xFD0-0xFDC: Reserved for future ID expansion
    _5: PaddingBytes<0x10>,
    /// 0xFE0: UARTPeriphID0; 0xFE4: UARTPeriphID1; 0xFE8: UARTPeriphID2; 0xFEC: UARTPeriphID3
    pub periph_id: [Register<u32>; 4],
    /// 0xFF0: UARTPCellID0; 0xFF4: UARTPCellID1; 0xFF8: UARTPCellID2; 0xFFC: UARTPCellID3
    pub p_cell_id: [Register<u32>; 4],
}

reg! { UARTDR(u32), rwi=0x0000_0000 }

impl RegisterReader<UARTDR> {
    pub fn data(&self) -> u8 {
        self.field(0..=7).try_into().expect("fuck")
    }
}

impl RegisterWriter<UARTDR> {
    pub fn data(&mut self, data: u8) {
        unsafe { self.field(0..=7, data as _) }
    }
}
