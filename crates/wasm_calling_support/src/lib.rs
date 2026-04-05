use std::mem::MaybeUninit;
pub use wasm_calling_support_macros::*;

pub unsafe trait MagicArg {
    const NUMBER_OF_ARGS: usize;

    unsafe fn read() -> Self;
    unsafe fn write(value: Self);
}

unsafe impl<T> MagicArg for MaybeUninit<T>
where
    T: MagicArg,
{
    const NUMBER_OF_ARGS: usize = T::NUMBER_OF_ARGS;

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

    fn wasm_calling_support_read_i8_arg() -> i8;
    fn wasm_calling_support_write_i8_arg(value: i8);
    fn wasm_calling_support_read_i16_arg() -> i16;
    fn wasm_calling_support_write_i16_arg(value: i16);
    fn wasm_calling_support_read_i32_arg() -> i32;
    fn wasm_calling_support_write_i32_arg(value: i32);
    fn wasm_calling_support_read_i64_arg() -> i64;
    fn wasm_calling_support_write_i64_arg(value: i64);

    fn wasm_calling_support_read_usize_arg() -> usize;
    fn wasm_calling_support_write_usize_arg(value: usize);
    fn wasm_calling_support_read_isize_arg() -> isize;
    fn wasm_calling_support_write_isize_arg(value: isize);

    fn wasm_calling_support_read_f32_arg() -> f32;
    fn wasm_calling_support_write_f32_arg(value: f32);
    fn wasm_calling_support_read_f64_arg() -> f64;
    fn wasm_calling_support_write_f64_arg(value: f64);

    fn wasm_calling_support_read_ptr_arg() -> *mut ();
    fn wasm_calling_support_write_ptr_arg(value: *const ());
}

unsafe impl MagicArg for u8 {
    const NUMBER_OF_ARGS: usize = 1;

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
    const NUMBER_OF_ARGS: usize = 1;

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
    const NUMBER_OF_ARGS: usize = 1;

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
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_u64_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_u64_arg(value) }
    }
}

unsafe impl MagicArg for i8 {
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_i8_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_i8_arg(value) }
    }
}
unsafe impl MagicArg for i16 {
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_i16_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_i16_arg(value) }
    }
}
unsafe impl MagicArg for i32 {
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_i32_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_i32_arg(value) }
    }
}
unsafe impl MagicArg for i64 {
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_i64_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_i64_arg(value) }
    }
}

unsafe impl MagicArg for usize {
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_usize_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_usize_arg(value) }
    }
}
unsafe impl MagicArg for isize {
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_isize_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_isize_arg(value) }
    }
}

unsafe impl MagicArg for f32 {
    const NUMBER_OF_ARGS: usize = 1;

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
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_f64_arg() }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_f64_arg(value) }
    }
}

unsafe impl<T> MagicArg for *const T {
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_ptr_arg() as *const T }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_ptr_arg(value as *const ()) }
    }
}

unsafe impl<T> MagicArg for *mut T {
    const NUMBER_OF_ARGS: usize = 1;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe { wasm_calling_support_read_ptr_arg() as *mut T }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        unsafe { wasm_calling_support_write_ptr_arg(value as *const ()) }
    }
}

unsafe impl<const N: usize, T> MagicArg for [T; N]
where
    T: MagicArg,
{
    const NUMBER_OF_ARGS: usize = T::NUMBER_OF_ARGS * N;

    #[inline(always)]
    unsafe fn read() -> Self {
        unsafe {
            let mut array = [const { MaybeUninit::<T>::uninit() }; N];

            for entry in array.iter_mut() {
                *entry = MaybeUninit::new(T::read());
            }

            array.map(|it| it.assume_init())
        }
    }

    #[inline(always)]
    unsafe fn write(value: Self) {
        for entry in value {
            unsafe { T::write(entry) };
        }
    }
}
