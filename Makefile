.PHONY: debug release clean all web watch build-web

build-web: release
	npm install
	npm run build

debug:
	cd amogus && \
	wasm-pack build

release:
	cd amogus && \
	RUSTFLAGS="-C target-cpu=native -C opt-level=s -C lto=fat -C panic=abort -C codegen-units=1" \
	wasm-pack build --out-dir pkg-release --release -- \
	--no-default-features \
	-Z build-std=std,panic_abort \
	-Z build-std-features=panic_immediate_abort

clean:
	rm -rf amogus/pkg amogus/pkg-release amogus/target

web: release
	npm run dev

all: debug release

watch:
	cd amogus && \
	cargo watch -i pkg -i pkg-release -s "make -C .. release"
