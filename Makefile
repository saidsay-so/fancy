all: cli service

.PHONY: cli service

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
