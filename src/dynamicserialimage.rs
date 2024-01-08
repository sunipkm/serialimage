#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[cfg(feature = "fitsio")]
use fitsio::errors::Error as FitsError;
#[cfg(feature = "fitsio")]
use std::path::{Path, PathBuf};

use image::{ColorType, DynamicImage};
pub use image::{ImageFormat, ImageResult};
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
///  * [`DynamicSerialImage`] <- [`SerialImageBuffer<u8>`]
///  * [`DynamicSerialImage`] <- [`SerialImageBuffer<u16>`]
///  * [`DynamicSerialImage`] <- [`SerialImageBuffer<f32>`]
///
/// With [`std::convert::TryFrom`]:
///  * [`DynamicImage`] <-> [`SerialImageBuffer<u8>`]
///  * [`DynamicImage`] <-> [`SerialImageBuffer<u16>`]
///  * [`DynamicImage`] <-> [`SerialImageBuffer<f32>`]
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
    pub fn get_metadata(&self) -> Option<ImageMetaData> {
        match self {
            DynamicSerialImage::U8(value) => value.get_metadata(),
            DynamicSerialImage::U16(value) => value.get_metadata(),
            DynamicSerialImage::F32(value) => value.get_metadata(),
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

    /// Get the underlying [`SerialImageBuffer<u8>`] if the image is of type [`DynamicSerialImage::U8`].
    pub fn as_u8(&self) -> Option<&SerialImageBuffer<u8>> {
        match self {
            DynamicSerialImage::U8(value) => Some(value),
            _ => None,
        }
    }

    /// Get the underlying [`SerialImageBuffer<u16>`] if the image is of type [`DynamicSerialImage::U16`].
    pub fn as_u16(&self) -> Option<&SerialImageBuffer<u16>> {
        match self {
            DynamicSerialImage::U16(value) => Some(value),
            _ => None,
        }
    }

    /// Get the underlying [`SerialImageBuffer<f32>`] if the image is of type [`DynamicSerialImage::F32`].
    pub fn as_f32(&self) -> Option<&SerialImageBuffer<f32>> {
        match self {
            DynamicSerialImage::F32(value) => Some(value),
            _ => None,
        }
    }

    /// Convert the image to grayscale. The transformation used is `0.2162 * red + 0.7152 * green + 0.0722 * blue` for converting RGB to grayscale (see [here](https://stackoverflow.com/a/56678483)).
    pub fn into_luma(&self) -> SerialImageBuffer<u16> {
        match self {
            DynamicSerialImage::U8(value) => value.into_luma(),
            DynamicSerialImage::U16(value) => value.into_luma(),
            DynamicSerialImage::F32(value) => value.into_luma(),
        }
    }

    /// Convert the image to grayscale with alpha channel. The transformation used is `0.2162 * red + 0.7152 * green + 0.0722 * blue` for converting RGB to grayscale (see [here](https://stackoverflow.com/a/56678483)).
    pub fn into_luma_alpha(&self) -> SerialImageBuffer<u16> {
        match self {
            DynamicSerialImage::U8(value) => value.into_luma_alpha(),
            DynamicSerialImage::U16(value) => value.into_luma_alpha(),
            DynamicSerialImage::F32(value) => value.into_luma_alpha(),
        }
    }

    /// Saves the buffer to a file at the path specified.
    ///
    /// The image format is derived from the file extension.
    /// `png`, `jpg`, `bmp`, `ico`, `tiff` and `exr` files are supported.
    pub fn save(&self, path: &str) -> ImageResult<()> {
        let img: DynamicImage = self.into();
        img.save(path)
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "fitsio")))]
    #[cfg(feature = "fitsio")]
    /// Save the image data to a FITS file.
    ///
    /// # Arguments
    ///  * `dir_prefix` - The directory where the file will be saved.
    ///  * `file_prefix` - The prefix of the file name. The file name will be of the form `{file_prefix}_{timestamp}.fits`.
    ///  * `progname` - The name of the program that generated the image.
    ///  * `compress` - Whether to compress the FITS file.
    ///  * `overwrite` - Whether to overwrite the file if it already exists.
    ///
    /// # Errors
    ///  * [`fitsio::errors::Error`] with the error description.
    pub fn savefits(
        &self,
        dir_prefix: &Path,
        file_prefix: &str,
        progname: Option<&str>,
        compress: bool,
        overwrite: bool,
    ) -> Result<PathBuf, FitsError> {
        match self {
            DynamicSerialImage::U8(value) => {
                value.savefits(dir_prefix, file_prefix, progname, compress, overwrite)
            }
            DynamicSerialImage::U16(value) => {
                value.savefits(dir_prefix, file_prefix, progname, compress, overwrite)
            }
            DynamicSerialImage::F32(value) => {
                value.savefits(dir_prefix, file_prefix, progname, compress, overwrite)
            }
        }
    }
}

impl DynamicSerialImage {
    /// Create a new image from a vector of [`u8`] pixels.
    ///
    /// # Arguments
    ///  * `width` - The width of the image.
    ///  * `height` - The height of the image.
    ///  * `data` - The image data as a vector of [`u8`] pixels.
    ///
    /// # Errors
    ///  - Error messages as strings.
    ///
    /// Note: The length of the vector must be `width * height * channels`.
    ///  - For grayscale images, `channels` is 1.
    ///  - For grayscale images with alpha channel, `channels` is 2.
    ///  - For RGB images, `channels` is 3.
    ///  - For RGBA images, `channels` is 4.
    pub fn from_vec_u8(width: usize, height: usize, data: Vec<u8>) -> Result<Self, &'static str> {
        Ok(DynamicSerialImage::U8(SerialImageBuffer::from_vec(
            width, height, data,
        )?))
    }

    /// Create a new image from a vector of [`u16`] pixels.
    ///
    /// # Arguments
    ///  * `width` - The width of the image.
    ///  * `height` - The height of the image.
    ///  * `data` - The image data as a vector of [`u16`] pixels.
    ///
    /// # Errors
    ///  - Error messages as strings.
    ///
    /// Note: The length of the vector must be `width * height * channels`.
    ///  - For grayscale images, `channels` is 1.
    ///  - For grayscale images with alpha channel, `channels` is 2.
    ///  - For RGB images, `channels` is 3.
    ///  - For RGBA images, `channels` is 4.
    pub fn from_vec_u16(width: usize, height: usize, data: Vec<u16>) -> Result<Self, &'static str> {
        Ok(DynamicSerialImage::U16(SerialImageBuffer::from_vec(
            width, height, data,
        )?))
    }

    /// Create a new image from a vector of [`f32`] pixels.
    ///
    /// # Arguments
    /// * `width` - The width of the image.
    /// * `height` - The height of the image.
    /// * `data` - The image data as a vector of [`f32`] pixels.
    ///
    /// # Errors
    ///  - Error messages as strings.
    ///
    /// Note: The length of the vector must be `width * height * channels`. Grayscale images are not supported.
    ///  - For RGB images, `channels` is 3.
    ///  - For RGBA images, `channels` is 4.
    pub fn from_vec_f32(width: usize, height: usize, data: Vec<f32>) -> Result<Self, &'static str> {
        Ok(DynamicSerialImage::F32(SerialImageBuffer::from_vec(
            width, height, data,
        )?))
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
            _ => Err("Could not convert DynamicSerialImage to SerialImageBuffer<u8>"),
        }
    }
}

impl TryInto<SerialImageBuffer<u8>> for &DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<u8>, &'static str> {
        match self {
            DynamicSerialImage::U8(value) => Ok(value.clone()),
            _ => Err("Could not convert DynamicSerialImage to SerialImageBuffer<u16>"),
        }
    }
}

impl TryInto<SerialImageBuffer<u16>> for DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<u16>, &'static str> {
        match self {
            DynamicSerialImage::U16(value) => Ok(value),
            _ => Err("Could not convert DynamicSerialImage to SerialImageBuffer<u8>"),
        }
    }
}

impl TryInto<SerialImageBuffer<u16>> for &DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<u16>, &'static str> {
        match self {
            DynamicSerialImage::U16(value) => Ok(value.clone()),
            _ => Err("Could not convert DynamicSerialImage to SerialImageBuffer<u16>"),
        }
    }
}

impl TryInto<SerialImageBuffer<f32>> for DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<f32>, &'static str> {
        match self {
            DynamicSerialImage::F32(value) => Ok(value),
            _ => Err("Could not convert DynamicSerialImage to SerialImageBuffer<u8>"),
        }
    }
}

impl TryInto<SerialImageBuffer<f32>> for &DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageBuffer<f32>, &'static str> {
        match self {
            DynamicSerialImage::F32(value) => Ok(value.clone()),
            _ => Err("Could not convert DynamicSerialImage to SerialImageBuffer<u16>"),
        }
    }
}
