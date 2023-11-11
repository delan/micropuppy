use core::mem::size_of;

/// Convert u32 to usize, or compile error if usize is smaller than u32.
/// Unlike try_into + unwrap, this fails even if the value would fit in usize.
pub fn as_usize(value: u32) -> usize {
    const _: () = assert!(size_of::<usize>() >= size_of::<u32>());
    value as usize
}
