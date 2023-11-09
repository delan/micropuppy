use vcell::VolatileCell;

use super::*;

#[repr(transparent)]
pub struct Register<S: RegisterSpec>(VolatileCell<S::Bits>);

impl<S: RegisterSpec + RegisterReadable> Register<S> {
    pub fn read<R>(&self, reader: impl FnOnce(&RegisterReader<S>) -> R) -> R {
        let r = RegisterReader::new(self.0.get());
        reader(&r)
    }
}

impl<S: RegisterSpec + RegisterWritable> Register<S> {
    pub unsafe fn write_zero(&self, writer: impl FnOnce(&mut RegisterWriter<S>)) {
        let mut w = RegisterWriter::zero();
        writer(&mut w);
        self.0.set(w.bits);
    }
}

impl<S: RegisterSpec + RegisterDefault> Register<S> {
    pub fn write_default(&self, writer: impl FnOnce(&mut RegisterWriter<S>)) {
        let mut w = RegisterWriter::default();
        writer(&mut w);
        self.0.set(w.bits);
    }
}
