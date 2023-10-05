#![warn(missing_docs)]
use image::ColorType;
pub use image::{DynamicImage, ImageFormat, ImageResult};
use serde::{Deserialize, Serialize};

use super::{ImageMetaData, SerialImageBuffer};

/// Dynamic serial image enumeration. This data type encapsulates the specific serial image data types.
///
/// The enumeration variants are [`DynamicSerialImage::U8`], [`DynamicSerialImage::U16`], [`DynamicSerialImage::F32`].
///
/// # Traits
/// [`DynamicSerialImage`] implements the [`std::clone::Clone`], [`std::convert::From`], [`std::convert::TryFrom`], [`std::convert::Into`] and [`std::fmt::Debug`] traits.
///
/// Specifically, the following conversions are implemented:
///
/// With [`std::convert::From`]:
///  * [`DynamicSerialImage`] <-> [`DynamicImage`]
///  * [`DynamicSerialImage`] <- [`SerialImageData<u8>`]
///  * [`DynamicSerialImage`] <- [`SerialImageData<u16>`]
///  * [`DynamicSerialImage`] <- [`SerialImageData<f32>`]
///
/// With [`std::convert::TryFrom`]:
///  * [`DynamicImage`] <-> [`SerialImageData<u8>`]
///  * [`DynamicImage`] <-> [`SerialImageData<u16>`]
///  * [`DynamicImage`] <-> [`SerialImageData<f32>`]
///  
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum DynamicSerialImage {
    /// 8-bit unsigned integer image data.
    U8(SerialImageBuffer<u8>),
    /// 16-bit unsigned integer image data.
    U16(SerialImageBuffer<u16>),
    /// 32-bit floating point image data.
    F32(SerialImageBuffer<f32>),
}

impl DynamicSerialImage {
    /// Get the image metadata.
    pub fn get_metadata(&self) -> Option<&ImageMetaData> {
        match self {
            DynamicSerialImage::U8(value) => value.get_metadata(),
            DynamicSerialImage::U16(value) => value.get_metadata(),
            DynamicSerialImage::F32(value) => value.get_metadata(),
        }
    }

    /// Get a mutable reference to the image metadata.
    pub fn get_mut_metadata(&mut self) -> Option<&mut ImageMetaData> {
        match self {
            DynamicSerialImage::U8(value) => value.get_mut_metadata(),
            DynamicSerialImage::U16(value) => value.get_mut_metadata(),
            DynamicSerialImage::F32(value) => value.get_mut_metadata(),
        }
    }

    /// Update the image metadata.
    pub fn set_metadata(&mut self, meta: ImageMetaData) {
        match self {
            DynamicSerialImage::U8(value) => value.set_metadata(Some(meta)),
            DynamicSerialImage::U16(value) => value.set_metadata(Some(meta)),
            DynamicSerialImage::F32(value) => value.set_metadata(Some(meta)),
        }
    }

    /// Get image width.
    pub fn width(&self) -> usize {
        match self {
            DynamicSerialImage::U8(value) => value.width(),
            DynamicSerialImage::U16(value) => value.width(),
            DynamicSerialImage::F32(value) => value.width(),
        }
    }

    /// Get image height.
    pub fn height(&self) -> usize {
        match self {
            DynamicSerialImage::U8(value) => value.height(),
            DynamicSerialImage::U16(value) => value.height(),
            DynamicSerialImage::F32(value) => value.height(),
        }
    }

    /// Get the underlying SerialImageBuffer<u8> if the image is of type [`DynamicSerialImage::U8`].
    pub fn as_u8(&self) -> Option<&SerialImageBuffer<u8>> {
        match self {
            DynamicSerialImage::U8(value) => Some(value),
            _ => None,
        }
    }

    /// Get the underlying SerialImageBuffer<u16> if the image is of type [`DynamicSerialImage::U16`].
    pub fn as_u16(&self) -> Option<&SerialImageBuffer<u16>> {
        match self {
            DynamicSerialImage::U16(value) => Some(value),
            _ => None,
        }
    }

    /// Get the underlying SerialImageBuffer<f32> if the image is of type [`DynamicSerialImage::F32`].
    pub fn as_f32(&self) -> Option<&SerialImageBuffer<f32>> {
        match self {
            DynamicSerialImage::F32(value) => Some(value),
            _ => None,
        }
    }

    /// Saves the buffer to a file at the path specified.
    ///
    ///The image format is derived from the file extension.
    /// See [`image::dynimage::save_buffer_with_format`] for supported types.
    pub fn save(&self, path: &str) -> ImageResult<()> {
        let img: DynamicImage = self.into();
        img.save(path)
    }
}

impl From<DynamicImage> for DynamicSerialImage {
    fn from(value: DynamicImage) -> DynamicSerialImage {
        let color = value.color();
        match color {
            ColorType::L8 | ColorType::Rgb8 | ColorType::Rgba8 | ColorType::La8 => {
                DynamicSerialImage::U8(value.try_into().unwrap())
            }
            ColorType::L16 | ColorType::Rgb16 | ColorType::Rgba16 | ColorType::La16 => {
                DynamicSerialImage::U16(value.try_into().unwrap())
            }
            ColorType::Rgb32F | ColorType::Rgba32F => {
                DynamicSerialImage::F32(value.try_into().unwrap())
            }
            _ => {
                panic!("Unsupported image type");
            }
        }
    }
}

impl From<&DynamicImage> for DynamicSerialImage {
    fn from(value: &DynamicImage) -> Self {
        let color = value.color();
        match color {
            ColorType::L8 | ColorType::Rgb8 | ColorType::Rgba8 | ColorType::La8 => {
                DynamicSerialImage::U8(value.try_into().unwrap())
            }
            ColorType::L16 | ColorType::Rgb16 | ColorType::Rgba16 | ColorType::La16 => {
                DynamicSerialImage::U16(value.try_into().unwrap())
            }
            ColorType::Rgb32F | ColorType::Rgba32F => {
                DynamicSerialImage::F32(value.try_into().unwrap())
            }
            _ => {
                panic!("Unsupported image type");
            }
        }
    }
}

impl From<DynamicSerialImage> for DynamicImage {
    fn from(value: DynamicSerialImage) -> Self {
        match value {
            DynamicSerialImage::U8(value) => value.try_into().unwrap(),
            DynamicSerialImage::U16(value) => value.try_into().unwrap(),
            DynamicSerialImage::F32(value) => value.try_into().unwrap(),
        }
    }
}

impl From<&DynamicSerialImage> for DynamicImage {
    fn from(value: &DynamicSerialImage) -> Self {
        match value {
            DynamicSerialImage::U8(value) => value.try_into().unwrap(),
            DynamicSerialImage::U16(value) => value.try_into().unwrap(),
            DynamicSerialImage::F32(value) => value.try_into().unwrap(),
        }
    }
}

impl From<SerialImageBuffer<u8>> for DynamicSerialImage {
    fn from(value: SerialImageBuffer<u8>) -> Self {
        DynamicSerialImage::U8(value)
    }
}

impl From<SerialImageBuffer<u16>> for DynamicSerialImage {
    fn from(value: SerialImageBuffer<u16>) -> Self {
        DynamicSerialImage::U16(value)
    }
}

impl From<SerialImageBuffer<f32>> for DynamicSerialImage {
    fn from(value: SerialImageBuffer<f32>) -> Self {
        DynamicSerialImage::F32(value)
    }
}

impl From<&SerialImageBuffer<u8>> for DynamicSerialImage {
    fn from(value: &SerialImageBuffer<u8>) -> Self {
        DynamicSerialImage::U8(value.clone())
    }
}

impl From<&SerialImageBuffer<u16>> for DynamicSerialImage {
    fn from(value: &SerialImageBuffer<u16>) -> Self {
        DynamicSerialImage::U16(value.clone())
    }
}

impl From<&SerialImageBuffer<f32>> for DynamicSerialImage {
    fn from(value: &SerialImageBuffer<f32>) -> Self {
        DynamicSerialImage::F32(value.clone())
    }
}

impl TryInto<SerialImageBuffer<u8>> for DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<u8>, &'static str> {
        match self {
            DynamicSerialImage::U8(value) => Ok(value),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u8>"),
        }
    }
}

impl TryInto<SerialImageBuffer<u8>> for &DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<u8>, &'static str> {
        match self {
            DynamicSerialImage::U8(value) => Ok(value.clone()),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u16>"),
        }
    }
}

impl TryInto<SerialImageBuffer<u16>> for DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<u16>, &'static str> {
        match self {
            DynamicSerialImage::U16(value) => Ok(value),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u8>"),
        }
    }
}

impl TryInto<SerialImageBuffer<u16>> for &DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<u16>, &'static str> {
        match self {
            DynamicSerialImage::U16(value) => Ok(value.clone()),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u16>"),
        }
    }
}

impl TryInto<SerialImageBuffer<f32>> for DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<f32>, &'static str> {
        match self {
            DynamicSerialImage::F32(value) => Ok(value),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u8>"),
        }
    }
}

impl TryInto<SerialImageBuffer<f32>> for &DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<f32>, &'static str> {
        match self {
            DynamicSerialImage::F32(value) => Ok(value.clone()),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u16>"),
        }
    }
}
