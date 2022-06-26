# LoLROFL

[![Crates.io](https://img.shields.io/crates/v/lolrofl.svg)](https://crates.io/crates/lolrofl)
[![Coverage status](https://img.shields.io/codecov/c/github/Ayowel/lolrofl-rs)](https://codecov.io/github/Ayowel/lolrofl-rs/)
[![MIT License](https://img.shields.io/badge/license-APACHE%202.0-blue.svg)](https://mit-license.org/)
[![Documentation](https://docs.rs/lolrofl/badge.svg)](https://docs.rs/lolrofl)

Rust library and tool to parse and inspect ROFL replay files generated from League of Legends games.

The end goal is to provide the ability to extract all data contained in a replay file in a meaningfull way based on the definitions available at https://github.com/loldevs/leaguespec/wiki after updating them to match current-day replay formats.

Backward-compatibility for replay files is NOT to be expected as of now.

__WARNING:__ This project uses semantic-versioning with v0 exception. As such, the API may be changed with every minor update while the major version is 0. Only patch versions should be presumed as forward-compatible.

## Command-line tool usage

Install lolrofl from the command-line, to be able to use any of its many subcommands to inspect replay files.

```bash
cargo install lolrofl --features "clap,json"
```

* `lolrofl get`: Get high-level information on the file
  * `lolrofl get info`: Print simple/high-level info on the file and the game
  * `lolrofl get metadata`: Print the game's metadata
  * `lolrofl get payload`: Print technical information on the file
* `lolrofl analyze`: Get low-level information on the file - usually for debug and development purpose
* `lolrofl export`: Export chunk or keyframe data to a file or directory

## Library usage

Add `lolrofl` to your project's `cargo.toml`.

```toml
[dependencies.lolrofl]
version = "^0.2.0"
```

Use `lolrofl` to parse a loaded file's content :

```rust
// Load a file in memory to build a ROFL parser
let content = std::fs::read(source_file).unwrap();
let data = lolrofl::Rofl::from_slice(&content[..]).unwrap();

// Print the expected file length (this may not match the actual file size if it is incomplete or corrupted)
println!("Expected file length: {:?} bytes", data.head().file_len());
// Print the file's metadata (a JSON string)
println!("{}", data.metadata()?);
// Print information on the game without depending on the metadata
println!("Game ID: {}", data.payload()?.id());
```
