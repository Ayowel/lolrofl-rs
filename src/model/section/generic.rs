use byteorder::{ByteOrder, LittleEndian};
use crate::{Errors, section::SectionCore};

/// How time may be expressed within a section packet
pub enum PacketTime {
    /// Absolute time from the game's start
    Absolute(f32),
    /// Relative time in milliseconds from the last section
    Relative(u8),
}

/// Generic data container used for quick scans and iteration over a ROFL segment's data
#[derive(Clone, Debug)]
pub struct GenericSection<'a> {
    /// Length of the constant part of the section
    core_len: usize,
    /// Type of the data within the packet
    data_type: u32,
    /// Data of the section
    data: &'a[u8],
}

impl GenericSection<'_> {
    /// Whether the time is encoded on 1 or 4 bytes
    const TIME_BYTE: u8 = 0x80;
    /// Whether a byte type is in the packet or the last packet's type
    /// should be used
    ///
    /// Backward compatibility would require to handle dynamically the
    /// associated size as it seems that the type bytes used to be only
    /// 1-byte long
    const TYPE_BYTE: u8 = 0x40;
    /// Whether the block's parameters are encoded on 1 or 4 bytes
    const BPARAM_BYTE: u8 = 0x20;
    /// Whether the block's content's length is encoded on 1 or 4 bytes
    const LENGTH_BYTE: u8 = 0x10;
    /// Get full raw internal section
    #[warn(deprecated)]
    pub fn bytes(&self) -> &[u8] { &self.data }
    /// Get the section's time
    pub fn time(&self) -> PacketTime {
        if self.data[0] & GenericSection::TIME_BYTE != 0 {
            PacketTime::Relative(self.data[1])
        } else {
            PacketTime::Absolute(f32::from_bits(LittleEndian::read_u32(&self.data[1..5])))
        }
    }
    /// The parameters for this packet. The slice may be either 1 or 4 bytes long
    /// 
    pub fn params(&self) -> &[u8] {
        let marker = self.data[0];

        let params_offset = 1
            + if marker & GenericSection::TIME_BYTE != 0 {1} else {4}
            + if marker & GenericSection::LENGTH_BYTE != 0 {1} else {4}
            + if marker & GenericSection::TYPE_BYTE != 0 {0} else {2};
        let params_len = if marker & GenericSection::BPARAM_BYTE != 0 {1} else {4};

        &self.data[params_offset..params_offset+params_len]
    }
    /// The type of the data within the packet
    ///
    /// Types should be within u16's space, however a larger type
    /// is used for future-proofing
    pub fn data_type(&self) -> u32 {
        self.data_type
    }
    /// Create a new GenericDataSegment from a slice.
    ///
    /// Providing a slice with the data of multiple GenericDataSegment returns
    /// the segment that starts at the first byte of the slice
    pub fn from_slice(slice: &[u8], last_datatype: Option<u32>) -> Result<GenericSection, crate::error::Errors> {
        if slice.len() == 0 {
            return Err(crate::error::Errors::NoData);
        }
        let marker = slice[0];

        let length_offset = 1 +
            if marker & GenericSection::TIME_BYTE != 0 {1} else {4};
        let type_offset = length_offset
            + if marker & GenericSection::LENGTH_BYTE != 0 {1} else {4};
        let core_len = type_offset
            + if marker & GenericSection::TYPE_BYTE != 0 {0} else {2}
            + if marker & GenericSection::BPARAM_BYTE != 0 {1} else {4};

        if slice.len() < core_len { return Err(Errors::BufferTooSmall); }
        let data_len =
            if marker & GenericSection::LENGTH_BYTE != 0 {
                slice[length_offset] as usize
            } else {
                LittleEndian::read_u32(&slice[length_offset..]) as usize
            };

        if slice.len() < core_len + data_len { return Err(Errors::BufferTooSmall); }

        let data_type =
            if marker & GenericSection::TYPE_BYTE != 0 {
                if last_datatype.is_none() { return Err(Errors::NoData); }
                last_datatype.unwrap()
            } else {
                LittleEndian::read_u16(&slice[type_offset..]) as u32
            };
        Ok(GenericSection {
            core_len,
            data: &slice[..core_len+data_len],
            data_type,
        })
    }
}

impl SectionCore for GenericSection<'_> {
    const KIND: u8 = 0;
    fn kind(&self) -> u8 {self.data[0]}
    fn core_len(&self) -> usize { self.core_len }
    fn data_len(&self) -> usize { self.data.len()-self.core_len }
    fn raw_data(&self) -> std::option::Option<&[u8]> {
        if self.data_len() == 0 { None } else { Some(&self.data[self.core_len..]) }
    }
}
