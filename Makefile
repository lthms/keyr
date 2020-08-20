build : build-keyr-daemon build-rust-bins

build/build.ninja :
ifeq (,$(wildcard ./build/build.ninja))
	meson build
endif

build-keyr-daemon : build/build.ninja
	ninja -C build

build-rust-bins :
	cargo build --release

install :
	cp build/keyr-daemon /usr/local/bin
	chmod a+s /usr/local/bin/keyr-daemon
	cp target/release/keyr-agent /usr/local/bin

uninstall :
	rm -rf /usr/local/bin/keyr-daemon
	rm -rf /usr/local/bin/keyr-agent

clean :
	ninja -C build clean
	cargo clean

.PHONY : clean build build-keyr-daemon build-rust-bins install
