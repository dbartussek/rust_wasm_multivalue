use std::mem::MaybeUninit;
pub use wasm_calling_support_macros::*;

pub unsafe trait MagicArg {
    unsafe fn read() -> Self;
    unsafe fn write(value: Self);
}

unsafe impl<T> MagicArg for MaybeUninit<T>
where
    T: MagicArg,
{
    unsafe fn read() -> Self {
        unsafe { MaybeUninit::new(MagicArg::read()) }
    }

    unsafe fn write(value: Self) {
        unsafe { MagicArg::write(value.assume_init()) }
    }
}

unsafe extern "C" {
    fn wasm_calling_support_read_u8_arg() -> u8;
    fn wasm_calling_support_write_u8_arg(value: u8);
    fn wasm_calling_support_read_u16_arg() -> u16;
    fn wasm_calling_support_write_u16_arg(value: u16);
    fn wasm_calling_support_read_u32_arg() -> u32;
    fn wasm_calling_support_write_u32_arg(value: u32);
    fn wasm_calling_support_read_u64_arg() -> u64;
    fn wasm_calling_support_write_u64_arg(value: u64);

    fn wasm_calling_support_read_f32_arg() -> f32;
    fn wasm_calling_support_write_f32_arg(value: f32);
    fn wasm_calling_support_read_f64_arg() -> f64;
    fn wasm_calling_support_write_f64_arg(value: f64);
}

unsafe impl MagicArg for u8 {
    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_u8_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_u8_arg(value) }
    }
}
unsafe impl MagicArg for u16 {
    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_u16_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_u16_arg(value) }
    }
}
unsafe impl MagicArg for u32 {
    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_u32_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_u32_arg(value) }
    }
}
unsafe impl MagicArg for u64 {
    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_u64_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_u64_arg(value) }
    }
}

unsafe impl MagicArg for f32 {
    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_f32_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_f32_arg(value) }
    }
}
unsafe impl MagicArg for f64 {
    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_f64_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_f64_arg(value) }
    }
}
