use byteorder::{BigEndian, ByteOrder};

use crate::{a53::gicv2::{CpuInterfaceRegisterBlock, DistributorRegisterBlock}, num::as_usize};

macro_rules! bounds_checked {
    ($(#[$meta:meta])* $vis:vis struct $name:ident ($int:ident ($low:literal ..= $high:literal))) => {
        $(#[$meta])* $vis struct $name($int);
        impl $name {
            pub fn value(&self) -> $int {
                self.0
            }
        }
        impl TryFrom<$int> for $name {
            type Error = ();
            fn try_from(inner: $int) -> Result<Self, Self::Error> {
                match inner {
                    $low ..= $high => Ok(Self(inner)),
                    _ => Err(()),
                }
            }
        }
    };
    ($($(#[$meta:meta])* $vis:vis struct $name:ident ($($details:tt)+);)+) => {
        $(bounds_checked!($(#[$meta])* $vis struct $name ($($details)+));)+
    };
}

pub struct Distributor(*mut DistributorRegisterBlock);
pub struct CpuInterface(*mut CpuInterfaceRegisterBlock);

/// Interrupt specifier, as found in devicetree.
///
/// https://www.kernel.org/doc/Documentation/devicetree/bindings/interrupt-controller/interrupts.txt
/// https://github.com/torvalds/linux/blob/305230142ae0637213bf6e04f6d9f10bbcb74af8/Documentation/devicetree/bindings/interrupt-controller/arm%2Cgic.yaml#L71-L93
#[derive(Debug)]
pub struct InterruptSpecifier<'dt>(&'dt [u8]);

bounds_checked! {
    /// GIC interrupt ID.
    #[derive(Clone, Copy, Debug, PartialEq)] pub struct InterruptId(usize (0..=1023));

    /// Zero-based PPI number, as found in devicetree.
    #[derive(Clone, Copy, Debug, PartialEq)] pub struct PpiNumber(usize (0..=15));

    /// Zero-based SPI number, as found in devicetree.
    #[derive(Clone, Copy, Debug, PartialEq)] pub struct SpiNumber(usize (0..=987));
}

impl Distributor {
    pub const fn new(base_address: *const u8) -> Self {
        Self(base_address as *mut DistributorRegisterBlock)
    }

    pub fn enable(&mut self) {
        let gicd = unsafe { &*self.0 };

        // enable group 0 interrupts (group 1 currently disabled)
        gicd.ctlr.write_initial(|w| w.enable(true));
    }

    pub fn enable_interrupt(&mut self, interrupt_id: impl Into<InterruptId>) {
        let gicd = unsafe { &*self.0 };

        let interrupt_id = interrupt_id.into().0;
        let (n, m) = (interrupt_id / 32, interrupt_id % 32);

        gicd.isenabler[n].write_initial(|w| w.set_enable(m));
    }
}

impl CpuInterface {
    pub const fn new(base_address: *const u8) -> Self {
        Self(base_address as *mut CpuInterfaceRegisterBlock)
    }

    pub fn enable(&mut self) {
        let gicc = unsafe { &*self.0 };

        // enable group 0 interrupts
        gicc.ctlr.write_initial(|w| w.enable(true));

        // set priority threshold to most lenient
        gicc.pmr.write_initial(|w| w.priority(0xff));
    }

    /// Acknowledge an interrupt, returning the entire GICC_IAR, the cpuid, and the interrupt id.
    pub fn acknowledge(&mut self) -> (u32, u8, InterruptId) {
        let gicc = unsafe { &mut *self.0 };
        gicc.iar.read(|r| (r.entire(), r.cpuid(), r.interrupt_id()))
    }

    /// Deactivate an interrupt, given its entire GICC_IAR as recommended by the GICC_EOIR docs.
    pub fn deactivate(&mut self, iar: u32) {
        let mut gicc = unsafe { &mut *self.0 };
        gicc.eoir.write_initial(|w| w.entire_iar(iar))
    }
}

impl InterruptId {
    pub const fn spurious() -> Self {
        Self(1023)
    }
}

impl From<PpiNumber> for InterruptId {
    fn from(value: PpiNumber) -> Self {
        Self(value.0 + 0x10)
    }
}

impl From<SpiNumber> for InterruptId {
    fn from(value: SpiNumber) -> Self {
        Self(value.0 + 0x20)
    }
}

impl InterruptSpecifier<'_> {
    pub fn interrupts_iter(interrupts: &[u8]) -> InterruptSpecifierIter {
        InterruptSpecifierIter(interrupts)
    }

    pub fn interrupt_id(&self) -> Result<InterruptId, ()> {
        let interrupt_type = BigEndian::read_u32(&self.0[0..]);
        let interrupt_number = BigEndian::read_u32(&self.0[4..]);
        match interrupt_type {
            0 => Ok(SpiNumber::try_from(as_usize(interrupt_number))?.into()),
            1 => Ok(PpiNumber::try_from(as_usize(interrupt_number))?.into()),
            _ => panic!(),
        }
    }
}

pub struct InterruptSpecifierIter<'dt>(&'dt [u8]);
impl<'dt> Iterator for InterruptSpecifierIter<'dt> {
    type Item = InterruptSpecifier<'dt>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() < 12 {
            return None;
        }
        let result = InterruptSpecifier(self.0);
        self.0 = &self.0[12..];
        Some(result)
    }
}
