prefix ?= $(DESTDIR)/usr
exec_prefix ?= $(prefix)

bindir ?= $(exec_prefix)/bin
libdir ?= $(exec_prefix)/lib
sysconfdir ?= $(DESTDIR)/etc
mandir ?= $(prefix)/share/man

UNITSDIR := $(libdir)/systemd/system
DBUSDIR := $(sysconfdir)/dbus-1/system.d
TARGET_NAME := fancy-service
NAME := fancyd

all:
	cargo build --release
	pandoc --standalone --to man extra/fancyd.8.md -o fancyd.8
	gzip fancyd.8

test:
	cargo test --release

install:
	[ -e "../Cargo.toml" ] && install -Dm744 -s ../target/release/"$(TARGET_NAME)" "$(bindir)/$(NAME)" || install -Dm744 -s target/release/"$(TARGET_NAME)" "$(bindir)/$(NAME)"
	install -Dm644 extra/fancy.service "$(UNITSDIR)/fancy.service"
	install -Dm644 extra/fancy-sleep.service "$(UNITSDIR)/fancy-sleep.service"
	install -Dm644 extra/com.musikid.fancy.conf "$(DBUSDIR)/com.musikid.fancy.conf"
	install -Dm644 nbfc_configs/Configs/* -t "$(sysconfdir)/fancy/configs"
	install -Dm644 fancyd.8.gz "$(mandir)/man8/fancyd.8.gz"

uninstall:
	rm "$(bindir)/$(NAME)"
	rm "$(UNITSDIR)/fancy.service"
	rm "$(UNITSDIR)/fancy-sleep.service"
	rm "$(DBUSDIR)/com.musikid.fancy.conf"
	rm -rf "$(sysconfdir)/fancy/configs"

clean:
	rm -rf fancyd.8.gz
