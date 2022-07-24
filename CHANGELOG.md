# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased][unreleased]

### Added ✨

-   Added some more trait impls to `NomBytes`:
    -   `From<Bytes>`
    -   `Compare<&'_ [u8]>`
    -   `FindSubstring<NomBytes>`
    -   `FindSubstring<&'_ [u8]>`
    -   `FindSubstring<&'_ str>`
-   Added another unit test to `RangeType<T>`

### Changed 🔧

-   Made `RangeType<T>` more generic and it can now slice `&str` in addition to `&[T]`.

### Fixed 🐛

-   Lint CHANGELOG

## [0.1.1][] - 2022-07-24

### Added ✨

-   Added `serde` support

### Changed 🔧

-   Updated README description
-   Removed unnecessary cloning where possible

### Fixed 🐛

-   Fixed minor typo in README

## [0.1.0][] - 2022-07-23

### Added ✨

-   Project setup
-   First release

[unreleased]: https://github.com/alexschrod/nombytes/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/alexschrod/nombytes/compare/v0.1...v0.1.1
[0.1.0]: https://github.com/alexschrod/nombytes/releases/tag/v0.1
