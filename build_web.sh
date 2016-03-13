

rm -rf web
mkdir web
# mkdir -p web/doc
mkdir -p web/book
mkdir -p web/book/images

echo "Building..."
cargo doc
cobalt build -s blog -d web
mdbook build book

echo "Copying files over..."
cp -r book/html/* web/book
cp -r book/images/* web/book/images
cp -r target/doc web/doc
