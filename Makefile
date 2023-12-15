
clean:
	@cargo clean
	@rm -rf ./artifacts

schema:
	ls ./contracts | xargs -n 1 -t beaker wasm ts-gen

build:
	cargo wasm

build-dev: clean
	cargo cw-optimizoor

check: build-dev
	ls -d ./artifacts/*.wasm | xargs -I x cosmwasm-check x

compile-m1:
	@./build_release_m1.sh

compile:
	@./build_release.sh

# build optimized .wasm contracts - faster for m1 macs, but for development only
build-optimized-m1: compile-m1

# build optimized .wasm contracts
build-optimized: compile
