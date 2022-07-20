use crate::{Errors, section::{GenericSection, SectionCore}};

/// An iterator for lightweight scanning of data sections in a segment
pub struct SegmentIterator<'a> {
    /// The segment's data
    data: &'a[u8],
    /// The iterator's mosition in the segment
    index: usize,
    /// Code of the last error that occured during an iteration
    last_error: Option<Errors>,
    /// Type of the last packet parsed, required
    last_type: Option<u32>,
}

impl<'a> SegmentIterator<'a> {
    /// Build a new iterator from a raw decrypted segment's slice
    pub fn new(data: &'a[u8]) -> SegmentIterator<'a> {
        SegmentIterator {
            data,
            index: 0,
            last_error: None,
            last_type: None,
        }
    }
    /// Whether the iterator is valid
    pub fn is_valid(&self) -> bool { self.last_error.is_none() }
    /// Get the last error that occured
    /// 
    /// Panics if no error occured
    pub fn error(&self) -> &Errors {self.last_error.as_ref().unwrap()}
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
    type Item = GenericSection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() <= self.index {
            return None;
        }
        GenericSection::from_slice(&self.data[self.index..], self.last_type)
        .and_then(|f| {
            self.index += f.len();
            self.last_type = Some(f.data_type());
            Ok(f)
        }).or_else(|e|{
            self.last_error = Some(e);
            Err(Errors::NoData)
        }).ok()
    }
}
