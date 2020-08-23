BINDIR := /usr/bin
UNITSDIR := /usr/lib/systemd/system
DBUSDIR := /etc/dbus-1/system.d
NAME := fancy-service

all:
	cargo build --release --locked

check:
	cargo test --release --locked

install:
	[[ -e "../Cargo.toml" ]] && install -Dm744 -s ../target/release/"$(NAME)" "$(BINDIR)/fancyd" || install -Dm744 -s target/release/"$(NAME)" "$(BINDIR)/fancyd"
	install -Dm644 .rpm/fancy.service "$(UNITSDIR)/fancy.service"
	install -Dm644 .rpm/fancy-sleep.service "$(UNITSDIR)/fancy-sleep.service"
	install -Dm644 .rpm/com.musikid.fancy.conf "$(DBUSDIR)/com.musikid.fancy.conf"
	install -Dm644 .rpm/configs/* -t "/etc/fancy/configs" 

uninstall:
	rm "$(BINDIR)/fancyd"
	rm "$(UNITSDIR)/fancy.service"
	rm "$(UNITSDIR)/fancy-sleep.service"
	rm "$(DBUSDIR)/com.musikid.fancy.conf"
	rm -rf "/etc/fancy/configs" 
