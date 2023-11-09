use core::marker::PhantomData;
use core::mem::size_of;

use vcell::VolatileCell;

use super::*;

pub struct Register<S: RegisterSpec> {
    cell: VolatileCell<S::Type>,
    phantom: PhantomData<S>,
}

const _: () = assert!(size_of::<Register<u32>>() == size_of::<u32>());

impl<S: RegisterSpec + RegisterReadable> Register<S> {
    pub fn read<R>(&self, reader: impl FnOnce(&RegisterRead<S>) -> R) -> R {
        let r = RegisterRead::new(self.cell.get());
        reader(&r)
    }
}

impl<S: RegisterSpec + RegisterWritable> Register<S> {
    pub unsafe fn write_zero(&self, writer: impl FnOnce(&mut RegisterWrite<S>)) {
        let mut w = RegisterWrite::zero();
        writer(&mut w);
        self.cell.set(w.bits);
    }
}

impl<S: RegisterSpec + RegisterDefault> Register<S> {
    pub fn write_default(&self, writer: impl FnOnce(&mut RegisterWrite<S>)) {
        let mut w = RegisterWrite::default();
        writer(&mut w);
        self.cell.set(w.bits);
    }
}
