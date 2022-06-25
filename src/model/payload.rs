#[cfg(feature="payload")]
use blowfish::{
    Blowfish,
    cipher::{
        BlockDecryptMut, KeyInit,
        generic_array::GenericArray,
    },
};
use byteorder::{ByteOrder, LittleEndian};

/** Blowfish impl with depad */
#[cfg(feature="payload")]
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
    #[cfg(feature="payload")]
    pub(crate) fn segment_encryption_key(&self) -> Vec<u8> {
        let key = base64::decode(&self.encryption_key).unwrap();
        blowfish_decrypt(&key[..], self.match_id.to_string().as_bytes(), true)
    }
    pub(crate) fn from_raw_section(data: &[u8]) -> PayloadHeader {
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
