BINDIR := bin
MUD := ${BINDIR}/mud

${MUD} : mud/main.c
	@mkdir -p ${BINDIR}
	gcc $< -ludev -linput -o $@
