# Changelog

## [Unreleased]

## [v0.1.4] - 2026-01-29

### Changed

- Upgrade `thiserror` dependency to `v2.0.18` ([#7](https://github.com/kylewlacy/tick-encoding/pull/7) by [@jaudiger](https://github.com/jaudiger))
- Add `#[inline]` annotations for functions on hot paths ([#9](https://github.com/kylewlacy/tick-encoding/pull/9) by [@jaudiger](https://github.com/jaudiger))
- Optimize encoding / decoding with lookup table ([#10](https://github.com/kylewlacy/tick-encoding/pull/10) by [@jaudiger](https://github.com/jaudiger))
- Optimize encoder state machine ([#14](https://github.com/kylewlacy/tick-encoding/pull/14) by [@jaudiger](https://github.com/jaudiger))
- Fix typo in docs ([#17](https://github.com/kylewlacy/tick-encoding/pull/17) by [@jaudiger](https://github.com/jaudiger))
- Tweak memory optimization in `decode` function ([#19](https://github.com/kylewlacy/tick-encoding/pull/19))

## [v0.1.3] - 2025-01-22

### Changed

- Upgrade `thiserror` dependency to `v2.0.11` ([#1](https://github.com/kylewlacy/tick-encoding/pull/1) by [@jaudiger](https://github.com/jaudiger))
- Make `requires_escape` into a `const fn` ([#2](https://github.com/kylewlacy/tick-encoding/pull/2) by [@jaudiger](https://github.com/jaudiger))

## [v0.1.2] - 2024-01-28

### Changed

- Implement [`std::error::Error`](https://doc.rust-lang.org/stable/std/error/trait.Error.html) for `tick_encoding::DecodeError`.
    - This was always intended to be in-place, but was unavailable in previous versions due to a mistake in a `#[cfg_attr(...)]` attribute.

## [v0.1.1] - 2024-01-28

### Fixed

- Fixed a bug where the sequence "\`60" would be accepted and decode to "\`". The canonical encoding is "\`\`", so this now returns an error.

## [v0.1.0] - 2024-01-28

### Added

- Initial release!

[Unreleased]: https://github.com/kylewlacy/tick-encoding/compare/v0.1.4...HEAD
[v0.1.4]: https://github.com/kylewlacy/tick-encoding/releases/tag/v0.1.4
[v0.1.3]: https://github.com/kylewlacy/tick-encoding/releases/tag/v0.1.3
[v0.1.2]: https://github.com/kylewlacy/tick-encoding/releases/tag/v0.1.2
[v0.1.1]: https://github.com/kylewlacy/tick-encoding/releases/tag/v0.1.1
[v0.1.0]: https://github.com/kylewlacy/tick-encoding/releases/tag/v0.1.0
