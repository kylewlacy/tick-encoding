# Changelog

## [Unreleased]

## [v0.1.1] - 2024-01-28

### Fixed

- Fixed a bug where the sequence "\`60" would be accepted and decode to "\`". The canonical encoding is "\`\`", so this now returns an error.

## [v0.1.0] - 2024-01-28

### Added

- Initial release!

[Unreleased]: https://github.com/kylewlacy/tick-encoding/compare/v0.1.1...HEAD
[v0.1.1]: https://github.com/kylewlacy/tick-encoding/releases/tag/v0.1.1
[v0.1.0]: https://github.com/kylewlacy/tick-encoding/releases/tag/v0.1.0
