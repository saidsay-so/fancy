# Fancy

<p align="center">
  <img alt="Logo" src="assets/logo.svg">
</p>
<h3 align="center">Control your laptop fans with a fancy ( ͡° ͜ʖ ͡°) software.</h3>

<br>

[![Tests](https://github.com/MusiKid/fancy/actions/workflows/test.yml/badge.svg?branch=develop)](https://github.com/MusiKid/fancy/actions/workflows/test.yml)
[![quality](https://img.shields.io/codacy/grade/cfd339dad3bb4ff09c14912ed5f0d64d)](https://app.codacy.com/gh/MusiKid/fancy/dashboard?branch=master)
[![license](https://img.shields.io/badge/license-MPL--2.0-blue)](LICENSE)
[![Copr build status](https://copr.fedorainfracloud.org/coprs/musikid/Fancy/package/fancy/status_image/last_build.png)](https://copr.fedorainfracloud.org/coprs/musikid/Fancy/package/fancy/)
[![codecov](https://codecov.io/github/MusiKid/fancy/branch/develop/graph/badge.svg)](https://codecov.io/github/MusiKid/fancy)
-------

Fancy is a set of software which allows you to control your laptop fans.
It includes a service daemon to allow accessing to the [EC](https://en.wikipedia.org/wiki/Embedded_controller#Tasks) and controlling it through D-Bus, a CLI to send commands and a GUI (WIP). It works only on Linux and Windows support is not planned<sup>[1](#linux-only)</sup>.

## Compatibility

You can check if your computer model is supported by checking if a configuration exists in
[nbfc_configs](https://github.com/MusiKid/nbfc_configs) (or maybe a similar model).

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

### Then, enable the service

```sh
sudo systemctl enable --now fancy fancy-sleep
```

The service should now be running.
However, it's not active since there isn't any config loaded.
You can see the recommended configurations for your computer:

```sh
fancy list --recommended
```

### Apply a config

```sh
fancy set -c "YOUR_COMPUTER_MODEL"
```

You can then set the fan speed. For example, to make it silent:

```sh
fancy set -f 0
```

If you have more fans:

```sh
fancy set -f 0 0
```

Back again to automatic:

```sh
fancy set -a
```

## FAQ

### Why? NBFC can also do it

NBFC is a great software (also one of the cleanest codebase I ever seen).
However, it's written in C#, which means that it depends on `mono` runtime on Linux.
`mono` is a pretty huge dependency,
especiallly when NBFC is the only thing which needs it, and uses a lot of RAM.
That's the reason why I started to write `fancy`
(and also because I wanted to test my Rust (<sub>kind of</sub>) "skills").

### Linux ONLY?

*Well*, [NBFC](https://github.com/hirschmann/nbfc) is already *well* integrated with the Windows "ecosystem", since C# is more common on Windows.
It works very *well*, so go check it. If you want to provide Windows support, you are also *wel*come.

## License

The project is licensed under MPL-2.0. You implicitly accept it when you send a pull request.

*WARNING:* The configurations are from NBFC, which is under GPLv3 license.

## Contributing

Please see [CONTRIBUTING.md](https://github.com/MusiKid/fancy/blob/master/CONTRIBUTING.md).

## Credits

Thanks to [@hirschmann](https://github.com/hirschmann/) for creating [NBFC](https://github.com/hirschmann/nbfc), where I shamelessly stolen some pieces of code (open source ¯\\_(ツ)_/¯), and all the contributors who created the configurations.

## Warning

I'm not responsible if your computer start to smell fried chicken,
if your house is being assaulted by the SWAT because your laptop is becoming a nuclear power plant,
if your head blow up because you saw a portion of the source code,
and blabla...
