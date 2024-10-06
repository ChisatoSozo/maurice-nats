rm -rf out

mkdir -p out/ts
mkdir -p out/rs
mkdir -p out/py

flatc --rust -o out/rs -I schemas schemas/*.fbs
flatc --python -o out/py -I schemas schemas/*.fbs
flatc --ts -o out/ts -I schemas schemas/*.fbs