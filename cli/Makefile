prefix ?= $(DESTDIR)/usr
exec_prefix ?= $(prefix)

bindir ?= $(exec_prefix)/bin
mandir ?= $(prefix)/share/man

TARGET_NAME := fancy-cli
NAME := fancy

all:
	cargo build --release
	pandoc --standalone --to man extra/fancy.1.md -o fancy.1
	gzip fancy.1

test:
	cargo test --release

install:
	[ -e "../Cargo.toml" ] && install -Dm755 -s ../target/release/"$(TARGET_NAME)" "$(bindir)/$(NAME)" || install -Dm755 -s target/release/"$(TARGET_NAME)" "$(bindir)/$(NAME)"
	install -Dm644 fancy.1.gz "$(mandir)/man1/fancy.1.gz"

uninstall:
	rm "$(bindir)/$(NAME)"

clean:
	rm -rf fancy.1.gz
