/*!
 * The sections that make up a payload segment
 */

mod generic;
pub use generic::*;

/// A generic interface for data segments' sections
/// 
/// This trait will probably be renamed in future versions
pub trait SectionCore {
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
