# Fancy

<p align="center">
  <img alt="Logo" src="assets/logo.svg">
</p>
<h3 align="center">Control your laptop fans with a fancy ( ͡° ͜ʖ ͡°) software.</h3>

<br>

[![Tests](https://github.com/MusiKid/fancy/actions/workflows/test.yml/badge.svg?branch=develop)](https://github.com/MusiKid/fancy/actions/workflows/test.yml)
[![license](https://img.shields.io/badge/license-MPL--2.0-blue)](LICENSE)
[![Copr build status](https://copr.fedorainfracloud.org/coprs/musikid/Fancy/package/fancy/status_image/last_build.png)](https://copr.fedorainfracloud.org/coprs/musikid/Fancy/package/fancy/)
[![Release](https://img.shields.io/github/v/release/musikid/fancy)](https://github.com/MusiKid/fancy/releases/latest)
[![codecov](https://codecov.io/github/MusiKid/fancy/branch/develop/graph/badge.svg)](https://codecov.io/github/MusiKid/fancy)

---

Fancy is a set of software which allows you to control your laptop fans.
It includes a service daemon to allow accessing to the [EC](https://en.wikipedia.org/wiki/Embedded_controller#Tasks)
and controlling it through D-Bus, a CLI to send commands and a GUI (WIP).
It works only on Linux and Windows support is not planned<sup>[1](#linux-only)</sup>.

## Warning

This project is currently being heavily refactored. If you want a more stable software, check out [nbfc-linux](https://github.com/nbfc-linux/nbfc-linux).

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

Check if the config was correctly set:

```sh
fancy get config
```

You can then set the fan speed. For example, to make it silent:

```sh
fancy set -f 0
```

Or if you have more fans:

```sh
fancy set -f 0 0
```

Back again to automatic:

```sh
fancy set -a
```

Back again to manual:

```sh
fancy set -m
```

## Documentation

You can take a look at the [book](https://musikid.github.io/fancy/).
For the CLI, the available commands are detailed in the fancy(1) man file (`man fancy`).

## FAQ

### Why? NBFC can also do it

NBFC is a great software (also one of the cleanest codebase I ever seen).
However, it's written in C#, which means that it depends on `mono` runtime on Linux.
`mono` is a pretty huge dependency,
especiallly when NBFC is the only thing which needs it, and uses a lot of RAM.
That's the reason why I started to write `fancy`
(and also because I wanted to test my Rust (<sub>kind of</sub>) "skills").

### Linux ONLY?

_Well_, [NBFC](https://github.com/hirschmann/nbfc) is already _well_ integrated
with the Windows "ecosystem", since C# is more common on Windows.
It works very _well_, so go check it. If you want to provide Windows support,
you are also *wel*come.

## License

The project is licensed under MPL-2.0.
You implicitly accept it when you send a pull request.

_WARNING:_ The configurations are from NBFC, which is under GPLv3 license.

## Contributing

Please see [CONTRIBUTING.md](https://github.com/MusiKid/fancy/blob/master/CONTRIBUTING.md).
You can also take a look at the [book](https://musikid.github.io/fancy/)
for details on the internal working.

## Credits

Thanks to [@hirschmann](https://github.com/hirschmann/) for creating [NBFC](https://github.com/hirschmann/nbfc),
where I shamelessly stolen some pieces of code (open source ¯\\_(ツ)_/¯),
and all the contributors who created the configurations.

## Warning

I'm not responsible if your computer start to smell fried chicken,
if your house is being assaulted by the SWAT because your laptop is becoming a nuclear power plant,
if your head blow up because you saw a portion of the source code,
and blabla...
