# Introduction

Fancy is a set of software which allows you to control your laptop fans.
It includes a service daemon to allow accessing to the EC and controlling it through D-Bus,
a CLI to send commands and a GUI (WIP).
It works only on Linux and Windows support is not planned[^linux-only].

## Installation

**NOTE: If you have Secure Boot enabled, you have to install [`acpi_ec`](https://github.com/MusiKid/acpi_ec) or disable it.**

#### Arch Linux (thanks to @[BachoSeven](https://github.com/BachoSeven)!)

```sh
yay -S fancy
```

#### Debian

For now, you can find the `.deb` in the [Releases](https://github.com/MusiKid/fancy/releases/latest).

<!--
```sh
sudo add-apt-repository ppa:musikid/fancy
sudo apt install fancy
```
-->

#### Fedora

```sh
sudo dnf copr enable musikid/Fancy
sudo dnf install fancy
```

#### For other distros

```sh
git clone https://github.com/MusiKid/fancy.git
cd fancy
make && sudo make install
```

[^linux-only]: _Well_, [NBFC](https://github.com/hirschmann/nbfc) is already _well_ integrated with the Windows "ecosystem", since C# is more common on Windows. It works very _well_, so go check it. If you want to provide Windows support, you are also *wel*come.
