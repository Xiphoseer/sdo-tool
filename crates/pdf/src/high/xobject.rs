use crate::common::ImageMetadata;

/// An embedded object resource
#[derive(Debug)]
pub enum XObject {
    /// An image
    Image(Image),
}

#[derive(Debug)]
/// An Image resource
pub struct Image {
    /// The metadata for this image
    pub meta: ImageMetadata,
    /// The data for the image
    pub data: Vec<u8>,
}

impl From<Image> for XObject {
    fn from(value: Image) -> Self {
        XObject::Image(value)
    }
}
