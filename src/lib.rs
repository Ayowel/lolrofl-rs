/*!
Rust library and tool to parse and inspect ROFL replay files generated from League of Legends games.

Backward-compatibility for replay files is NOT to be expected as of now.

## Usage as a command-line tool

After building with `cargo install --bin lolrofl --features "clap json payload"`, a new `lolrofl` executable become available.

Said executable allows the inspection of ROFL files to extract game information, metadata, or development intel with the following commands:

* `get`: Get high-level information on the file
  * `get info`: Print simple/high-level info on the file and the game
  * `get metadata`: Print the game's metadata
  * `get payload`: Print technical information on the file
* `analyze`: Get low-level information on the file - usually for debug and development purpose
* `export`: Export chunk or keyframe data to a file or directory

## Usage as a library

Use `lolrofl` to parse a loaded file's content :

```rust
// Load a file in memory
let content = std::fs::read(source_file).unwrap();

// Load the data with the Rofl object
let data = lolrofl::Rofl::from_slice(&content[..]).unwrap();

// Print the file's signature
println!("{:?}", data.head().signature());
// Print the file's metadata
println!("{}", data.metadata().unwrap());
// Print information on the game without depending on the metadata
let payload = data.payload.unwrap();
println!("The game {} lasted {} seconds", payload.id(), payload.duration()/1000);
```
*/

mod error;
pub use error::*;
pub mod iter;
pub mod model;
use model::*;

/// Base ROFL file parser
pub struct Rofl<'a> {
    /// ROFL file's Start Header
    head: BinHeader,
    /// ROFL File's data
    data: &'a[u8],
}

impl Rofl<'_> {
    /// Starting bytes of a ROFL file
    pub const MAGIC: [u8; 4] = [82,73,79,84]; // TODO: check if 6 bytes instead of 0
    /// Get the ROFL header
    pub fn head(&self) -> &BinHeader { &self.head }
    /// Get the loaded JSON Metadata string
    pub fn metadata(&self) -> Result<&str, Errors> {
        if self.data.len() < self.head.metadata_offset() + self.head.metadata_len() {
            return Err(error::Errors::BufferTooSmall);
        }
        std::str::from_utf8(
                &self.data[self.head.metadata_offset()..self.head.metadata_offset() + self.head.metadata_len()]
        )
        .or_else(|_| Err(error::Errors::InvalidBuffer))
    }
    /// Get the loaded payload header
    pub fn payload(&self) -> Result<PayloadHeader, Errors> {
        if self.data.len() < self.head.payload_header_offset() + self.head.payload_header_len() {
            Err(Errors::BufferTooSmall)
        } else {
            let payload = PayloadHeader::from_raw_section(
                &self.data[self.head.payload_header_offset()..self.head.payload_header_offset() + self.head.payload_header_len()]
            );
            Ok(payload)
        }
    }
    /// TODO
    pub fn segment_iter<'a>(&'a self) -> Result<crate::iter::PayloadIterator<'a>, error::Errors> {
        if self.data.len() < self.head.file_len() {
            Err(error::Errors::BufferTooSmall)
        } else {
            self.payload().and_then(|p|
                crate::iter::PayloadIterator::new(
                    &self.data[self.head.payload_offset()..self.head.file_len()],
                    &p,
                )
            )
        }
    }
    /// Create a new Rofl instance from a ROFL file's slice
    pub fn from_slice<'a>(slice: &'a[u8]) -> Result<Rofl<'a>, Errors> {
        if slice.len() < Rofl::MAGIC.len() || Rofl::MAGIC != slice[..Rofl::MAGIC.len()] {
            return Err(Errors::InvalidBuffer);
        }
        // FIXME: return Option<> in BinHeader initializers and control slice size
        let header = BinHeader::from_raw_source(slice);

        Ok(Rofl {
            head: header,
            data: slice,
        })
    }
}
