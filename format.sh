cargo fmt
cargo sort

for dir in ./crates/*/
do
    cd $dir
    echo $(pwd)
    cargo sort
    cd ../..
done
