use core::{ptr, mem::size_of};

use byteorder::{BigEndian, ByteOrder};

pub struct Distributor(*mut u32);
pub struct CpuInterface(*mut u32);

/// GIC interrupt ID, 0 through 1023.
#[derive(Debug)]
pub struct InterruptId(pub usize);

/// Interrupt specifier, as found in devicetree.
///
/// https://www.kernel.org/doc/Documentation/devicetree/bindings/interrupt-controller/interrupts.txt
/// https://github.com/torvalds/linux/blob/305230142ae0637213bf6e04f6d9f10bbcb74af8/Documentation/devicetree/bindings/interrupt-controller/arm%2Cgic.yaml#L71-L93
#[derive(Debug)]
pub struct InterruptSpecifier<'dt>(&'dt [u8]);
#[derive(Debug)]
pub struct PpiNumber(pub usize);
#[derive(Debug)]
pub struct SpiNumber(pub usize);

impl Distributor {
    pub fn new(base_address: *const u8) -> Self {
        Self(base_address as *mut u32)
    }

    pub fn enable(&mut self) {
        unsafe {
            // enable group 0 interrupts (group 1 currently disabled)
            ptr::write_volatile(self.ctlr(), 1);
        }
    }

    pub fn enable_interrupt(&mut self, interrupt_id: impl Into<InterruptId>) {
        unsafe {
            let interrupt_id = interrupt_id.into().0;
            let isenabler = self.isenabler(interrupt_id / 32);
            ptr::write_volatile(isenabler, 1 << (interrupt_id % 32));
        }
    }

    unsafe fn ctlr(&self) -> *mut u32 {
        self.0.add(0)
    }

    unsafe fn isenabler(&self, n: usize) -> *mut u32 {
        self.0.add(64 + n)
    }
}

impl CpuInterface {
    pub fn new(base_address: *const u8) -> Self {
        Self(base_address as *mut u32)
    }

    pub fn enable(&mut self) {
        unsafe {
            // enable group 0 interrupts
            // all other bits zero in IHI 0048B.b, Figure 4-24
            ptr::write_volatile(self.ctlr(), 1);

            // set priority threshold to most lenient
            ptr::write_volatile(self.pmr(), 0xff);
        }
    }

    unsafe fn ctlr(&self) -> *mut u32 {
        self.0.add(0)
    }

    unsafe fn pmr(&self) -> *mut u32 {
        self.0.add(1)
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

    pub fn interrupt_id(&self) -> InterruptId {
        let interrupt_type = BigEndian::read_u32(&self.0[0..]);
        let interrupt_number = BigEndian::read_u32(&self.0[4..]);
        match interrupt_type {
            0 => SpiNumber(as_usize(interrupt_number)).into(),
            1 => PpiNumber(as_usize(interrupt_number)).into(),
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

/// Convert u32 to usize, or compile error if usize is smaller than u32.
/// Unlike try_into + unwrap, this fails even if the value would fit in usize.
fn as_usize(value: u32) -> usize {
    const _: () = assert!(size_of::<usize>() >= size_of::<u32>());
    value as usize
}
