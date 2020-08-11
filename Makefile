build : build-keyrd build-rust-bins

build/build.ninja :
ifeq (,$(wildcard ./build/build.ninja))
	meson build
endif

build-keyrd : build/build.ninja
	ninja -C build

build-rust-bins :
	cargo build --release

install :
	cp build/keyrd /usr/local/bin
	chmod a+s /usr/local/bin/keyrd
	cp target/release/keyr-sync target/release/keyr-fmt /usr/local/bin

uninstall :
	rm -rf /usr/local/bin/keyrd
	rm -rf /usr/local/bin/keyrf
	rm -rf /usr/local/bin/keyrr

clean :
	ninja -C build clean
	cargo clean

.PHONY : clean build build-keyrd build-rust-bins install
