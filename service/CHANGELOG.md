# Service Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- Refactoring code:
  - `Arc` have been replaced by `Rc`, since the code is single-threaded now.
  - `Mutex` and `RwLock` have been replaced by `RefCell` where appropriated.
  - Remove `Unpin` trait.

## [v0.2.0] - 2020-09-25

### Added

- Support of other sensors than the CPU one for temperature computation.

## [v0.1.0]

Initial version.

[v0.2.0]: https://github.com/musikid/fancy/compare/fancy-service-0.1.0...fancy-service-0.2.0
[v0.1.0]: https://github.com/musikid/fancy/compare/fancy-service-0.1.0
