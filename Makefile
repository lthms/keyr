BINDIR := bin
MUD := ${BINDIR}/mud
MUF := ${BINDIR}/muf
MUR := ${BINDIR}/mur

WARGS := -Wextra -Wpedantic -Wall
LARGS := -l udev -linput

.PHONY : clean build install

build : ${MUD} ${MUF}

${MUD} : mud/main.c
	@mkdir -p ${BINDIR}
	gcc $< -o $@ ${WARGS} ${LARGS}

${MUF} ${MUR} ::
	cargo build --release
	cp target/release/muf ${MUF}
	cp target/release/mur ${MUR}

clean :
	cargo clean
	rm -rf bin/

install :
	cp ${MUD} /usr/local/bin
	chmod a+s /usr/local/bin/mud
	cp ${MUF} /usr/local/bin
	cp ${MUR} /usr/local/bin

uninstall :
	rm -rf /usr/local/bin/mud
	rm -rf /usr/local/bin/muf
	rm -rf /usr/local/bin/mur
