#[cfg(feature="payload")]
use blowfish::{
    Blowfish,
    cipher::{
        BlockDecryptMut, KeyInit,
        generic_array::GenericArray,
    },
};
use crate::{Errors, PayloadHeader, Segment};
use crate::SEGMENT_HEADER_LEN;

/// An iterator for lightweight scanning of data segments in a payload
pub struct PayloadIterator<'a> {
    /// The segment's data
    data: &'a[u8],
    /// The iterator's mosition in the segment
    index: usize,
    /// The number of payload segments to go through
    segment_count: usize,
    /// Code of the last error that occured during an iteration
    last_error: Option<Errors>,
    /// The decryption cipher key
    #[cfg(feature="payload")]
    key: Blowfish::<byteorder::BigEndian>,
    /// Whether to parse segment data or only iterate over headers
    parse_data: bool,
}

impl<'a> PayloadIterator<'a> {
    /// Build a new iterator from a raw decrypted segment's slice
    pub fn new(data: &'a[u8], head: &'_ PayloadHeader, parse_data: bool) -> Result<PayloadIterator<'a>, crate::error::Errors> {
        let segment_count = (head.chunk_count()+head.keyframe_count()) as usize;
        if data.len() < segment_count*SEGMENT_HEADER_LEN {
            return Err(Errors::BufferTooSmall);
        }
        Ok(PayloadIterator {
            data,
            segment_count,
            parse_data,
            index: 0,
            last_error: None,
            #[cfg(feature="payload")]
            key: Blowfish::<byteorder::BigEndian>::new_from_slice(&head.segment_encryption_key()[..]).unwrap(),
        })
    }

    /// Whether the iterator is valid
    pub fn is_valid(&self) -> bool { self.last_error.is_none() }
    /// Get the last error that occured
    /// 
    /// Panics if no error occured
    pub fn to_error(self) -> Errors {self.last_error.unwrap()}
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

impl<'a> std::iter::Iterator for PayloadIterator<'a> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.segment_count {
            return None;
        }
        #[allow(unused_mut)]
        Segment::from_slice(&self.data[self.index*SEGMENT_HEADER_LEN..])
        .and_then(|mut f| {
            #[cfg(feature="payload")]
            {
                if self.parse_data {
                    let segment_data_start = SEGMENT_HEADER_LEN * self.segment_count + f.offset();
                    if self.data.len() < segment_data_start + f.len() {
                        return Err(Errors::BufferTooSmall);
                    } else {
                        decrypt_segment(&self.data[segment_data_start..segment_data_start+f.len()], f.data_mut(), &mut self.key)?;
                    }
                }
            }
            self.index += 1;
            Ok(f)
        }).or_else(|e|{
            self.last_error = Some(e);
            Err(Errors::NoData)
        }).ok()
    }
}

/// Decrypt a payload segment.
/// The provided slice must match the exact extent of the encrypted data
#[cfg(feature="payload")]
fn decrypt_segment(cipher: &[u8], out: &mut Vec<u8>, key: &mut Blowfish::<byteorder::BigEndian>) -> Result<(), crate::error::Errors> {
    use std::io::Read;

    let mut data_store = cipher.to_vec();

    for i in (0..data_store.len()).step_by(8) {
        key.decrypt_block_mut(
            GenericArray::from_mut_slice(&mut data_store[i..i+8])
        );
    }

    let depad_size = data_store[data_store.len()-1] as usize;
    assert_eq!(data_store.len() >= depad_size, true);
    data_store.resize(data_store.len()-depad_size, 0);

    let mut decoder = flate2::read::GzDecoder::new(&data_store[..]);
    let decoder_result = decoder.read_to_end(out);
    if decoder_result.is_err() {
        return Err(Errors::InvalidBuffer);
    }
    Ok(())
}