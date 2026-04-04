clear

cargo fmt

cargo build --package=wasm_calling_test --release --target wasm32-unknown-unknown

cargo run

wasm-opt target/wasm32-unknown-unknown/release/wasm_calling_test_adjusted.wasm -o target/wasm32-unknown-unknown/release/wasm_calling_test_opt.wasm -O4 --inlining-optimizing --traps-never-happen

wasm2wat target/wasm32-unknown-unknown/release/wasm_calling_test.wasm > target/wasm32-unknown-unknown/release/wasm_calling_test.wat
wasm2wat target/wasm32-unknown-unknown/release/wasm_calling_test_adjusted.wasm > target/wasm32-unknown-unknown/release/wasm_calling_test_adjusted.wat
wasm2wat target/wasm32-unknown-unknown/release/wasm_calling_test_opt.wasm > target/wasm32-unknown-unknown/release/wasm_calling_test_opt.wat
