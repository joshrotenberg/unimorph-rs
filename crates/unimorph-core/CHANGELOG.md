# Changelog

All notable changes to this project will be documented in this file.

## [0.2.1] - 2026-01-19

### Bug Fixes

- Correct README CLI examples and handle trailing tabs in TSV ([#88](https://github.com/joshrotenberg/unimorph-rs/pull/88))

### Features

- Add detailed import reporting with compression/LFS metadata ([#92](https://github.com/joshrotenberg/unimorph-rs/pull/92))



## [0.2.0] - 2026-01-19

### Documentation

- Document transparent compression support ([#85](https://github.com/joshrotenberg/unimorph-rs/pull/85))

### Features

- Add transparent decompression for compressed UniMorph files ([#83](https://github.com/joshrotenberg/unimorph-rs/pull/83))
- Add Git LFS support for large files ([#86](https://github.com/joshrotenberg/unimorph-rs/pull/86))



## [0.1.8] - 2026-01-07



## [0.1.7] - 2026-01-07



## [0.1.6] - 2026-01-07



## [0.1.5] - 2026-01-07



## [0.1.4] - 2026-01-06

### Documentation

- Update references from unimorph-cli to unimorph



## [0.1.3] - 2026-01-06

### Bug Fixes

- Update examples from Italian to Spanish ([#67](https://github.com/joshrotenberg/unimorph-rs/pull/67))

### Documentation

- Add Python bindings section to README ([#66](https://github.com/joshrotenberg/unimorph-rs/pull/66))



## [0.1.2] - 2026-01-06

### Bug Fixes

- Add readme field to crate manifests for crates.io display ([#55](https://github.com/joshrotenberg/unimorph-rs/pull/55))

### Documentation

- Streamline README and fix inaccuracies ([#56](https://github.com/joshrotenberg/unimorph-rs/pull/56))

### Features

- Add sample command for random entry sampling ([#60](https://github.com/joshrotenberg/unimorph-rs/pull/60))
- Add Docker support ([#62](https://github.com/joshrotenberg/unimorph-rs/pull/62))



## [0.1.1] - 2026-01-06

### Miscellaneous Tasks

- Release v0.1.0 ([#51](https://github.com/joshrotenberg/unimorph-rs/pull/51))



## [0.1.0] - 2026-01-06

### Bug Fixes

- Export -o - now correctly writes to stdout ([#47](https://github.com/joshrotenberg/unimorph-rs/pull/47))

### Features

- Initial implementation of unimorph-rs toolkit
- Add tracing support and improve error messages
- Add export functionality (parquet, tsv, jsonl)
- Add tracing instrumentation to export module
- Add fluent query builder API
- Add tracing instrumentation to query builder
- Additional CLI UX improvements
- Add commit SHA tracking for update detection
- Add download progress with byte counts
- Add importing phase indicator and features command
- Add --contains filter to search command ([#43](https://github.com/joshrotenberg/unimorph-rs/pull/43))

### Testing

- Add comprehensive testing strategy


