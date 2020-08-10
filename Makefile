BINDIR := bin
KEYRD := ${BINDIR}/keyrd
KEYRF := ${BINDIR}/keyr-sync
KEYRR := ${BINDIR}/keyr-fmt

WARGS := -Wextra -Wpedantic -Wall
LARGS := -l udev -linput

.PHONY : clean build install

build : ${KEYRD} ${KEYRF}

${KEYRD} : keyrd/main.c
	@mkdir -p ${BINDIR}
	gcc $< -o $@ ${WARGS} ${LARGS}

${KEYRF} ${KEYRR} ::
	cargo build --release
	cp target/release/keyr-sync ${KEYRF}
	cp target/release/keyr-fmt ${KEYRR}

clean :
	cargo clean
	rm -rf bin/

install :
	cp ${KEYRD} /usr/local/bin
	chmod a+s /usr/local/bin/keyrd
	cp ${KEYRF} /usr/local/bin
	cp ${KEYRR} /usr/local/bin

uninstall :
	rm -rf /usr/local/bin/keyrd
	rm -rf /usr/local/bin/keyrf
	rm -rf /usr/local/bin/keyrr
