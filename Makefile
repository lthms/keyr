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
	cp target/release/keyr-agent /usr/local/bin

uninstall :
	rm -rf /usr/local/bin/keyrd
	rm -rf /usr/local/bin/keyr-agent

clean :
	ninja -C build clean
	cargo clean

.PHONY : clean build build-keyrd build-rust-bins install
