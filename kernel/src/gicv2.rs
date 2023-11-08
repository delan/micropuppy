use core::ptr;

pub struct Distributor(*mut u32);
pub struct CpuInterface(*mut u32);

/// GIC interrupt ID, 0 through 1023.
pub struct InterruptId(pub usize);

/// PPI number, as found in devicetree.
pub struct PpiNumber(pub usize);

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

    pub fn enable_ppi(&mut self, interrupt_id: impl Into<InterruptId>) {
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
}

impl From<PpiNumber> for InterruptId {
    fn from(value: PpiNumber) -> Self {
        Self(value.0 + 0x10)
    }
}
