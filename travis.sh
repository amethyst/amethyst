MDBOOK_RELEASE="v0.2.1/mdbook-v0.2.1-x86_64-unknown-linux-gnu.tar.gz"

echo "Build and test without profiler"
cargo test --all -v -- --deny missing_docs || exit 1

if [ ${TRAVIS_OS_NAME} = "linux" ]
then
    echo "Install mdbook"
    curl -L -o mdbook.tar.gz https://github.com/rust-lang-nursery/mdBook/releases/download/${MDBOOK_RELEASE}
    tar -xvf mdbook.tar.gz -C ./
    rm mdbook.tar.gz

    echo "Build all the examples in the book"
    ./mdbook test book -L target/debug/deps || exit 2
fi

echo "Build and test with profiler"
cargo test --all --features profiler -v -- --deny missing_docs || exit 3

echo "Build and test with sdl_controller"
cargo test --all --features sdl_controller -v -- --deny missing_docs || exit 4
