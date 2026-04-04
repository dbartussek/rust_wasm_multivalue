pub use wasm_calling_support_macros::*;

pub trait MagicArg {
    fn read(this: *mut Self) -> Self;
    fn write(this: *mut Self, value: Self);
}

#[inline(never)]
pub extern "C" fn magic_read<T>(_: *const T) -> T {
    unreachable!()
}
