use byteorder::{ByteOrder, LittleEndian};

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
