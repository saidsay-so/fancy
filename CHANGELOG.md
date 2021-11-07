# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.1](https://github.com/musikid/fancy/compare/v0.7.0..0.7.1) - 2021-11-07

### CI

- Fix fedora job

- Add codecov config to ignore cli folder


### Features

- Add status subcommand


## [0.7.0](https://github.com/musikid/fancy/compare/v0.6.1..v0.7.0) - 2021-11-07

### Bug Fixes

- Keep old configuration for dbus

- Early return when error occurs

- Set NotebookModel field as mandatory


### CI

- Fix release job

- Remove unused variable

- Revert "remove unused variable"


### Documentation

- Fix typo

- Add note about formats

- Improve docs


### Features

- Add support for other formats

> Would resolve #14

- Deserialize fields as PascalCase


### Miscellaneous Tasks

- Improve release script


### Refactor

- Directly use dbus error type

- Replace borrow_mut by replace

- Improve changed props callback


### Testing

- Add tests for config loader

- Add more tests

- Add tests for json

- Add test for invalid name


## [0.6.1](https://github.com/musikid/fancy/compare/v0.6.0..v0.6.1) - 2021-11-06

### Bug Fixes

- Remove bug when setting target speed

> The `target_fans_speeds`` Vec did not have a length.

- Fix load config out of allowed path

> The service potentially allowed a vulnerability when the given
configuration name was a relative path.

- Test config when setting first time

> The fan configuration is tested directly when trying to set it instead
of panicking when it is invalid


### CI

- Fix release job


### Miscellaneous Tasks

- Update changelog

- Exit when error occured in release script

- Improve release script


### Refactor

- Improve logging


### Build

- Set release number to 1


## [0.6.0](https://github.com/musikid/fancy/compare/v0.5.0..v0.6.0) - 2021-11-06

### Bug Fixes

- Move to fork of psutil

- Make unused fields optional

> Related to #12

- Fix crash when given invalid config

> The service now also returns what caused the error.

Related to #15

- Log after the value is read

- Check config only when flag is set

> Related to #18

- Remove problematic `unwrap()`


### CI

- Add coverage

- Add book worflow

- Replace cargo-deny action

- Fix test workflow

- Fix book workflow

- Fix toolchain version

- Add release body


### Documentation

- Add coverage to README

- Add book

- Improve documentation

- [skip ci] remove Codacy badge

- Add release badge

- Add a note for the book

- Improve README

- Add page for supported models

- Add manual flag to man file

- Add section for debugging

- Add manual flag

- Correct typo in debug section


### Features

- Add completions to build script

- Add flag for manual speed management

> Related to #18


### Miscellaneous Tasks

- [skip ci] switch to generated changelog

- Update lockfile

- Update lockfile

- Update release script


### Refactor

- Move to edition 2021

- Apply clippy suggestions

- Improve logging

- Reorder instructions


### Styling

- Use imports


### Build

- Add flags to improve performance

- Update release script

- Fix Makefile

- Fix path in Makefile

- Add completions to Makefile

- Add completion files to spec file

- Remove ununsed argument in Makefile

- Change checksum


## [0.5.0](https://github.com/musikid/fancy/compare/v0.4.0..v0.5.0) - 2021-10-02

### Bug Fixes

- Send change signal for target speeds

- Fix borrow error

- Check len from fans_speeds instead

- Fix setting multiple fans speeds

- Update lockfile with releases


### Documentation

- Add instructions for AUR package

- Improve README


### Features

- Add poll interval to dbus interface

- Get fans names

- Return only speeds for FansSpeeds

- Use anyhow for error reporting


### Miscellaneous Tasks

- Use Duration::ZERO

- Fix imports

- Ignore gz file

- Add git-cliff changelog generator

- Add release script


### Refactor

- Split nbfc serialize to its own module


### Build

- Fix systemd macro require

- Fix spec changelog

- Add PHONY targets

- Generate changelog with rpkg


## [0.4.0](https://github.com/musikid/fancy/compare/v0.3.1..v0.4.0) - 2021-05-09

### Bug Fixes

- Treat errors when loading config

- Check first if temperature is high

- Restore target speeds after change

> Restore the target fans speeds after the configuration is changed.


### CI

- Remove Rust action

> rustc and cargo should be in the Ubuntu repository (hopefully).


### Documentation

- Improve docs


### Features

- Add other temp computation methods

> Add support for choosing the temperature computation method, between CPU
sensor only and average of all sensors.


### Miscellaneous Tasks

- Update lockfile


### Refactor

- Improve docs and code structure

- Use `From` trait for RawPort


### Testing

- Add test for config refresh

- Improve `all_configs` test


### Build

- Remove systemd from deps

- Lower required rust version

- Update checksums

- Use `write_all` in build script

- Force gzip command in Makefiles

- Fix copyrights

- Release v0.4.0


## [0.3.1](https://github.com/musikid/fancy/compare/v0.3.0..v0.3.1) - 2021-04-21

### Bug Fixes

- Correct path in systemd file

- Set auto to false with target fan speed

> Fixes the issue when a target fan speed was set (implying manual control)
and `auto` could still be set to `true`.


### Documentation

- Add COPR badge and fix link in changelog

- Improve README

- Improve docs

- Improve README


### Miscellaneous Tasks

- Update lockfile

- Bump dependencies


### Refactor

- Fix imports


### Testing

- Re-enable `all_configs` test


### Build

- Fix packaging errors

> According to lintian, Debian seems to prefer service files in
/lib.
It also add an override for the fancy-sleep.service, which has suspend
as target.

- Fix issue with config files

> RPM requires to add `%config` macro for configuration files.

- Add Rust to build dependencies

- Add `--locked` flag to Makefiles

> The flag `--locked` stops Cargo from updating dependencies

- Add support for Arch packaging

- Add test target to main Makefile

- Change build dep from cargo to rust


## [0.3.0](https://github.com/musikid/fancy/compare/..v0.3.0) - 2021-04-14

### Bug Fixes

- Check the number of speeds provided to `set_target_fans_speeds`

- Fix test


### CI

- Update workflows

- Install rust toolchain for all jobs

- Remove unique build job

> The build job was used to unify the build step.
However, it doesn't work well with Debian packaging and does not improve
so much the build time.

- Create $DESTDIR for `make-archive` job

> Fix the issue that made the job fail because $DESTDIR was not created.

- Add support for building/uploading RPM package

> Introduces a new job for building the RPM package and uploading it both
to GitHub and to Fedora COPR repository.

- Fix errors

- Fix errors in workflow

- Fix the same mistake again

- Fix RPM build

- Restore RPM package building for GitHub

> It restores the `rpkg local` step because `rpkg build` only build source
RPM to send it to COPR.


### Documentation

- Merge all changelogs into one

- Improve docs

- Improve


### Features

- Add function to set speed by index


### Miscellaneous Tasks

- Remove gui folder

- Remove generated code from git


### Refactor

- More information to log


### Build

- Add Debian packaging for the project

- Move man pages build to subfolders Makefile

- Fix errors in Makefiles

- Remove formatting of generated interface code

- Add support for RPM packaging

- Fix `mandir` variable in Makefile

- Remove changelog in RPM .spec

- Fix variable issues

> The `prefix` variable wasn't expanded correctly, which led to wrong
paths being written.

- Add macro to get version for rpkg


<!-- generated by git-cliff -->
