BINDIR := $(DESTDIR)/usr/bin
NAME := fancy

all:
	cargo build --release

test:
	cargo test --release

install:
	[ -e "../Cargo.toml" ] && install -Dm744 -s ../target/release/"$(NAME)" "$(BINDIR)/$(NAME)" || install -Dm744 -s target/release/"$(NAME)" "$(BINDIR)/$(NAME)"

uninstall:
	rm "$(BINDIR)/$(NAME)"
