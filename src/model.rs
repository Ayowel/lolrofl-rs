/*!
Structs that map to parts of the data model of a ROFL file.
*/

mod binheader;
mod payload;
mod segment;
pub mod section;
pub use binheader::*;
pub use payload::*;
pub use segment::*;
