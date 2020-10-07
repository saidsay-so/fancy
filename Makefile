all: cli service

.PHONY: cli service install uninstall

cli:
	make -C cli/

service:
	make -C service/

install:
	make install -C service/
	make install -C cli/

uninstall:
	make uninstall -C service/
	make uninstall -C cli/
