.PHONY: debug release clean all web watch build-web

build-web: release
	npm install
	npm run build


debug:
	cd amogus && \
	wasm-pack build

release:
	cd amogus && \
	wasm-pack build --out-dir pkg-release --release

clean:
	rm -rf amogus/pkg amogus/pkg-release amogus/target

web: release
	npm run dev

all: debug release

watch:
	cd amogus && \
	cargo watch -i pkg -i pkg-release -s "make -C .. release"
