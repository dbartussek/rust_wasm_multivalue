use wasm_calling_support::wrap_wasm;

#[repr(C)]
#[derive(Default)]
pub struct Foo {
    pub a: u32,
    pub b: u32,
    pub c: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct Bar {
    pub a: u32,
    pub foo: Foo,
}

static TEST: Foo = Foo {a: 1, b: 2, c: 3};

#[unsafe(no_mangle)]
pub fn test() {
    
}

// #[wrap_wasm]
pub fn test_impl(foo: Foo, bar: Bar) -> Bar {
    if foo.c > 42 {
        Bar {
            a: 0,
            foo: Foo {
                a: foo.a + 1,
                b: foo.b + 2,
                c: foo.c + 3,
            },
        }
    } else {
        bar
    }
}

#[repr(C)]
#[derive(Default)]
pub struct Test2 {
    pub a: u64,
    pub b: u16,
}

// #[wrap_wasm]
pub fn test2(tst: Test2) -> Test2 {
    tst
}
