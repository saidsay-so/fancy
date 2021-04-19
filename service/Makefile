prefix ?= /usr/local

exec_prefix ?= $(prefix)

bindir ?= $(exec_prefix)/bin
libdir ?= $(exec_prefix)/lib
sysconfdir ?= /etc
mandir ?= $(prefix)/share/man
INSTALL ?= install

UNITDIR := $(libdir)/systemd/system
DBUSDIR := $(sysconfdir)/dbus-1/system.d

TARGET_NAME := fancy-service
NAME := fancyd

all:
	cargo build --release --locked
	pandoc --standalone --to man extra/fancyd.8.md -o fancyd.8
	gzip fancyd.8

.PHONY: test

test:
	cargo test --release --locked

install:
	[ -e "../Cargo.toml" ] && $(INSTALL) -Dm744 -s ../target/release/$(TARGET_NAME) $(DESTDIR)$(bindir)/$(NAME) || $(INSTALL) -Dm744 -s target/release/$(TARGET_NAME) $(DESTDIR)$(bindir)/$(NAME)
	$(INSTALL) -Dm644 extra/fancy.service $(DESTDIR)$(UNITDIR)/fancy.service
	$(INSTALL) -Dm644 extra/fancy-sleep.service $(DESTDIR)$(UNITDIR)/fancy-sleep.service
	$(INSTALL) -Dm644 extra/com.musikid.fancy.conf $(DESTDIR)$(DBUSDIR)/com.musikid.fancy.conf
	$(INSTALL) -Dm644 nbfc_configs/Configs/* -t $(DESTDIR)$(sysconfdir)/fancy/configs
	$(INSTALL) -Dm644 fancyd.8.gz $(DESTDIR)$(mandir)/man8/fancyd.8.gz

uninstall:
	rm $(DESTDIR)$(bindir)/$(NAME)
	rm $(DESTDIR)$(UNITDIR)/fancy.service
	rm $(DESTDIR)$(UNITDIR)/fancy-sleep.service
	rm $(DESTDIR)$(DBUSDIR)/com.musikid.fancy.conf
	rm -rf $(DESTDIR)$(sysconfdir)/fancy/configs

clean:
	rm -rf fancyd.8.gz
