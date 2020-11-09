BINDIR := $(DESTDIR)/usr/bin
TARGET_NAME := fancy-cli
NAME := fancy

all:
	cargo build --release

test:
	cargo test --release

install:
	[ -e "../Cargo.toml" ] && install -Dm744 -s ../target/release/"$(TARGET_NAME)" "$(BINDIR)/$(NAME)" || install -Dm744 -s target/release/"$(TARGET_NAME)" "$(BINDIR)/$(NAME)"

uninstall:
	rm "$(BINDIR)/$(NAME)"
