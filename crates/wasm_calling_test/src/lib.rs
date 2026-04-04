use std::ptr::null_mut;
use wasm_calling_support::{MagicArg, wrap_wasm};

#[repr(C)]
#[derive(Default, Clone, MagicArg)]
pub struct Test {
    pub a: u64,
    pub b: u16,

    pub array: [u8; 2],

    pub ptr: *mut u8,

    pub signed: i32,

    pub size: usize,

    pub float: f32,
}

#[inline(never)]
fn foo() -> Test {
    Test {
        a: 1000,
        b: 2000,

        array: [2; 2],

        ptr: null_mut(),

        signed: -42,
        size: 128,

        float: -42.0,
    }
}

#[wrap_wasm]
pub fn test(mut tst: Test) -> Test {
    tst.a += 1;

    if tst.a > 10 {
        return foo();
    }

    tst
}
