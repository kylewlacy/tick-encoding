# Changelog

## [Unreleased]

### Changed

- Implement [`std::error::Error`](https://doc.rust-lang.org/stable/std/error/trait.Error.html) for `tick_encoding::DecodeError`.
    - This was always intended to be in-place, but was unavailable in previous versions due to a mistake in a `#[cfg_attr(...)]` attribute.

## [v0.1.1] - 2024-01-28

### Fixed

- Fixed a bug where the sequence "\`60" would be accepted and decode to "\`". The canonical encoding is "\`\`", so this now returns an error.

## [v0.1.0] - 2024-01-28

### Added

- Initial release!

[Unreleased]: https://github.com/kylewlacy/tick-encoding/compare/v0.1.1...HEAD
[v0.1.1]: https://github.com/kylewlacy/tick-encoding/releases/tag/v0.1.1
[v0.1.0]: https://github.com/kylewlacy/tick-encoding/releases/tag/v0.1.0
