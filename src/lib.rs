/*!
Rust library and tool to parse and inspect ROFL replay files generated from League of Legends games.

Backward-compatibility for replay files is NOT to be expected as of now.

# Usage as a command-line tool

After building with `cargo install --bin lolrofl --features "clap json payload"`, a new `lolrofl` executable become available.

Said executable allows the inspection of ROFL files to extract game information, metadata, or development intel with the following commands:

* `get`: Get high-level information on the file
  * `get info`: Print simple/high-level info on the file and the game
  * `get metadata`: Print the game's metadata
  * `get payload`: Print technical information on the file
* `analyze`: Get low-level information on the file - usually for debug and development purpose
* `export`: Export chunk or keyframe data to a file or directory

# Usage as a library

Use `lolrofl` to parse a loaded file's content :

```rust
// Parse the file's content with Rofl
# let content = lolrofl::test::sample_base_file_0();
// let content = std::fs::read(source_file).unwrap();
let data = lolrofl::Rofl::from_slice(&content[..]).unwrap();

// Print the file's length as specified within the file (this may not match the actual file size if it is incomplete or corrupted)
println!("Expected file length: {:?} bytes", data.head().file_len());
// Print the file's metadata (a JSON string)
println!("{}", data.metadata().unwrap());
// Print information on the game without depending on the metadata
let payload = data.payload().unwrap();
println!("The game {} lasted {} seconds", payload.id(), payload.duration()/1000);
# assert_eq!(payload.duration(), 91722);
```
*/

mod error;
pub use error::*;
pub mod iter;
pub mod model;
// FIXME: the test feature is only required because doctest context is not passed by cargo at compile-time
#[cfg(any(doctest, test, feature = "test"))]
pub mod test;
use model::*;

/// Base ROFL file parser
/// 
/// # Usage
/// 
/// ```rust
/// // Parse a file's content
/// # let content = lolrofl::test::sample_base_file_0();
/// // let content = std::fs::read("game.rofl").unwrap();
/// let game = lolrofl::Rofl::from_slice(&content[..]).unwrap();
///
/// let header = game.head(); // File header
/// let data = game.metadata(); // Game metadata JSON string
/// # assert_eq!(data.is_ok(), true);
/// let payload = game.payload(); // Game payload
/// # assert_eq!(payload.is_ok(), true);
/// ```
pub struct Rofl<'a> {
    /// ROFL file's Start Header
    head: BinHeader,
    /// ROFL File's data
    data: &'a[u8],
}

impl Rofl<'_> {
    /// Starting bytes of a ROFL file
    /// 
    /// This is public for ease of file recognition but should generally NOT be relied upon
    pub const MAGIC: [u8; 4] = [82,73,79,84]; // TODO: check if 6 bytes instead of 0
    /// Get the ROFL header
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # let content = lolrofl::test::sample_base_file_0();
    /// // let content = std::fs::read("game.rofl").unwrap();
    /// let game = lolrofl::Rofl::from_slice(&content[..]).unwrap();
    /// println!("Expected file length: {} bytes", game.head().file_len());
    /// # assert_eq!(game.head().file_len(), 0x01cd);
    /// ```
    pub fn head(&self) -> &BinHeader { &self.head }
    /// Get the loaded JSON Metadata string
    /// 
    /// # Warning
    /// 
    /// The returned string is not guaranteed to be valid if the file is malformed
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # let content = lolrofl::test::sample_base_file_0();
    /// // let content = std::fs::read("game.rofl").unwrap();
    /// let game = lolrofl::Rofl::from_slice(&content[..]).unwrap();
    /// let meta = json::parse(game.metadata().unwrap()).unwrap();
    /// println!("Duration: {} ms", meta["gameLength"]);
    /// println!("Patch: {}", meta["gameVersion"]);
    /// # assert_eq!(meta["gameVersion"], "12.10.444.2068");
    /// ```
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
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # let content = lolrofl::test::sample_base_file_0();
    /// // let content = std::fs::read("game.rofl").unwrap();
    /// let game = lolrofl::Rofl::from_slice(&content[..]).unwrap();
    ///
    /// let payload = game.payload().unwrap();
    /// println!("Game ID: {}", payload.id());
    /// println!("Duration: {} ms", payload.duration());
    /// # assert_eq!(payload.duration(), 0x01664a);
    /// # assert_eq!(payload.chunk_count(), 6);
    /// ```
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
    /// Get an iterator over the payload's segments
    /// 
    /// `with_data` is implicitly `false` if the lib was compiled without the `payload` feature
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// # let content = lolrofl::test::sample_base_file_0();
    /// // let content = std::fs::read("game.rofl").unwrap();
    /// let game = lolrofl::Rofl::from_slice(&content[..]).unwrap();
    ///
    /// for segment in game.segment_iter(false).unwrap() {
    ///     println("{} {}", if segment.is_chunk() { "Chunk" } else { "Keyframe" }, segment.id())
    /// }
    /// ```
    /// 
    /// ```rust
    /// // Truncated file
    /// # let content = lolrofl::test::sample_base_file_0();
    /// // let content = std::fs::read("game.rofl").unwrap();
    /// let game = lolrofl::Rofl::from_slice(&content[..]).unwrap();
    ///
    /// let mut data = game.segment_iter(false);
    /// assert_eq!(data.is_err(), true)
    /// ```
    pub fn segment_iter<'a>(&'a self, with_data: bool) -> Result<crate::iter::PayloadIterator<'a>, error::Errors> {
        // FIXME: the doctest should be runnable
        if self.data.len() < self.head.file_len() {
            Err(error::Errors::BufferTooSmall)
        } else {
            self.payload().and_then(|p|
                crate::iter::PayloadIterator::new(
                    &self.data[self.head.payload_offset()..self.head.file_len()],
                    &p,
                    with_data,
                )
            )
        }
    }
    /// Create a new Rofl instance from a ROFL file's slice
    /// 
    /// # Panics
    /// 
    /// If the buffer contains less than 288 bytes - in the future, this will be an error
    /// 
    /// # Errors
    /// 
    /// If the slice does not start with [`MAGIC`]
    /// 
    /// [`MAGIC`]: Rofl::MAGIC
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # let content = lolrofl::test::sample_base_file_0();
    /// // let content = std::fs::read("game.rofl").unwrap();
    /// let game = lolrofl::Rofl::from_slice(&content[..]).unwrap();
    /// ```
    pub fn from_slice<'a>(slice: &'a[u8]) -> Result<Rofl<'a>, Errors> {
        if slice.len() < Rofl::MAGIC.len() || Rofl::MAGIC != slice[..Rofl::MAGIC.len()] {
            return Err(Errors::InvalidBuffer);
        }
        // FIXME: return Result<> in BinHeader initializers and control slice size
        let header = BinHeader::from_raw_source(slice);

        Ok(Rofl {
            head: header,
            data: slice,
        })
    }
}
