use byteorder::{ByteOrder, LittleEndian};
use crate::section::SectionCore;

/// Generic data container used for quick scans and iteration over a ROFL segment's data
#[derive(Clone, Debug)]
pub struct GenericSection<'a> {
    /// Length of the constant part of the section
    core_len: usize,
    /// Data of the section
    data: &'a[u8],
}

impl GenericSection<'_> {
    /// Get full raw internal section
    #[warn(deprecated)]
    pub fn bytes(&self) -> &[u8] { &self.data }

    /// Create a new GenericDataSegment from a slice.
    ///
    /// Providing a slice with the data of multiple GenericDataSegment returns
    /// the segment that starts at the first byte of the slice
    pub fn from_slice(slice: &[u8]) -> Result<GenericSection, crate::error::Errors> {
        if slice.len() == 0 {
            return Err(crate::error::Errors::NoData);
        }
        match slice[0] {
            1 | 2 => GenericSection::buffer_to_generic(&slice, 5, 2, 15), // Start segment
            17 => GenericSection::buffer_to_generic(&slice, 5, 1, 12),
            32 => GenericSection::buffer_to_generic(&slice, 4, 1, 12),
            33 => GenericSection::buffer_to_generic(&slice, 5, 2, 12),
            49 | 50 => GenericSection::buffer_to_generic(&slice, 5, 1, 9),
            81 => GenericSection::buffer_to_generic(&slice, 5, 1, 10),  
            113 => GenericSection::buffer_to_generic(&slice, 5, 1, 7),
            129 | 130 => GenericSection::buffer_to_generic(&slice, 2, 2, 12),
            145 | 146 | 147 => GenericSection::buffer_to_generic(&slice, 2, 1, 9),
            149 => GenericSection::buffer_to_generic(&slice, 4, 1, 13),
            161 | 162 => GenericSection::buffer_to_generic(&slice, 2, 2, 9),
            177 | 178 | 179 => GenericSection::buffer_to_generic(&slice, 2, 1, 6),
            193 => GenericSection::buffer_to_generic(&slice, 2, 2, 10),
            209 => GenericSection::buffer_to_generic(&slice, 2, 1, 7),
            225 | 226 => GenericSection::buffer_to_generic(&slice, 2, 2, 7),
            241 | 242 => GenericSection::buffer_to_generic(&slice, 2, 1, 4),
            _ => Err(crate::error::Errors::InvalidBuffer),
        }
    }

    /// Internal helper function to safely build a GenericDataSegment from a slice
    fn buffer_to_generic(slice: &[u8], size_offset: usize, size_len: usize, core_size: usize) -> Result<GenericSection, crate::error::Errors> {
        if slice.len() < core_size {
            eprintln!("Failed due to insufficient slice length for core ({} of {} expected)", slice.len(), core_size);
            return Err(crate::error::Errors::BufferTooSmall);
        }
        let data_len = match size_len {
            0 => 0,
            1 => slice[size_offset] as usize,
            2 => LittleEndian::read_u16(&slice[size_offset..]) as usize,
            _ => panic!("Unsupported size length: {}", size_len),
        };
        if slice.len() < core_size + data_len {
            eprintln!("Failed due to insufficient slice length for data ({} of {} + {} expected)", slice.len(), core_size, data_len);
            return Err(crate::error::Errors::BufferTooSmall);
        }

        Ok(GenericSection {
            core_len: core_size,
            data: &slice[0..core_size+data_len],
        })
    }
}

impl SectionCore for GenericSection<'_> {
    const KIND: u8 = 0;
    fn kind(&self) -> u8 {self.data[0]}
    fn core_len(&self) -> usize { self.core_len }
    fn data_len(&self) -> usize { self.data.len()-self.core_len }
    fn raw_data(&self) -> std::option::Option<&[u8]> { if self.data_len() == 0 { None } else {Some(&self.data[self.core_len..])} }
}
