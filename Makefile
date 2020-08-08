BINDIR := bin
MUD := ${BINDIR}/mud
MUF := ${BINDIR}/muf

WARGS := -Wextra -Wpedantic -Wall
LARGS := -l udev -linput

.PHONY : clean build install

build : ${MUD} ${MUF}

${MUD} : mud/main.c
	@mkdir -p ${BINDIR}
	gcc $< -o $@ ${WARGS} ${LARGS}

${MUF} ::
	cargo build --release
	cp target/release/muf $@

clean :
	cargo clean
	rm -rf bin/

install :
	cp ${MUD} /usr/local/bin
	chmod a+s /usr/local/bin/mud
	cp ${MUF} /usr/local/bin

uninstall :
	rm -rf /usr/local/bin/mud
	rm -rf /usr/local/bin/muf
