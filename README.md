# LoLROFL

[![Crates.io](https://img.shields.io/crates/v/lolrofl.svg?maxAge=2592000)](https://crates.io/crates/lolrofl)
[![Documentation](https://docs.rs/lolrofl/badge.svg)](https://docs.rs/lolrofl)

Rust library and tool to parse and inspect ROFL replay files generated from League of Legends games.

The end goal is to provide the ability to extract all data contained in a replay file in a meaningfull way based on the definitions available at https://github.com/loldevs/leaguespec/wiki after updating them to match current-day replay formats.

Backward-compatibility for replay files is NOT to be expected as of now.

:warning: This project uses semantic-versioning with v0 exception. As such, the API may be changed with every minor update while the major version is 0. Only patch versions should be presumed as forward-compatible.

## Usage

Add lolrofl to your project's `cargo.toml`.

```toml
[dependencies.lolrofl]
version = "^0.1.0"
```
