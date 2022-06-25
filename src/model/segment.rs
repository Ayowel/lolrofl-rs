/*!
Data segments that make up a payload as well as their section components
*/

use byteorder::{ByteOrder, LittleEndian};
use crate::iter::SegmentIterator;

/// Length in bytes of a segment header
pub(crate) const SEGMENT_HEADER_LEN: usize = 17;

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
    /// Get the raw segment's data as a mutable Vec
    /// 
    /// __WARNING:__This should only be used if you decrypt/load segment data yourself
    pub fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
    /// Build a new segment headet from a payload's data
    /// 
    /// This does not load the segment's data section
    /// 
    /// Use from_slice instead
    #[warn(deprecated)]
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
    /// Build a new segment headet from a payload's data
    /// 
    /// This does not load the segment's data section
    pub fn from_slice(data: &[u8]) -> Result<Segment, crate::error::Errors> {
        if data.len()<SEGMENT_HEADER_LEN {
            Err(crate::error::Errors::BufferTooSmall)
        } else {
            Ok(Segment::from_raw_section(data))
        }
    }
    /// Attach data to the Segment
    /// 
    /// CAUTION: no validation is performed on the provided data,
    /// ensure that the provided vec is the segment's vec
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
    /// Whether this segment is a chunk
    pub fn is_chunk(&self) -> bool {
        self.segment_type == SegmentType::Chunk as u8
    }
    /// Whether this segment is a keyframe
    pub fn is_keyframe(&self) -> bool {
        self.segment_type == SegmentType::Keyframe as u8
    }
    /// Get a section iterator over the data of the segment
    pub fn section_iter<'a>(&'a self) -> Result<SegmentIterator<'a>, crate::Errors> {
        if self.data.len() == 0 {
            Err(crate::Errors::NoData)
        } else {
            Ok(SegmentIterator::new(&self.data[..]))
        }
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
