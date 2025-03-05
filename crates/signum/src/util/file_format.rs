use super::FourCC;

/// Trait for values that encode a kind of file format, e.g. [`crate::chsets::FontKind`]
pub trait FileFormatKind {
    /// Get the extension used for files of this type
    fn extension(&self) -> &'static str;

    /// Get the magic used to detect files of this type
    fn magic(&self) -> FourCC;

    /// Get the file format name for this printer kind
    fn file_format_name(&self) -> &'static str;
}
