use core::mem::size_of;

/// Converts a value to `usize`, or raises a compile error if the **type** is wider than a `usize`.
///
/// Unlike `try_into` + `unwrap`, this fails even if the **value** would fit in `usize`.
pub trait AsUsize {
    fn as_usize(self) -> usize;
}

macro_rules! impl_for {
    ($ty:ty) => {
        impl AsUsize for $ty {
            fn as_usize(self) -> usize {
                const _: () = assert!(size_of::<usize>() >= size_of::<$ty>());

                self as usize
            }
        }
    };
}

impl_for!(u8);
impl_for!(u16);
impl_for!(u32);
