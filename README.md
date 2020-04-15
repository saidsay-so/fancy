# Fancy

[//]: # (TODO: REMOVE THIS UGLY "LOGO")
![Logo](assets/logo-96.png)
<!-- TODO: Really, add a good logo -->

## Control your laptop fans with a fancy ( ͡° ͜ʖ ͡°) interface.  

Fancy is a set of software which allows you to control your laptop fans. It includes a service daemon to allow accessing to the [EC](https://en.wikipedia.org/wiki/Embedded_controller#Tasks) and controlling it through D-Bus, a CLI to send commands, a GUI (WIP). It's Linux-only and Windows support is not planned<sup>[1](#linux-only)</sup>.

## Compatibility
You can check if your computer model is supported by checking if a configuration exists in `nbfc_configs` (or a similar model).

## Installation

#### NOTE: If you have Secure Boot enabled, you have to instead install [`acpi_ec`](https://github.com/MusiKid/acpi_ec).
#### NOTE: Users of Arch Linux (with no Secure Boot) or distros which builds ec_sys directly in the kernel (`modinfo ec_sys | grep filename` returns builtin), should instead add `ec_sys.write_support=1` to the kernel command line parameters (https://wiki.archlinux.org/index.php/Kernel_parameters) for the first step.
### First, enable `ec_sys` kernel module to allow `fancy` to access to the EC.
```sh
sudo sh -c "echo ec_sys >> /etc/modules-load.d/ec_sys.conf"
sudo sh -c "echo 'options ec_sys write_support=1' >> /etc/modprobe.d/ec_sys-write-support.conf"
sudo systemctl restart systemd-modules-load.service
```

### After that, install `fancy`:
##### Debian:
```sh
sudo add-apt-repository ppa:musikid/fancy
sudo apt install fancy-service fancy-cli # fancy-gui
```

##### Fedora:
```sh
sudo dnf copr enable musikid/fancy
sudo dnf install fancy-service fancy-cli # fancy-gui
```

### Then, enable the service:
```sh
sudo systemctl enable --now fancy fancy-sleep
```

The service should be running. However, it's not active since there isn't any config loaded. 
You can see the recommended configurations for your computer:
```sh
fancy list --recommended
```

### Apply a config:
```sh
fancy set -c "RECOMMENDED"
```

You can then set the fan(s) speed(s). For example, to make them silent:
```sh
fancy set -f 0
```

## FAQ

### Why? NBFC can also do it.
NBFC is a great software (also one of the cleanest codebase I ever seen). However, it's written in C#, which means that it depends on `mono` runtime on Linux. `mono` is a pretty huge dependency, especiallly when NBFC is the only thing which needs it, and uses a lot of RAM. That's the reason why I started to write `fancy` (and also because I wanted to test my Rust (<sub>kind of</sub>) "skills"). 

### Linux ONLY?
Well, [NBFC](https://github.com/hirschmann/nbfc) is already well integrated with the Windows "ecosystem", since C# is more common on Windows. It works very well, so go check it. If you want to provide Windows support, you are also welcome.

### Did you really steal code to NBFC?
For real, the only things that I pasted as-is are the "algorithm" to convert the fan speed to a percentage (and vice-versa) and the structures. I wrote the other parts of the code myself, inspired by the NBFC's code.

## License
The project is licensed under MPL-2.0. You implicitly accept it when you send a pull request.

*WARNING:* The configurations are from NBFC, which is under GPLv3 license.

## Contributing
Please see [CONTRIBUTING.md](https://github.com/MusiKid/fancy/blob/master/CONTRIBUTING.md).

## Credits
Thanks to [@hirschmann](https://github.com/hirschmann/) for creating [NBFC](https://github.com/hirschmann/nbfc), where I shamelessly stolen some pieces of code (open source ¯\\_(ツ)_/¯), and all the contributors who created the configurations.
