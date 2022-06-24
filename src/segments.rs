//! Definition of data structures used in the payload's segments of a ROFL file
use byteorder::{ByteOrder, LittleEndian};
use crate::error;

/// A generic interface for data segments' sections
/// 
/// This trait will probably be renamed in future versions
pub trait SegmentDataCore {
    /// The supported section's ID
    /// 
    /// This will probably be deprecated in future releases
    const KIND: u8;
    /// Get the supported section's ID
    /// 
    /// This will probably be deprecated in future releases
    #[inline]
    fn kind(&self) -> u8 {Self::KIND}
    /// Get the length of the constant part of the section
    fn core_len(&self) -> usize;
    /// Get the length of the variable part of the section
    fn data_len(&self) -> usize;
    /// Get the full length of the section
    #[inline]
    fn len(&self) -> usize {self.core_len() + self.data_len()}
    /// Get the raw variable part of the section if any and supported
    fn raw_data(&self) -> Option<&[u8]> {None}
}

/// 9-bytes section at the start of a data segment
/// 
/// FIXME: This is a 15-bytes + X section, not 9-bytes
#[derive(Clone, Debug)]
pub struct StartSegment {
    /// In-game timestamp for the start of the segment
    timestamp: f32,
    /// TODO
    len: u16,
    /// TODO
    pos_7: u16,
    /// Variable data
    data: Vec<u8>,
}

impl StartSegment {
    pub fn from_slice(slice: &[u8]) -> Result<StartSegment, error::Errors> {
        // FIXME: This was written when the header's size was believed to be 9, not 15
        if slice.len() < 15 {
            return Err(error::Errors::BufferTooSmall);
        }
        let len = LittleEndian::read_u16(&slice[5..7]);
        if slice.len() < 15 + len as usize {
            return Err(error::Errors::BufferTooSmall);
        }
        Ok(StartSegment {
            timestamp: f32::from_bits(LittleEndian::read_u32(&slice[1..5])),
            len,
            pos_7: LittleEndian::read_u16(&slice[7..9]),
            data: slice[..15 + len as usize].to_vec(),
        })
    }

    pub fn timestamp(&self) -> f32 {
        self.timestamp
    }

    pub fn p7(&self) -> u16 { self.pos_7 }
}

impl SegmentDataCore for StartSegment {
    const KIND: u8 = 1;
    #[inline]
    fn core_len(&self) -> usize {15}
    #[inline]
    fn data_len(&self) -> usize {self.len as usize}
    #[inline]
    fn raw_data(&self) -> Option<&[u8]> {Some(&self.data[..])}
}

/// Generic data container used for quick scans and iteration over a ROFL segment's data
#[derive(Clone, Debug)]
pub struct GenericDataSegment<'a> {
    /// Length of the constant part of the section
    core_len: usize,
    /// Data of the section
    data: &'a[u8],
}

impl GenericDataSegment<'_> {
    /// Get full raw internal section
    #[warn(deprecated)]
    pub fn bytes(&self) -> &[u8] { &self.data }

    /// Create a new GenericDataSegment from a slice.
    ///
    /// Providing a slice with the data of multiple GenericDataSegment returns
    /// the segment that starts at the first byte of the slice
    pub fn from_slice(slice: &[u8]) -> Result<GenericDataSegment, error::Errors> {
        if slice.len() == 0 {
            return Err(error::Errors::NoData);
        }
        match slice[0] {
            1 | 2 => GenericDataSegment::buffer_to_generic(&slice, 5, 2, 15), // Start segment
            17 => GenericDataSegment::buffer_to_generic(&slice, 5, 1, 12),
            32 => GenericDataSegment::buffer_to_generic(&slice, 4, 1, 12),
            33 => GenericDataSegment::buffer_to_generic(&slice, 5, 2, 12),
            49 | 50 => GenericDataSegment::buffer_to_generic(&slice, 5, 1, 9),
            81 => GenericDataSegment::buffer_to_generic(&slice, 5, 1, 10),  
            113 => GenericDataSegment::buffer_to_generic(&slice, 5, 1, 7),
            129 | 130 => GenericDataSegment::buffer_to_generic(&slice, 2, 2, 12),
            145 | 146 | 147 => GenericDataSegment::buffer_to_generic(&slice, 2, 1, 9),
            149 => GenericDataSegment::buffer_to_generic(&slice, 4, 1, 13),
            161 | 162 => GenericDataSegment::buffer_to_generic(&slice, 2, 2, 9),
            177 | 178 | 179 => GenericDataSegment::buffer_to_generic(&slice, 2, 1, 6),
            193 => GenericDataSegment::buffer_to_generic(&slice, 2, 2, 10),
            209 => GenericDataSegment::buffer_to_generic(&slice, 2, 1, 7),
            225 | 226 => GenericDataSegment::buffer_to_generic(&slice, 2, 2, 7),
            241 | 242 => GenericDataSegment::buffer_to_generic(&slice, 2, 1, 4),
            _ => Err(error::Errors::InvalidBuffer),
        }
    }

    /// Internal helper function to safely build a GenericDataSegment from a slice
    fn buffer_to_generic(slice: &[u8], size_offset: usize, size_len: usize, core_size: usize) -> Result<GenericDataSegment, error::Errors> {
        if slice.len() < core_size {
            eprintln!("Failed due to insufficient slice length for core ({} of {} expected)", slice.len(), core_size);
            return Err(error::Errors::BufferTooSmall);
        }
        let data_len = match size_len {
            0 => 0,
            1 => slice[size_offset] as usize,
            2 => LittleEndian::read_u16(&slice[size_offset..]) as usize,
            _ => panic!("Unsupported size length: {}", size_len),
        };
        if slice.len() < core_size + data_len {
            eprintln!("Failed due to insufficient slice length for data ({} of {} + {} expected)", slice.len(), core_size, data_len);
            return Err(error::Errors::BufferTooSmall);
        }

        Ok(GenericDataSegment {
            core_len: core_size,
            data: &slice[0..core_size+data_len],
        })
    }
}

impl SegmentDataCore for GenericDataSegment<'_> {
    const KIND: u8 = 0;
    fn kind(&self) -> u8 {self.data[0]}
    fn core_len(&self) -> usize { self.core_len }
    fn data_len(&self) -> usize { self.data.len()-self.core_len }
    fn raw_data(&self) -> std::option::Option<&[u8]> { if self.data_len() == 0 { None } else {Some(&self.data[self.core_len..])} }
}
