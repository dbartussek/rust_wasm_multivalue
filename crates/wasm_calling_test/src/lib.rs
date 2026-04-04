use wasm_calling_support::{MagicArg, wrap_wasm};

#[repr(C)]
#[derive(Default, Clone, MagicArg)]
pub struct Test {
    pub a: u64,
    pub b: u16,
}

#[inline(never)]
fn foo() -> Test {
    Test { a: 1000, b: 2000 }
}

#[wrap_wasm]
pub fn test(mut tst: Test) -> Test {
    tst.a += 1;

    if tst.a > 10 {
        return foo();
    }

    tst
}
