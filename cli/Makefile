prefix ?= /usr/local

exec_prefix ?= $(prefix)

bindir ?= $(exec_prefix)/bin
mandir ?= $(prefix)/share/man
INSTALL ?= install

TARGET_NAME := fancy-cli
TARGET_DIR := $(if $(realpath ../Cargo.toml), ../target, target)
NAME := fancy

all:
	cargo build --release --locked
	pandoc --standalone --to man extra/fancy.1.md -o fancy.1
	gzip -f fancy.1

.PHONY: test install uninstall clean

test:
	cargo test --release --locked

install:
	$(INSTALL) -Dm755 -s $(TARGET_DIR)/release/$(TARGET_NAME) $(DESTDIR)$(bindir)/$(NAME)
	$(INSTALL) -Dm644 fancy.1.gz $(DESTDIR)$(mandir)/man1/fancy.1.gz

uninstall:
	rm $(DESTDIR)$(bindir)/$(NAME)
	rm $(DESTDIR)$(mandir)/man1/fancy.1.gz

clean:
	rm -rf fancy.1.gz
