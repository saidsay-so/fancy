prefix ?= /usr/local

exec_prefix ?= $(prefix)

bindir ?= $(exec_prefix)/bin
datadir ?= $(prefix)/share
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
	$(INSTALL) -D -p -m 644 $(TARGET_DIR)/release/build/fancy-cli-*/out/fancy.bash $(DESTDIR)$(datadir)/bash-completion/completions/fancy
	$(INSTALL) -D -p -m 644 $(TARGET_DIR)/release/build/fancy-cli-*/out/fancy.fish $(DESTDIR)$(datadir)/fish/vendor_completions.d/fancy.fish
	$(INSTALL) -D -p -m 644 $(TARGET_DIR)/release/build/fancy-cli-*/out/_fancy $(DESTDIR)$(datadir)/zsh/site-functions/_fancy
	$(INSTALL) -Dm644 fancy.1.gz $(DESTDIR)$(mandir)/man1/fancy.1.gz

uninstall:
	rm $(DESTDIR)$(bindir)/$(NAME)
	rm $(DESTDIR)$(datadir)/bash-completion/completions/fancy
	rm $(DESTDIR)$(datadir)/fish/vendor_completions.d/fancy.fish
	rm $(DESTDIR)$(datadir)/zsh/site-functions/_fancy
	rm $(DESTDIR)$(mandir)/man1/fancy.1.gz

clean:
	rm -rf fancy.1.gz
