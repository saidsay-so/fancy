CLI_NAME := fancy

SERVICE_NAME := fancy-service
BINDIR := $(DESTDIR)/usr/bin
UNITSDIR := $(DESTDIR)/usr/lib/systemd/system
DBUSDIR := $(DESTDIR)/etc/dbus-1/system.d

all: cli service

.PHONY: cli service install uninstall

cli:
	make -C cli/

service:
	make -C service/

install:
	install -Dm744 -s target/release/"$(CLI_NAME)" "$(BINDIR)/$(CLI_NAME)"
	install -Dm744 -s target/release/"$(SERVICE_NAME)" "$(BINDIR)/fancyd"
	install -Dm644 extra/fancy.service "$(UNITSDIR)/fancy.service"
	install -Dm644 extra/fancy-sleep.service "$(UNITSDIR)/fancy-sleep.service"
	install -Dm644 extra/com.musikid.fancy.conf "$(DBUSDIR)/com.musikid.fancy.conf"
	install -Dm644 nbfc_configs/* -t "$(DESTDIR)/etc/fancy/configs"

uninstall:
	rm "$(BINDIR)/$(CLI_NAME)"
	rm "$(BINDIR)/fancyd"
	rm "$(UNITSDIR)/fancy.service"
	rm "$(UNITSDIR)/fancy-sleep.service"
	rm "$(DBUSDIR)/com.musikid.fancy.conf"
	rm -rf "$(DESTDIR)/etc/fancy/configs"
