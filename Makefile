prefix ?= /usr/local

mandir ?= $(prefix)/share/man
INSTALL ?= install

all: cli service man

.PHONY: cli service man

man:
	# Get the list of configurations supported out-of-box
	find service/nbfc_configs/Configs -type f -print0 | xargs -0 -L1 basename -s .xml | sort | sed 's/^/- /g' | cat fancy.7.md - | pandoc --standalone --to man -o fancy.7
	gzip fancy.7

cli:
	make -C cli/

service:
	make -C service/

test:
	make test -C service/
	make test -C cli/

install:
	make install -C service/
	make install -C cli/
	$(INSTALL) -Dm644 fancy.7.gz $(DESTDIR)$(mandir)/man7/fancy.7.gz

uninstall:
	make uninstall -C service/
	make uninstall -C cli/

clean:
	make clean -C service/
	make clean -C cli/
	cargo clean
	rm -rf fancy.7.gz
