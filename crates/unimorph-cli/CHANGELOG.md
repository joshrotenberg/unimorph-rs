# Changelog

All notable changes to this project will be documented in this file.

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


