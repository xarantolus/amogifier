.PHONY: debug release clean all

debug:
	cd amogus && \
	wasm-pack build

release:
	cd amogus && \
	wasm-pack build --out-dir pkg-release --release

clean:
	rm -rf amogus/pkg amogus/pkg-release amogus/target

all: debug release
