# Changelog

All notable changes to this project will be documented in this file.

## [0.1.6] - 2026-01-07

### Bug Fixes

- Add repository URL to --version output ([#72](https://github.com/joshrotenberg/unimorph-rs/pull/72))



## [0.1.5] - 2026-01-07

### Miscellaneous Tasks

- Switch to cargo-dist for release binaries



## [0.1.4] - 2026-01-06

### Bug Fixes

- Format timestamps in info command and update help text
- Export -o - now correctly writes to stdout ([#47](https://github.com/joshrotenberg/unimorph-rs/pull/47))
- Add version to unimorph-core dependency for crates.io publishing ([#53](https://github.com/joshrotenberg/unimorph-rs/pull/53))
- Add readme field to crate manifests for crates.io display ([#55](https://github.com/joshrotenberg/unimorph-rs/pull/55))
- Update examples from Italian to Spanish ([#67](https://github.com/joshrotenberg/unimorph-rs/pull/67))
- Conditionally import eyre for parquet feature

### Documentation

- Add shell completions instructions to README ([#42](https://github.com/joshrotenberg/unimorph-rs/pull/42))
- Add mdBook documentation site ([#48](https://github.com/joshrotenberg/unimorph-rs/pull/48))
- Streamline README and fix inaccuracies ([#56](https://github.com/joshrotenberg/unimorph-rs/pull/56))
- Add Python bindings section to README ([#66](https://github.com/joshrotenberg/unimorph-rs/pull/66))
- Update references from unimorph-cli to unimorph

### Features

- Initial implementation of unimorph-rs toolkit
- Add tracing support and improve error messages
- Add CLI enhancements
- Enhance list command with GitHub API ([#19](https://github.com/joshrotenberg/unimorph-rs/pull/19))
- Add info and update commands (#22, #23)
- Additional CLI UX improvements
- Add commit SHA tracking for update detection
- Add download progress with byte counts
- Add importing phase indicator and features command
- Add short command aliases ([#37](https://github.com/joshrotenberg/unimorph-rs/pull/37))
- Add --json flag to download, delete, and repair commands ([#39](https://github.com/joshrotenberg/unimorph-rs/pull/39))
- Add color output for better readability ([#40](https://github.com/joshrotenberg/unimorph-rs/pull/40))
- Add --contains filter to search command ([#43](https://github.com/joshrotenberg/unimorph-rs/pull/43))
- Add configuration file support ([#44](https://github.com/joshrotenberg/unimorph-rs/pull/44))
- Add default language configuration ([#45](https://github.com/joshrotenberg/unimorph-rs/pull/45))
- Add pipe-friendly output modes ([#46](https://github.com/joshrotenberg/unimorph-rs/pull/46))
- Add sample command for random entry sampling ([#60](https://github.com/joshrotenberg/unimorph-rs/pull/60))
- Add Docker support ([#62](https://github.com/joshrotenberg/unimorph-rs/pull/62))
- Rename unimorph-cli to unimorph for simpler cargo install

### Miscellaneous Tasks

- Release v0.1.0 ([#51](https://github.com/joshrotenberg/unimorph-rs/pull/51))
- Release v0.1.1 ([#52](https://github.com/joshrotenberg/unimorph-rs/pull/52))
- Release v0.1.2 ([#59](https://github.com/joshrotenberg/unimorph-rs/pull/59))
- Release v0.1.3 ([#68](https://github.com/joshrotenberg/unimorph-rs/pull/68))

### Refactor

- Modularize CLI into separate command files
- Improve CLI UX with positional arguments

### Testing

- Add comprehensive testing strategy



## [0.1.3] - 2026-01-06

### Bug Fixes

- Update examples from Italian to Spanish ([#67](https://github.com/joshrotenberg/unimorph-rs/pull/67))
- Conditionally import eyre for parquet feature

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



## [0.1.0] - 2026-01-06

### Bug Fixes

- Format timestamps in info command and update help text
- Export -o - now correctly writes to stdout ([#47](https://github.com/joshrotenberg/unimorph-rs/pull/47))

### Features

- Initial implementation of unimorph-rs toolkit
- Add tracing support and improve error messages
- Add CLI enhancements
- Enhance list command with GitHub API ([#19](https://github.com/joshrotenberg/unimorph-rs/pull/19))
- Add info and update commands (#22, #23)
- Additional CLI UX improvements
- Add commit SHA tracking for update detection
- Add download progress with byte counts
- Add importing phase indicator and features command
- Add short command aliases ([#37](https://github.com/joshrotenberg/unimorph-rs/pull/37))
- Add --json flag to download, delete, and repair commands ([#39](https://github.com/joshrotenberg/unimorph-rs/pull/39))
- Add color output for better readability ([#40](https://github.com/joshrotenberg/unimorph-rs/pull/40))
- Add --contains filter to search command ([#43](https://github.com/joshrotenberg/unimorph-rs/pull/43))
- Add configuration file support ([#44](https://github.com/joshrotenberg/unimorph-rs/pull/44))
- Add default language configuration ([#45](https://github.com/joshrotenberg/unimorph-rs/pull/45))
- Add pipe-friendly output modes ([#46](https://github.com/joshrotenberg/unimorph-rs/pull/46))

### Refactor

- Modularize CLI into separate command files
- Improve CLI UX with positional arguments

### Testing

- Add comprehensive testing strategy


