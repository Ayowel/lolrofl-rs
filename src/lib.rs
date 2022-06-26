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
// let content = std::fs::read(source_file).unwrap();
# let content: Vec<u8> = vec![ 82, 73, 79, 84, 0, 0, // magic
# 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255, // signature
# 0x20, 1, // head size
# 0xcd, 1, 0, 0, // File size
# 0x20, 1, 0, 0, // Metadata offset
# 0x6b, 0, 0, 0, // Metadata length
# 0x8b, 1, 0, 0, // Payload header offset
# 0x42, 0, 0, 0, // Payload header length
# 0xcd, 1, 0, 0, // Payload offset
# // Metadata string
# 0x7b, 0x22, 0x67, 0x61, 0x6d, 0x65, 0x4c, 0x65, 0x6e, 0x67, 0x74, 0x68, 0x22, 0x3a, 0x39, 0x31,
# 0x37, 0x32, 0x32, 0x2c, 0x22, 0x67, 0x61, 0x6d, 0x65, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e,
# 0x22, 0x3a, 0x22, 0x31, 0x32, 0x2e, 0x31, 0x30, 0x2e, 0x34, 0x34, 0x34, 0x2e, 0x32, 0x30, 0x36,
# 0x38, 0x22, 0x2c, 0x22, 0x6c, 0x61, 0x73, 0x74, 0x47, 0x61, 0x6d, 0x65, 0x43, 0x68, 0x75, 0x6e,
# 0x6b, 0x49, 0x64, 0x22, 0x3a, 0x36, 0x2c, 0x22, 0x6c, 0x61, 0x73, 0x74, 0x4b, 0x65, 0x79, 0x46,
# 0x72, 0x61, 0x6d, 0x65, 0x49, 0x64, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x73, 0x74, 0x61, 0x74, 0x73,
# 0x4a, 0x73, 0x6f, 0x6e, 0x22, 0x3a, 0x22, 0x5b, 0x5d, 0x22, 0x7d,
# // Payload header
# 0xca, 0x63, 0xb6, 0x5f, 0x01, 0, 0, 0, // Match ID
# 0x4a, 0x66, 0x1 , 0x0, // Match duration
# 0x2, 0x0, 0x0, 0x0, // Number of keyframes
# 0x6, 0x0, 0x0, 0x0, // Number of chunks
# 0x1, 0x0, 0x0, 0x0, // Last data chunk
# 0x2, 0x0, 0x0, 0x0, // First game chunk
# 0x0, 0x87, 0x93, 0x3, // Keyframe interval
# 0x20, 0x0, // Encryption Key Length
# // Encryption key
# 0x30, 0x4d, 0x35, 0x44, 0x67, 0x41, 0x32, 0x50, 0x73, 0x58, 0x4a, 0x59, 0x55, 0x36, 0x69, 0x30, 0x2f, 0x49, 0x58,
# 0x4f, 0x33, 0x35, 0x59, 0x6a, 0x53, 0x50, 0x66, 0x79, 0x4f, 0x63, 0x6a, 0x38,
# // Arbitrary bytes, more would follow in an actual file
# 0x1,
# ];
// Parse the file's content with Rofl
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
    /// Get an iterator over the payload's segments
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
