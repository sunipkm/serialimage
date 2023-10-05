#![warn(missing_docs)]
#![doc = document_features::document_features!()]
#[cfg(feature = "fitsio")]
use std::{
    fs::remove_file,
    io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[cfg(feature = "fitsio")]
use fitsio::{
    errors::Error as FitsError,
    images::{ImageDescription, ImageType},
    FitsFile,
};

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

    /// Saves the buffer to a file at the path specified.
    ///
    /// The image format is derived from the file extension.
    /// `png`, `jpg`, `bmp`, `ico`, `tiff` and `exr` files are supported.
    pub fn save(&self, path: &str) -> ImageResult<()> {
        let img: DynamicImage = self.into();
        img.save(path)
    }

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
        self,
        dir_prefix: &Path,
        file_prefix: &str,
        progname: Option<&str>,
        compress: bool,
        overwrite: bool,
    ) -> Result<PathBuf, FitsError> {
        if !dir_prefix.exists() {
            return Err(FitsError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Directory {:?} does not exist", dir_prefix),
            )));
        }
        let meta = self.get_metadata();
        let meta2 = meta.clone();

        let timestamp;
        let cameraname;
        if let Some(meta) = meta {
            timestamp = meta
                .timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_millis();
            cameraname = meta.camera_name.clone();
        } else {
            timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_millis();
            cameraname = "unknown".to_owned();
        }

        let file_prefix = if file_prefix.trim().is_empty() {
            cameraname.clone()
        } else {
            file_prefix.to_owned()
        };

        let fpath = dir_prefix.join(Path::new(&format!(
            "{}_{}.fits",
            file_prefix, timestamp as u64
        )));

        if fpath.exists() {
            if !overwrite {
                return Err(FitsError::Io(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("File {:?} already exists", fpath),
                )));
            } else {
                let res = remove_file(fpath.clone());
                if let Err(msg) = res {
                    return Err(FitsError::Io(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Could not remove file {:?}: {}", fpath, msg),
                    )));
                }
            }
        }
        let width = self.width();
        let height = self.height();
        let imgsize = [height as usize, width as usize];
        let data_type = match &self {
            DynamicSerialImage::U8(_) => ImageType::UnsignedByte,
            DynamicSerialImage::U16(_) => ImageType::UnsignedShort,
            DynamicSerialImage::F32(_) => ImageType::Float,
        };

        let img_desc = ImageDescription {
            data_type,
            dimensions: &imgsize,
        };

        let path = Path::new(dir_prefix).join(Path::new(&format!(
            "{}_{}.fits{}",
            file_prefix,
            timestamp as u64,
            if compress { "[compress]" } else { "" }
        )));

        let mut fptr = FitsFile::create(path.clone()).open()?;

        let hdu = match self {
            DynamicSerialImage::U8(value) => {
                let img = value;
                let primary = if img.is_luma() { "LUMINANCE" } else { "RED" };
                let hdu = fptr.create_image(primary, &img_desc)?;
                let channels;
                if img.is_luma() {
                    hdu.write_image(&mut fptr, img.get_luma().unwrap())?;
                    hdu.write_key(&mut fptr, "CHANNELS", 1)?;
                    channels = 1;
                } else if img.is_rgb() {
                    hdu.write_image(&mut fptr, img.get_red().unwrap())?;
                    let ghdu = fptr.create_image("GREEN", &img_desc)?;
                    ghdu.write_image(&mut fptr, img.get_green().unwrap())?;
                    let bhdu = fptr.create_image("BLUE", &img_desc)?;
                    bhdu.write_image(&mut fptr, img.get_blue().unwrap())?;
                    hdu.write_key(&mut fptr, "CHANNELS", 3)?;
                    channels = 3;
                } else {
                    return Err(FitsError::Message(format!(
                        "Unsupported image type {:?}",
                        data_type
                    )));
                }
                if let Some(alpha) = img.get_alpha() {
                    let ahdu = fptr.create_image("ALPHA", &img_desc)?;
                    ahdu.write_image(&mut fptr, alpha)?;
                    hdu.write_key(&mut fptr, "CHANNELS", channels + 1)?;
                }
                hdu
            }
            DynamicSerialImage::U16(value) => {
                let img: SerialImageBuffer<u16> = value;
                let primary = if img.is_luma() { "LUMINANCE" } else { "RED" };
                let hdu = fptr.create_image(primary, &img_desc)?;
                let channels;
                if img.is_luma() {
                    hdu.write_image(&mut fptr, img.get_luma().unwrap())?;
                    hdu.write_key(&mut fptr, "CHANNELS", 1)?;
                    channels = 1;
                } else if img.is_rgb() {
                    hdu.write_image(&mut fptr, img.get_red().unwrap())?;
                    let ghdu = fptr.create_image("GREEN", &img_desc)?;
                    ghdu.write_image(&mut fptr, img.get_green().unwrap())?;
                    let bhdu = fptr.create_image("BLUE", &img_desc)?;
                    bhdu.write_image(&mut fptr, img.get_blue().unwrap())?;
                    hdu.write_key(&mut fptr, "CHANNELS", 3)?;
                    channels = 3;
                } else {
                    return Err(FitsError::Message(format!(
                        "Unsupported image type {:?}",
                        data_type
                    )));
                }
                if let Some(alpha) = img.get_alpha() {
                    let ahdu = fptr.create_image("ALPHA", &img_desc)?;
                    ahdu.write_image(&mut fptr, alpha)?;
                    hdu.write_key(&mut fptr, "CHANNELS", channels + 1)?;
                }
                hdu
            }

            DynamicSerialImage::F32(value) => {
                let img = value;
                let primary = if img.is_luma() { "LUMINANCE" } else { "RED" };
                let hdu = fptr.create_image(primary, &img_desc)?;
                let channels;
                if img.is_luma() {
                    hdu.write_image(&mut fptr, img.get_luma().unwrap())?;
                    hdu.write_key(&mut fptr, "CHANNELS", 1)?;
                    channels = 1;
                } else if img.is_rgb() {
                    hdu.write_image(&mut fptr, img.get_red().unwrap())?;
                    let ghdu = fptr.create_image("GREEN", &img_desc)?;
                    ghdu.write_image(&mut fptr, img.get_green().unwrap())?;
                    let bhdu = fptr.create_image("BLUE", &img_desc)?;
                    bhdu.write_image(&mut fptr, img.get_blue().unwrap())?;
                    hdu.write_key(&mut fptr, "CHANNELS", 3)?;
                    channels = 3;
                } else {
                    return Err(FitsError::Message(format!(
                        "Unsupported image type {:?}",
                        data_type
                    )));
                }
                if let Some(alpha) = img.get_alpha() {
                    let ahdu = fptr.create_image("ALPHA", &img_desc)?;
                    ahdu.write_image(&mut fptr, alpha)?;
                    hdu.write_key(&mut fptr, "CHANNELS", channels + 1)?;
                }
                hdu
            }
        };

        hdu.write_key(&mut fptr, "PROGRAM", progname.unwrap_or("unknown"))?;
        hdu.write_key(&mut fptr, "CAMERA", cameraname.as_str())?;
        hdu.write_key(&mut fptr, "TIMESTAMP", timestamp as u64)?;
        if let Some(meta) = meta2 {
            hdu.write_key(&mut fptr, "CCDTEMP", meta.temperature)?;
            hdu.write_key(&mut fptr, "EXPOSURE_US", meta.exposure.as_micros() as u64)?;
            hdu.write_key(&mut fptr, "ORIGIN_X", meta.img_left)?;
            hdu.write_key(&mut fptr, "ORIGIN_Y", meta.img_top)?;
            hdu.write_key(&mut fptr, "BINX", meta.bin_x)?;
            hdu.write_key(&mut fptr, "BINY", meta.bin_y)?;
            hdu.write_key(&mut fptr, "GAIN", meta.gain)?;
            hdu.write_key(&mut fptr, "OFFSET", meta.offset)?;
            hdu.write_key(&mut fptr, "GAIN_MIN", meta.min_gain)?;
            hdu.write_key(&mut fptr, "GAIN_MAX", meta.max_gain)?;
            for obj in meta.get_extended_data().iter() {
                hdu.write_key(&mut fptr, &obj.0, obj.1.as_str())?;
            }
        }

        Ok(path)
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
