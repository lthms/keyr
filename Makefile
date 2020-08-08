BINDIR := bin
MUD := ${BINDIR}/mud

WARGS := -Wextra -Wpedantic -Wall
LARGS := -l udev -linput

${MUD} : mud/main.c
	@mkdir -p ${BINDIR}
	gcc $< -o $@ ${WARGS} ${LARGS}

.PHONY : clean

clean :
	rm -rf bin/
