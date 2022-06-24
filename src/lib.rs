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

// Build a selector for data loading
let data_selector = lolrofl::consts::LOAD_HEAD | lolrofl::consts::LOAD_METADATA;

// Load the data with the Rofl object
let data = lolrofl::Rofl::from_slice(&content[..], data_selector).unwrap();

// Print the file's signature
println!("{:?}", data.head().signature());
// Print the file's metadata
println!("{:?}", data.meta().unwrap());
```
*/

use blowfish::{Blowfish};
use byteorder::{ByteOrder, LittleEndian};
use blowfish::cipher::{BlockDecryptMut, KeyInit};
use blowfish::cipher::generic_array::GenericArray;
use flate2::read::GzDecoder;
use std::io::Read;

pub mod error;
pub mod segments;
use segments::SegmentDataCore;

/// Base ROFL file's parser
pub struct Rofl<'a> {
    /// ROFL file's Start Header
    head: BinHeader,
    /// ROFL File's JSON metadata string
    metadata: Option<String>,
    /// ROFL File's Payload header
    payload: Option<PayloadHeader>,
    /// ROFL File's Chunk Segments
    chunks: Vec<Segment>,
    /// ROFL File's Keyframe Segments
    keyframes: Vec<Segment>,
    /// ROFL File's data
    data: &'a[u8],
    /// ROFL segments' decryption cipher
    cipher: Option<Blowfish::<byteorder::BigEndian>>,
}

/// Constants used to define which parts of a ROFL file we want to load
/// 
/// __WARNING:__ This module should disappear in the short run
pub mod consts {
    /// Load The file's header
    /// 
    /// Note that the file's header will ALWAYS be loaded
    pub const LOAD_HEAD: u16 = 0x01;
    /// Load the file's JSON metadata string
    pub const LOAD_METADATA: u16 = 0x02;
    /// Load the payload's header
    pub const LOAD_PAYLOAD_HEAD: u16 = 0x04;
    /// Load the segments' headers
    pub const LOAD_SEGMENTS_HEAD: u16 = 0x08;
    /// Load the segments' data
    pub const LOAD_SEGMENTS: u16 = 0x10;
    /// Load everything
    pub const LOAD_ALL: u16 = 0x1f;
}

impl Rofl<'_> {
    /// Length in bytes of a segment header
    const PAYLOAD_INTERNAL_HEADER_LEN: usize = 17;
    /// Starting bytes of a ROFL file
    pub const MAGIC: [u8; 4] = [82,73,79,84]; // TODO: check if 6 bytes instead of 0
    /// Get the ROFL header
    pub fn head(&self) -> &BinHeader { &self.head }
    /// Get the loaded JSON Metadata string
    pub fn meta(&self) -> Option<&String> { self.metadata.as_ref() }
    /// Get the loaded payload header
    pub fn payload(&self) -> Option<&PayloadHeader> { self.payload.as_ref() }
    /// Get the loaded chunk headers and data
    pub fn chunks(&self) -> &Vec<Segment> { &self.chunks }
    /// Get the loaded keyframe headers and data
    pub fn keyframes(&self) -> &Vec<Segment> { &self.keyframes }
    /// Load the JSON Metadata String if it is not already and returns it
    pub fn load_meta(&mut self) -> Result<&String, error::Errors> {
        if self.metadata.is_none() {
            if self.data.len() < self.head.metadata_offset() + self.head.metadata_len() {
                return Err(error::Errors::BufferTooSmall); // TODO: buffer too small
            }
            let json_metadata_str = std::str::from_utf8(
                    &self.data[self.head.metadata_offset()..self.head.metadata_offset() + self.head.metadata_len()]
            );
            if json_metadata_str.is_err() {
                return Err(error::Errors::InvalidBuffer);
            }
            self.metadata = json_metadata_str.ok().map(|s| s.to_string());
        }
        self.metadata.as_ref().ok_or(error::Errors::NoData)
    }
    /// Load the cipher key required to decrypt a segment
    fn cipher(&mut self) -> Option<&Blowfish::<byteorder::BigEndian>> {
        if self.cipher.is_none() {
            self.cipher = self.payload()
                .and_then(|p| Blowfish::<byteorder::BigEndian>::new_from_slice(&p.segment_encryption_key()[..]).ok());
        }
        self.cipher.as_ref()
    }
    /// Load the payload header if it is not already loaded and return it
    pub fn load_payload(&mut self) -> Result<Option<&PayloadHeader>, error::Errors> {
        if self.payload.is_none() {
            self.payload = Some(PayloadHeader::from_raw_section( // Fix PayloadHeader::from_raw_section to be able to fail
                &self.data[self.head.payload_header_offset()..self.head.payload_header_offset() + self.head.payload_header_len()]
            ));
        }
        Ok(self.payload.as_ref())
    }
    /// Load the segment headers if they are not already loaded and return them
    pub fn load_segments_heads(&mut self) -> Result<(), error::Errors> {
        if self.chunks.len() == 0 && self.keyframes.len() == 0 {
            self.load_payload().and_then(|payload_opt| {
                payload_opt.and_then(|p| Some(p.chunk_count() + p.keyframe_count())).ok_or(error::Errors::NoData)
            }).and_then(|segment_total| {
                for i in 0..segment_total as usize {
                    let segment = Segment::from_raw_section(&self.data[self.head.payload_offset()+Rofl::PAYLOAD_INTERNAL_HEADER_LEN*i..]);
                    if segment.is_chunk() {
                        self.chunks.push(segment);
                    } else {
                        self.keyframes.push(segment);
                    }
                }
                Ok(())
            })
        } else {
            Ok(())
        }
    }
    /// Decrypt a segment
    fn rofl_decrypt_decompress(data: &[u8], cipher: &mut Blowfish::<byteorder::BigEndian>, out: &mut Vec<u8>) -> Result<(), error::Errors>{
        let decrypted_data = blow_decrypt(data, cipher, true);
        if decrypted_data.len() < data.len() {
            return Err(error::Errors::BufferTooSmall);
        }
        let padding = decrypted_data[data.len()-1] as usize;
        if 8 < padding {
            return Err(error::Errors::InvalidBuffer);
        }
        let mut decoder = GzDecoder::new(&decrypted_data[..data.len()-padding]);
        let decoder_result = decoder.read_to_end(out);
        if decoder_result.is_err() {
            return Err(error::Errors::InvalidBuffer);
        }
        Ok(())
    }
    /// Load the segments if they are not already loaded
    /// 
    /// This function does not return anything in case of success
    pub fn load_segments(&mut self) -> Result<(), error::Errors> {
        self.load_segments_heads().and_then(|_| {
            self.cipher().ok_or(error::Errors::NoData)?;
            let payload = self.payload.as_ref().unwrap();
            // Theorical and actually loaded segment counts must match after explicit loading
            assert_eq!((payload.chunk_count() + payload.keyframe_count()) as usize, self.chunks.len() + self.keyframes.len());
            let payload_count = (payload.chunk_count() + payload.keyframe_count()) as usize;
            for i in 0..payload_count {
                let segment = if i < self.chunks.len() {
                    &mut self.chunks[i]
                } else {
                    &mut self.keyframes[i-self.chunks.len()]
                };
                if !segment.is_loaded() {
                    let payload_data_start = self.head.payload_offset() + Rofl::PAYLOAD_INTERNAL_HEADER_LEN * payload_count;
                    Rofl::rofl_decrypt_decompress(
                        &self.data[payload_data_start+segment.offset()..payload_data_start+segment.offset()+segment.len()],
                        self.cipher.as_mut().unwrap(),
                        &mut segment.data
                    )?;
                }
            }
            Ok(())
        })
    }
    /// Load a payload segment if it is not already loaded and return it
    fn load_segment(&mut self, id: u32, is_chunk: bool, slice: &[u8]) -> Result<&Segment, error::Errors> {
        // FIXME: This code should ensure that the segment is not already loaded
        let segment;
        if is_chunk {
            if id as usize - 1 >= self.chunks.len() { return Err(error::Errors::NoData); }
            segment = &mut self.chunks[id as usize - 1];
        } else {
            if id as usize - 1 >= self.keyframes.len() { return Err(error::Errors::NoData); }
            segment = &mut self.keyframes[id as usize - 1];
        }
        let payload_number = (self.payload.as_ref().unwrap().chunk_count + self.payload.as_ref().unwrap().keyframe_count) as usize;
        let decryption_key = self.payload.as_ref().unwrap().segment_encryption_key();
        let payload_data_start = self.head.payload_offset() + Rofl::PAYLOAD_INTERNAL_HEADER_LEN * payload_number;
        let decrypted_data = blowfish_decrypt(&slice[payload_data_start+segment.offset()..payload_data_start+segment.offset()+segment.len()], &decryption_key[..], false);
        if decrypted_data.len() < segment.len() {
            return Err(error::Errors::BufferTooSmall);
        }
        let padding = decrypted_data[segment.len()-1] as usize;
        if 8 < padding {
            return Err(error::Errors::InvalidBuffer);
        }
        let mut decoder = GzDecoder::new(&decrypted_data[..segment.len()-padding]);
        let decoder_result = decoder.read_to_end(&mut segment.data);
        if decoder_result.is_err() {
            return Err(error::Errors::InvalidBuffer);
        }
        Ok(segment)
    }
    /// Load a chunk if it is not already loaded and return it
    pub fn load_chunk(&mut self, id: u32, slice: &[u8]) -> Result<&Segment, error::Errors> {
        self.load_segment(id, true, slice)
    }
    /// Load a keyframe if it is not already loaded and return it
    pub fn load_keyframe(&mut self, id: u32, slice: &[u8]) -> Result<&Segment, error::Errors> {
        self.load_segment(id, false, slice)
    }
    /// Create a new Rofl instance from a ROFL file's slice
    pub fn from_slice<'a>(slice: &'a[u8], config: u16) -> Result<Rofl<'a>,()> {
        // TODO: Return meaningful errors
        if slice.len() < Rofl::MAGIC.len() || Rofl::MAGIC != slice[..Rofl::MAGIC.len()] {
            return Err(()); // TODO: magic does not exist
        }
        // FIXME: return Option<> in BinHeader initializers and control slice size
        let header = BinHeader::from_raw_source(slice);

        let metadata;
        if config & consts::LOAD_METADATA == 0 {
            metadata = None;
        } else {
            if slice.len() < header.metadata_offset() + header.metadata_len() {
                return Err(()); // TODO: buffer too small
            }
            let json_metadata_str = std::str::from_utf8(
                    &slice[header.metadata_offset()..header.metadata_offset() + header.metadata_len()]
            );
            if json_metadata_str.is_err() {
                return Err(()); // TODO: Invalid string data
            }
            metadata = json_metadata_str.ok();
        }
        let payload_head;
        let mut chunks = Vec::new();
        let mut keyframes = Vec::new();
        if config & (consts::LOAD_PAYLOAD_HEAD | consts::LOAD_SEGMENTS_HEAD | consts::LOAD_SEGMENTS) == 0 {
            payload_head = None;
        } else {
            if slice.len() < header.payload_header_offset() + header.payload_header_len() {
                return Err(()); // TODO: buffer too small
            }
            let payload = PayloadHeader::from_raw_section(
                &slice[header.payload_header_offset()..header.payload_header_offset() + header.payload_header_len()]
            );
            
            if config & (consts::LOAD_SEGMENTS_HEAD | consts::LOAD_SEGMENTS) != 0 {
                let decryption_key = payload.segment_encryption_key();
                let payload_data_start = header.payload_offset() + (Rofl::PAYLOAD_INTERNAL_HEADER_LEN * (payload.chunk_count() + payload.keyframe_count()) as usize);
                let decrypted_data;
                if config & consts::LOAD_SEGMENTS != 0 {
                    decrypted_data = blowfish_decrypt(&slice[payload_data_start..], &decryption_key[..], false);
                } else {
                    decrypted_data = Vec::new();
                }

                for i in 0..(payload.chunk_count() + payload.keyframe_count()) as usize {
                    let mut segment = Segment::from_raw_section(&slice[header.payload_offset()+Rofl::PAYLOAD_INTERNAL_HEADER_LEN*i..]);
                    if config & consts::LOAD_SEGMENTS != 0 {
                        if decrypted_data.len() < segment.offset()+segment.len() {
                            return Err(()); // TODO: buffer too small
                        }
                        let padding = decrypted_data[segment.offset()+segment.len()-1] as usize;
                        if 8 < padding {
                            return Err(()); // TODO: Sanity check error, padding is always <= 8 bytes
                        }
                        let mut decoder = GzDecoder::new(&decrypted_data[segment.offset()..segment.offset()+segment.len()-padding]);
                        let decoder_result = decoder.read_to_end(&mut segment.data);
                        if decoder_result.is_err() {
                            return Err(()); // TODO: gzip failed
                        }
                    }
                    if segment.is_chunk() {
                        chunks.push(segment);
                    } else {
                        keyframes.push(segment);
                    }
                }
            }
            payload_head = Some(payload);
        }
        Ok(Rofl {
            head: header,
            metadata: metadata.and_then(|m| Some(m.to_string())),
            payload: payload_head,
            chunks,
            keyframes,
            data: slice,
            cipher: None,
        })
    }
}

/// An iterator for lightweight scanning of a data segment
pub struct SegmentIterator<'a> {
    /// The segment's data
    data: &'a[u8],
    /// The iterator's mosition in the segment
    index: usize,
    /// Whether the iterator overflew or failed to produce a valid object during iteration
    invalid: bool,
}

impl<'a> SegmentIterator<'a> {
    /// Build a new iterator from a raw decrypted segment's slice
    pub fn new(data: &'a[u8]) -> SegmentIterator<'a> {
        SegmentIterator {
            data,
            index: 0,
            invalid: false,
        }
    }

    /// Whether the iterator is valid
    pub fn is_valid(&self) -> bool { !self.invalid }

    /// The index in the data slice the iterator is at
    /// 
    /// This should only be used for debugging purposes when
    /// is_valid returns false after an iteration
    pub fn internal_index(&self) -> usize { self.index }
    /// The data slice the iterator is moving through
    /// 
    /// This should only be used for debugging purposes when
    /// is_valid returns false after an iteration
    pub fn internal_slice(&self) -> &[u8] { self.data }
}

impl<'a> std::iter::Iterator for SegmentIterator<'a> {
    type Item = segments::GenericDataSegment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() <= self.index {
            return None;
        }
        segments::GenericDataSegment::from_slice(&self.data[self.index..])
        .and_then(|f| {
            self.index += f.len();
            Ok(f)
        }).or_else(|e|{
            self.invalid = true;
            Err(e)
        }).ok()
    }
}

/** Blowfish impl with depad */
fn blow_decrypt(cipher: &[u8], decrypt: &mut Blowfish::<byteorder::BigEndian>, depad: bool) -> Vec<u8> {
    let mut data_store = cipher.to_vec();

    for i in (0..data_store.len()).step_by(8) {
        decrypt.decrypt_block_mut(
            GenericArray::from_mut_slice(&mut data_store[i..i+8])
        );
    }

    if depad {
        let depad_size = data_store[data_store.len()-1] as usize;
        assert_eq!(data_store.len() >= depad_size, true);
        data_store.resize(data_store.len()-depad_size, 0);
    }

    data_store
}

/** Blowfish impl with depad */
fn blowfish_decrypt(cipher: &[u8], key: &[u8], depad: bool) -> Vec<u8> {
    assert_eq!(cipher.len()%8, 0);
    assert_ne!(cipher.len(), 0);

    let mut data_store = vec![0; cipher.len()];
    let mut decrypt = Blowfish::<byteorder::BigEndian>::new_from_slice(&key).unwrap();
    
    for i in (0..data_store.len()).step_by(8) {
        decrypt.decrypt_block_b2b_mut(
            GenericArray::from_slice(&cipher[i..i+8]),
            GenericArray::from_mut_slice(&mut data_store[i..i+8])
        );
    }

    if depad {
        let depad_size = data_store[data_store.len()-1] as usize;
        assert_eq!(data_store.len() >= depad_size, true);
        data_store.resize(data_store.len()-depad_size, 0);
    }

    data_store
}

/// Dezip function
/// 
/// TODO: delete this
fn dezip(dataset: &[u8]) -> Vec<u8> {
    let mut unzipped_data = Vec::new();
    let mut decoder = GzDecoder::new(dataset);
    decoder.read_to_end(&mut unzipped_data).ok();
    unzipped_data
}

/// ROFL file's header information
#[derive(Debug)]
pub struct BinHeader {
    /// The file's signature
    signature: Vec<u8>, // Fixed-size: 256 bits (or 0 if ignored)
    /// The size of the header (constant in all known examples)
    header_length: u16,
    /// Total file length
    file_length: u32,
    /// Offset in bytes from the start of the file of the metadata section
    metadata_offset: u32,
    /// Length in bytes of the metadata section
    metadata_length: u32,
    /// Offset in bytes from the start of the file of the payload header section
    payload_header_offset: u32,
    /// Length in bytes of the payload header section
    payload_header_length: u32,
    /// Offset in bytes from the start of the file of the payload section
    payload_offset: u32,
}

impl std::fmt::Display for BinHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "Header size: {0}\n",
                "File size: {1}\n",
                "Metadata offset: {2}\n",
                "Metadata length: {3}\n",
                "Payload Header offset: {4}\n",
                "Payload Header Length: {5}\n",
                "Payload offset: {6}",
            ),
            self.header_length,
            self.file_length,
            self.metadata_offset,
            self.metadata_length,
            self.payload_header_offset,
            self.payload_header_length,
            self.payload_offset,
            )
    }
}

impl BinHeader {
    /// Get the file's signature
    pub fn signature(&self) -> &Vec<u8> {
        &self.signature
    }
    /// Get the file's header length
    pub fn header_len(&self) -> usize {
        self.header_length as usize
    }
    /// Get the file's length in bytes as per its binary data
    /// 
    /// This may not match the actual file's length if an error occured
    pub fn file_len(&self) -> usize {
        self.file_length as usize
    }
    /// Length of the file's metadata section
    /// 
    /// This should not be required in normal use
    pub fn metadata_len(&self) -> usize {
        self.metadata_length as usize
    }
    /// Offset of the file's metadata section
    /// 
    /// This should not be required in normal use
    pub fn metadata_offset(&self) -> usize {
        self.metadata_offset as usize
    }
    /// Length of the file's payload header section
    /// 
    /// This should not be required in normal use
    pub fn payload_header_len(&self) -> usize {
        self.payload_header_length as usize
    }
    /// Offset of the file's payload header section
    /// 
    /// This should not be required in normal use
    pub fn payload_header_offset(&self) -> usize {
        self.payload_header_offset as usize
    }
    /// Offset of the file's payload section
    /// 
    /// This should not be required in normal use
    pub fn payload_offset(&self) -> usize {
        self.payload_offset as usize
    }
    
    /// Create a new header from a manually-loaded file start section
    /// 
    /// Use from_raw_source instead
    #[warn(deprecated)]
    fn from_raw_section(data: &[u8]) -> BinHeader {
        BinHeader {
            signature: Vec::from(&data[6..262]),
            header_length: LittleEndian::read_u16(&data[262..]),
            file_length: LittleEndian::read_u32(&data[264..]),
            metadata_offset: LittleEndian::read_u32(&data[268..]),
            metadata_length: LittleEndian::read_u32(&data[272..]),
            payload_header_offset: LittleEndian::read_u32(&data[276..]),
            payload_header_length: LittleEndian::read_u32(&data[280..]),
            payload_offset: LittleEndian::read_u32(&data[284..]),
        }
    }
    /// Create a new header from a manually-loaded file start section
    /// 
    /// This will be replaced by a from_raw function in the future
    pub fn from_raw_source(data: &[u8]) -> BinHeader {
        BinHeader::from_raw_section(&data[0..])
    }
}

/// ROFL file's payload header information
#[derive(Debug)]
pub struct PayloadHeader {
    /// The ID of the game
    match_id: u64,
    /// The duration of the game in milliseconds
    match_length: u32,
    /// The number of keyframes in the payload
    keyframe_count: u32,
    /// The number of chunks in the payload
    chunk_count: u32,
    /// The last chunk used to load data before the game
    end_startup_chunk_id: u32,
    /// The chunk that contains the game's data
    start_game_chunk_id: u32,
    /// The duration covered by a single keyframe
    keyframe_interval: u32,
    /// The length of the encrypted key used to encrypt the game's data
    /// 
    /// NOTE: This attribute should be removed in later versions
    encryption_key_length: u16,
    /// The encrypted key of the game's payload
    encryption_key: Vec<u8>,
}

impl PayloadHeader {
    /// Get the ID of the game
    pub fn id(&self) -> u64 { self.match_id }
    /// Get the duration of the game in milliseconds
    pub fn duration(&self) -> u32 { self.match_length }
    /// Get the number of keyframes
    pub fn keyframe_count(&self) -> u32 { self.keyframe_count }
    /// Get the number of chunks
    pub fn chunk_count(&self) -> u32 { self.chunk_count }
    /// Get the last loading chunk
    pub fn load_end_chunk(&self) -> u32 { self.end_startup_chunk_id }
    /// Get the first game chunk
    pub fn game_start_chunk(&self) -> u32 { self.start_game_chunk_id }
    /// Get the duration of a keyframe in milliseconds
    pub fn keyframe_interval(&self) -> u32 { self.keyframe_interval }
    /// Get the encrypted payload encryption key
    pub fn encryption_key(&self) -> &str { std::str::from_utf8(&self.encryption_key[..]).unwrap() }
    /// Get the decrypted payload encryption key
    pub fn segment_encryption_key(&self) -> Vec<u8> {
        let key = base64::decode(&self.encryption_key).unwrap();
        blowfish_decrypt(&key[..], self.match_id.to_string().as_bytes(), true)
    }
    /// Decrypt a raw payload data segment from the same file as this payload
    /// 
    /// WARN: This function will be changed in the future to return a Result
    #[warn(deprecated)]
    pub fn expand_payload_data(slice: &[u8], key: &[u8]) -> Vec<u8> {
        let zipped = blowfish_decrypt(slice, key, true);
        let d = dezip(&zipped[..]);
        d
    }
    fn from_raw_section(data: &[u8]) -> PayloadHeader {
        PayloadHeader {
            match_id: LittleEndian::read_u64(&data[..8]),
            match_length: LittleEndian::read_u32(&data[8..12]),
            keyframe_count: LittleEndian::read_u32(&data[12..16]),
            chunk_count: LittleEndian::read_u32(&data[16..20]),
            end_startup_chunk_id: LittleEndian::read_u32(&data[20..24]),
            start_game_chunk_id: LittleEndian::read_u32(&data[24..28]),
            keyframe_interval: LittleEndian::read_u32(&data[28..32]),
            encryption_key_length: LittleEndian::read_u16(&data[32..34]),
            encryption_key: data[(34 as usize)..((34+LittleEndian::read_u16(&data[32..34])) as usize)].to_vec(),
        }
    }
}

impl std::fmt::Display for PayloadHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "Match ID: {0}\n",
                "Match Length: {1} ms\n",
                "Keyframe count: {2}\n",
                "Last loading Chunk: {3}\n",
                "First game chunk: {4}\n",
                "Total chunk count: {5}\n",
                "Keyframe interval: {6}\n",
                "Encryption key ({7} chars): {8:?}",
            ),
            self.match_id,
            self.match_length,
            self.keyframe_count,
            self.start_game_chunk_id,
            self.end_startup_chunk_id,
            self.chunk_count,
            self.keyframe_interval,
            self.encryption_key_length,
            std::str::from_utf8(&self.encryption_key[..]).unwrap(),
        )
    }
}

/// Container for Chunk and Keyframe data
#[derive(Debug)]
pub struct Segment {
    /// The segment's ID
    id: u32,
    /// Whether the segment is a chunk or a keyframe
    segment_type: u8,
    /// Length of the segment's data
    length: u32,
    /// ID of the first associated Chunk (if this segment is a keyframe), else 0
    chunk_id: u32,
    /// Internal offset of the segment's data
    offset: u32,
    /// Segment's data (if it is loaded)
    data: Vec<u8>,
}

/// Internal enum that maps segment type high-level names to their numerical value
#[derive(Debug)] #[repr(u8)]
enum SegmentType {
    Chunk = 1,
    Keyframe = 2,
}

impl Segment {
    /// The segment's ID
    pub fn id(&self) -> u32 { self.id }

    /// The length in bytes of the segment's data
    pub fn len(&self) -> usize { self.length as usize }

    /// The offset in bytes from the segment headers' end at which the segment's data starts
    pub fn offset(&self) -> usize { self.offset as usize }

    /// Whether the segment's data section is loaded
    pub fn is_loaded(&self) -> bool { !self.data.is_empty() }

    /// Get the raw segment's data
    pub fn data(&self) -> &Vec<u8> { &self.data }
    /// Build a new segment headet from a payload's data
    /// 
    /// This does not load the segment's data section
    pub fn from_raw_section(data: &[u8]) -> Segment {
        Segment {
            id: LittleEndian::read_u32(&data[0..]),
            segment_type: data[4],
            length: LittleEndian::read_u32(&data[5..]),
            chunk_id: LittleEndian::read_u32(&data[9..]),
            offset: LittleEndian::read_u32(&data[13..]),
            data: Vec::new(),
        }
    }
    /// Whether this segment is a chunk
    pub fn is_chunk(&self) -> bool {
        self.segment_type == SegmentType::Chunk as u8
    }

    /// Whether this segment is a keyframe
    pub fn is_keyframe(&self) -> bool {
        self.segment_type == SegmentType::Keyframe as u8
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "{} {} (len: {}, next: {}, offset: {}, loaded: {})",
            if self.is_chunk() {"Chunk"} else { if self.is_keyframe() {"Keyframe"} else {"Segment"} },
            self.id, self.length, self.chunk_id, self.offset, self.data.len() > 0,
        )
    }
}
