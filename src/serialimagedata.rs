#![deny(missing_docs)]
use std::{
    fmt::Display,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use image::{ColorType, DynamicImage, ImageBuffer};
use serde::{Deserialize, Serialize};
/// Valid types for the serial image data structure: [`u8`], [`u16`], [`f32`].
pub trait SerialImageStorageTypes {}

impl SerialImageStorageTypes for u8 {}
impl SerialImageStorageTypes for u16 {}
impl SerialImageStorageTypes for f32 {}

/// Serial image type enumeration. The enumeration variants are [`SerialImagePixel::U8`], [`SerialImagePixel::U16`], [`SerialImagePixel::F32`].
/// The variants contain the number of elements per pixel.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum SerialImagePixel {
    /// Pixel elements are [`u8`].
    U8(usize),
    /// Pixel elements are [`u16`].
    U16(usize),
    /// Pixel elements are [`f32`].
    F32(usize),
}

impl TryFrom<ColorType> for SerialImagePixel {
    type Error = &'static str;
    fn try_from(value: ColorType) -> Result<SerialImagePixel, &'static str> {
        match value {
            ColorType::L8 => Ok(SerialImagePixel::U8(1)),
            ColorType::L16 => Ok(SerialImagePixel::U16(1)),
            ColorType::Rgb8 => Ok(SerialImagePixel::U8(3)),
            ColorType::Rgb16 => Ok(SerialImagePixel::U16(3)),
            ColorType::Rgba8 => Ok(SerialImagePixel::U8(4)),
            ColorType::Rgba16 => Ok(SerialImagePixel::U16(4)),
            ColorType::La8 => Ok(SerialImagePixel::U8(2)),
            ColorType::La16 => Ok(SerialImagePixel::U16(2)),
            ColorType::Rgb32F => Ok(SerialImagePixel::F32(3)),
            ColorType::Rgba32F => Ok(SerialImagePixel::F32(4)),
            _ => Err("Unsupported image type"),
        }
    }
}

impl TryInto<ColorType> for SerialImagePixel {
    type Error = &'static str;
    fn try_into(self) -> Result<ColorType, &'static str> {
        match self {
            SerialImagePixel::U8(value) => {
                if value == 1 {
                    Ok(ColorType::L8)
                } else if value == 3 {
                    Ok(ColorType::Rgb8)
                } else if value == 4 {
                    Ok(ColorType::Rgba8)
                } else if value == 2 {
                    Ok(ColorType::La8)
                } else {
                    Err("Unsupported image type")
                }
            }
            SerialImagePixel::U16(value) => {
                if value == 1 {
                    Ok(ColorType::L16)
                } else if value == 3 {
                    Ok(ColorType::Rgb16)
                } else if value == 4 {
                    Ok(ColorType::Rgba16)
                } else if value == 2 {
                    Ok(ColorType::La16)
                } else {
                    Err("Unsupported image type")
                }
            }
            SerialImagePixel::F32(value) => {
                if value == 3 {
                    Ok(ColorType::Rgb32F)
                } else if value == 4 {
                    Ok(ColorType::Rgba32F)
                } else {
                    Err("Unsupported image type")
                }
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
/// Serializable Image Data Structure.
///
/// This structure is derived from the [`DynamicImage`] structure and is used to serialize the image data.
/// This structure implements the [`std::clone::Clone`] trait, as well as the [`std::convert::TryFrom`] and [`std::convert::TryInto`] traits.
pub struct SerialImageData<T: SerialImageStorageTypes> {
    meta: ImageMetaData,
    imgdata: Vec<T>,
    width: usize,
    height: usize,
    pixel: SerialImagePixel,
}

impl<T: SerialImageStorageTypes> SerialImageData<T> {
    /// Create a new serial image data structure.
    ///
    /// # Arguments
    ///  * `meta` - Image metadata.
    ///  * `imgdata` - Raw image data, which is a vector of [`u8`], [`u16`] or [`f32`] values.
    ///  * `width` - Width of the image.
    ///  * `height` - Height of the image.
    ///  * `pixel` - Pixel type of the image. The pixel type is of [`SerialImagePixel`].
    ///
    /// # Returns
    ///  * `Some(SerialImageData)` - If the image data is valid, i.e. the number of elements in the raw image data vector is equal to the width x height x number of elements per pixel, then the function returns a [`Some`] variant containing the serial image data structure.
    pub fn new(
        meta: ImageMetaData,
        imgdata: Vec<T>,
        width: usize,
        height: usize,
        pixel: SerialImagePixel,
    ) -> Option<Self> {
        let elem = match pixel {
            SerialImagePixel::U8(value) => value,
            SerialImagePixel::U16(value) => value,
            SerialImagePixel::F32(value) => value,
        };
        if elem * width * height != imgdata.len() {
            return None;
        }
        Some(Self {
            meta,
            imgdata,
            width,
            height,
            pixel,
        })
    }

    /// Get the image metadata.
    pub fn get_metadata(&self) -> &ImageMetaData {
        &self.meta
    }

    /// Get a mutable reference to the image metadata.
    pub fn get_mut_metadata(&mut self) -> &mut ImageMetaData {
        &mut self.meta
    }

    /// Update the image metadata.
    pub fn set_metadata(&mut self, meta: ImageMetaData) {
        self.meta = meta;
    }

    /// Get the underlying raw image data.
    pub fn get_data(&self) -> &Vec<T> {
        &self.imgdata
    }

    /// Get a mutable reference to the underlying raw image data.
    pub fn get_mut_data(&mut self) -> &mut Vec<T> {
        &mut self.imgdata
    }

    /// Get the width of the image.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the image.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the pixel type of the image. The pixel type is of [`SerialImagePixel`].
    pub fn pixel(&self) -> SerialImagePixel {
        self.pixel
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[deny(missing_docs)]
/// Image metadata structure.
/// This structure implements the [`std::fmt::Display`] and [`std::clone::Clone`] traits.
pub struct ImageMetaData {
    /// Binning in X direction
    pub bin_x: u32,
    /// Binning in Y direction
    pub bin_y: u32,
    /// Top of image (pixels, binned coordinates)
    pub img_top: u32,
    /// Left of image (pixels, binned coordinates)
    pub img_left: u32,
    /// Camera temperature (C)
    pub temperature: f32,
    /// Exposure time
    pub exposure: Duration,
    /// Timestamp of the image
    pub timestamp: SystemTime,
    /// Name of the camera
    pub camera_name: String,
    /// Gain (raw)
    pub gain: i64,
    /// Offset (raw)
    pub offset: i64,
    /// Minimum gain (raw)
    pub min_gain: i32,
    /// Maximum gain (raw)
    pub max_gain: i32,
    extended_metadata: Vec<(String, String)>,
}

impl ImageMetaData {
    /// Create a new image metadata structure.
    pub fn new(
        timestamp: SystemTime,
        exposure: Duration,
        temperature: f32,
        bin_x: u32,
        bin_y: u32,
        camera_name: &str,
        gain: i64,
        offset: i64,
    ) -> Self {
        Self {
            bin_x,
            bin_y,
            img_top: 0,
            img_left: 0,
            temperature,
            exposure,
            timestamp,
            camera_name: camera_name.to_string(),
            gain,
            offset,
            ..Default::default()
        }
    }

    /// Create a new image metadata structure with full parameters.
    pub fn full_builder(
        bin_x: u32,
        bin_y: u32,
        img_top: u32,
        img_left: u32,
        temperature: f32,
        exposure: Duration,
        timestamp: SystemTime,
        camera_name: &str,
        gain: i64,
        offset: i64,
        min_gain: i32,
        max_gain: i32,
    ) -> Self {
        Self {
            bin_x,
            bin_y,
            img_top,
            img_left,
            temperature,
            exposure,
            timestamp,
            camera_name: camera_name.to_string(),
            gain,
            offset,
            min_gain,
            max_gain,
            ..Default::default()
        }
    }
}

impl Default for ImageMetaData {
    fn default() -> Self {
        Self {
            bin_x: 1,
            bin_y: 1,
            img_top: 0,
            img_left: 0,
            temperature: 0f32,
            exposure: Duration::from_secs(0),
            timestamp: UNIX_EPOCH,
            camera_name: String::new(),
            gain: 0,
            offset: 0,
            min_gain: 0,
            max_gain: 0,
            extended_metadata: Vec::new(),
        }
    }
}

impl Display for ImageMetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ImageMetaData [{:#?}]:\n
            \tCamera name: {}\n
            \tImage Bin: {} x {}\n
            \tImage Origin: {} x {}
            \tExposure: {} s\n
            \tGain: {}, Offset: {}\n
            \tTemperature: {} C\n",
            self.timestamp,
            self.camera_name,
            self.bin_x,
            self.bin_y,
            self.img_left,
            self.img_top,
            self.exposure.as_secs(),
            self.gain,
            self.offset,
            self.temperature
        )?;
        if self.extended_metadata.len() > 0 {
            write!(f, "\tExtended Metadata:\n")?;
            for obj in self.extended_metadata.iter() {
                write!(f, "\t\t{}: {}\n", obj.0, obj.1)?;
            }
        };
        Ok(())
    }
}

impl ImageMetaData {
    /// Add an extended attribute to the image metadata using `vec::push()`.
    ///
    /// # Panics
    ///
    /// If the new capacity exceeds `isize::MAX` bytes.
    pub fn add_extended_attrib(&mut self, key: &str, val: &str) {
        self.extended_metadata
            .push((key.to_string(), val.to_string()));
    }

    /// Get the extended attributes of the image metadata.
    pub fn get_extended_data(&self) -> &Vec<(String, String)> {
        &self.extended_metadata
    }
}

impl TryFrom<DynamicImage> for SerialImageData<u8> {
    type Error = &'static str;
    fn try_from(value: DynamicImage) -> Result<SerialImageData<u8>, &'static str> {
        let img = value.clone();
        let meta = ImageMetaData::default();
        let color = img.color();
        let width = img.width();
        let height = img.height();
        let pixel: Result<SerialImagePixel, &'static str> = color.try_into();
        let pixel = match pixel {
            Ok(p) => p,
            Err(msg) => {
                return Err(msg);
            }
        };
        let imgdata = match color {
            ColorType::L8 => {
                let img = img.into_luma8();
                img.into_raw()
            }
            ColorType::Rgb8 => {
                let img = img.into_rgb8();
                img.into_raw()
            }
            ColorType::Rgba8 => {
                let img = img.into_rgba8();
                img.into_raw()
            }
            ColorType::La8 => {
                let img = img.into_luma_alpha8();
                img.into_raw()
            }
            _ => {
                return Err("Unsupported image type");
            }
        };
        Ok(
            SerialImageData::new(meta, imgdata, width as usize, height as usize, pixel)
                .ok_or("Could not create image L8 image")?,
        )
    }
}

impl TryFrom<&DynamicImage> for SerialImageData<u8> {
    type Error = &'static str;
    fn try_from(value: &DynamicImage) -> Result<SerialImageData<u8>, &'static str> {
        let img = value.clone();
        let meta = ImageMetaData::default();
        let color = img.color();
        let width = img.width();
        let height = img.height();
        let pixel = color.try_into()?;
        let imgdata = match color {
            ColorType::L8 => {
                let img = img.into_luma8();
                img.into_raw()
            }
            ColorType::Rgb8 => {
                let img = img.into_rgb8();
                img.into_raw()
            }
            ColorType::Rgba8 => {
                let img = img.into_rgba8();
                img.into_raw()
            }
            ColorType::La8 => {
                let img = img.into_luma_alpha8();
                img.into_raw()
            }
            _ => {
                return Err("Unsupported image type");
            }
        };
        Ok(
            SerialImageData::new(meta, imgdata, width as usize, height as usize, pixel)
                .ok_or("Could not create image L8 image")?,
        )
    }
}

impl TryFrom<DynamicImage> for SerialImageData<u16> {
    type Error = &'static str;
    fn try_from(value: DynamicImage) -> Result<SerialImageData<u16>, &'static str> {
        let img = value.clone();
        let meta = ImageMetaData::default();
        let color = img.color();
        let width = img.width();
        let height = img.height();
        let pixel = color.try_into()?;
        let imgdata = match color {
            ColorType::L16 => {
                let img = img.into_luma16();
                img.into_raw()
            }
            ColorType::Rgb16 => {
                let img = img.into_rgb16();
                img.into_raw()
            }
            ColorType::Rgba16 => {
                let img = img.into_rgba16();
                img.into_raw()
            }
            ColorType::La16 => {
                let img = img.into_luma_alpha16();
                img.into_raw()
            }
            _ => {
                return Err("Unsupported image type");
            }
        };
        Ok(
            SerialImageData::new(meta, imgdata, width as usize, height as usize, pixel)
                .ok_or("Could not create image L16 image")?,
        )
    }
}

impl TryFrom<&DynamicImage> for SerialImageData<u16> {
    type Error = &'static str;
    fn try_from(value: &DynamicImage) -> Result<SerialImageData<u16>, &'static str> {
        let img = value.clone();
        let meta = ImageMetaData::default();
        let color = img.color();
        let width = img.width();
        let height = img.height();
        let pixel = color.try_into()?;
        let imgdata = match color {
            ColorType::L16 => {
                let img = img.into_luma16();
                img.into_raw()
            }
            ColorType::Rgb16 => {
                let img = img.into_rgb16();
                img.into_raw()
            }
            ColorType::Rgba16 => {
                let img = img.into_rgba16();
                img.into_raw()
            }
            ColorType::La16 => {
                let img = img.into_luma_alpha16();
                img.into_raw()
            }
            _ => {
                return Err("Unsupported image type");
            }
        };
        Ok(
            SerialImageData::new(meta, imgdata, width as usize, height as usize, pixel)
                .ok_or("Could not create image L16 image")?,
        )
    }
}

impl TryFrom<DynamicImage> for SerialImageData<f32> {
    type Error = &'static str;
    fn try_from(value: DynamicImage) -> Result<SerialImageData<f32>, &'static str> {
        let img = value.clone();
        let meta = ImageMetaData::default();
        let color = img.color();
        let width = img.width();
        let height = img.height();
        let pixel = color.try_into()?;
        let imgdata = match color {
            ColorType::Rgb32F => {
                let img = img.into_rgb32f();
                img.into_raw()
            }
            ColorType::Rgba32F => {
                let img = img.into_rgba32f();
                img.into_raw()
            }
            _ => {
                return Err("Unsupported image type");
            }
        };
        Ok(
            SerialImageData::new(meta, imgdata, width as usize, height as usize, pixel)
                .ok_or("Could not create image F32 image")?,
        )
    }
}

impl TryFrom<&DynamicImage> for SerialImageData<f32> {
    type Error = &'static str;
    fn try_from(value: &DynamicImage) -> Result<SerialImageData<f32>, &'static str> {
        let img = value.clone();
        let meta = ImageMetaData::default();
        let color = img.color();
        let width = img.width();
        let height = img.height();
        let pixel = color.try_into()?;
        let imgdata = match color {
            ColorType::Rgb32F => {
                let img = img.into_rgb32f();
                img.into_raw()
            }
            ColorType::Rgba32F => {
                let img = img.into_rgba32f();
                img.into_raw()
            }
            _ => {
                return Err("Unsupported image type");
            }
        };
        Ok(
            SerialImageData::new(meta, imgdata, width as usize, height as usize, pixel)
                .ok_or("Could not create image F32 image")?,
        )
    }
}

impl TryFrom<SerialImageData<u8>> for DynamicImage {
    type Error = &'static str;
    fn try_from(value: SerialImageData<u8>) -> Result<DynamicImage, &'static str> {
        let imgdata = value.get_data().clone();
        let width = value.width();
        let height = value.height();
        let color = value.pixel().try_into()?;

        let img = match color {
            ColorType::L8 => {
                let img = image::GrayImage::from_vec(width as u32, height as u32, imgdata)
                    .ok_or("Could not create image L8 image")?;
                DynamicImage::ImageLuma8(img)
            }
            ColorType::Rgb8 => {
                let img = image::RgbImage::from_vec(width as u32, height as u32, imgdata)
                    .ok_or("Could not create image Rgb8 image")?;
                DynamicImage::ImageRgb8(img)
            }
            ColorType::Rgba8 => {
                let img = image::RgbaImage::from_vec(width as u32, height as u32, imgdata)
                    .ok_or("Could not create image Rgba8 image")?;
                DynamicImage::ImageRgba8(img)
            }
            ColorType::La8 => {
                let img = image::GrayAlphaImage::from_vec(width as u32, height as u32, imgdata)
                    .ok_or("Could not create image La8 image")?;
                DynamicImage::ImageLumaA8(img)
            }
            _ => {
                return Err("Unsupported image type");
            }
        };
        Ok(img)
    }
}

impl TryFrom<&SerialImageData<u8>> for DynamicImage {
    type Error = &'static str;
    fn try_from(value: &SerialImageData<u8>) -> Result<DynamicImage, &'static str> {
        let imgdata = value.get_data().clone();
        let width = value.width();
        let height = value.height();
        let color = value.pixel().try_into()?;

        let img = match color {
            ColorType::L8 => {
                let img = image::GrayImage::from_vec(width as u32, height as u32, imgdata)
                    .ok_or("Could not create image L8 image")?;
                DynamicImage::ImageLuma8(img)
            }
            ColorType::Rgb8 => {
                let img = image::RgbImage::from_vec(width as u32, height as u32, imgdata)
                    .ok_or("Could not create image Rgb8 image")?;
                DynamicImage::ImageRgb8(img)
            }
            ColorType::Rgba8 => {
                let img = image::RgbaImage::from_vec(width as u32, height as u32, imgdata)
                    .ok_or("Could not create image Rgba8 image")?;
                DynamicImage::ImageRgba8(img)
            }
            ColorType::La8 => {
                let img = image::GrayAlphaImage::from_vec(width as u32, height as u32, imgdata)
                    .ok_or("Could not create image La8 image")?;
                DynamicImage::ImageLumaA8(img)
            }
            _ => {
                return Err("Unsupported image type");
            }
        };
        Ok(img)
    }
}

impl TryFrom<SerialImageData<u16>> for DynamicImage {
    type Error = &'static str;
    fn try_from(value: SerialImageData<u16>) -> Result<DynamicImage, &'static str> {
        let imgdata = value.get_data();
        let width = value.width();
        let height = value.height();
        let color = value.pixel().try_into()?;

        let img =
            match color {
                ColorType::L16 => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Luma<u16>, Vec<u16>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_luma16()
                        .ok_or("Could not create image L16 image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                ColorType::Rgb16 => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Rgb<u16>, Vec<u16>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_rgb16()
                        .ok_or("Could not create image L16 image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                ColorType::Rgba16 => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Rgba<u16>, Vec<u16>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_rgba16()
                        .ok_or("Could not create image L16 image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                ColorType::La16 => {
                    let mut img =
                        DynamicImage::from(ImageBuffer::<image::LumaA<u16>, Vec<u16>>::new(
                            width as u32,
                            height as u32,
                        ));
                    let imgbuf = img
                        .as_mut_luma_alpha16()
                        .ok_or("Could not create image L16 image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                _ => {
                    return Err("Unsupported image type");
                }
            };
        Ok(img)
    }
}

impl TryFrom<&SerialImageData<u16>> for DynamicImage {
    type Error = &'static str;
    fn try_from(value: &SerialImageData<u16>) -> Result<DynamicImage, &'static str> {
        let imgdata = value.get_data().clone();
        let width = value.width();
        let height = value.height();
        let color = value.pixel().try_into()?;

        let img =
            match color {
                ColorType::L16 => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Luma<u16>, Vec<u16>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_luma16()
                        .ok_or("Could not create image L16 image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                ColorType::Rgb16 => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Rgb<u16>, Vec<u16>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_rgb16()
                        .ok_or("Could not create image L16 image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                ColorType::Rgba16 => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Rgba<u16>, Vec<u16>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_rgba16()
                        .ok_or("Could not create image L16 image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                ColorType::La16 => {
                    let mut img =
                        DynamicImage::from(ImageBuffer::<image::LumaA<u16>, Vec<u16>>::new(
                            width as u32,
                            height as u32,
                        ));
                    let imgbuf = img
                        .as_mut_luma_alpha16()
                        .ok_or("Could not create image L16 image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                _ => {
                    return Err("Unsupported image type");
                }
            };
        Ok(img)
    }
}

impl TryFrom<SerialImageData<f32>> for DynamicImage {
    type Error = &'static str;
    fn try_from(value: SerialImageData<f32>) -> Result<DynamicImage, &'static str> {
        let imgdata = value.get_data();
        let width = value.width();
        let height = value.height();
        let color = value.pixel().try_into()?;

        let img =
            match color {
                ColorType::Rgb32F => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Rgb<f32>, Vec<f32>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_rgb32f()
                        .ok_or("Could not create image Rgb32F image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                ColorType::Rgba32F => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_rgba32f()
                        .ok_or("Could not create image Rgba32F image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                _ => {
                    return Err("Unsupported image type");
                }
            };
        Ok(img)
    }
}

impl TryFrom<&SerialImageData<f32>> for DynamicImage {
    type Error = &'static str;
    fn try_from(value: &SerialImageData<f32>) -> Result<DynamicImage, &'static str> {
        let imgdata = value.get_data().clone();
        let width = value.width();
        let height = value.height();
        let color = value.pixel().try_into()?;

        let img =
            match color {
                ColorType::Rgb32F => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Rgb<f32>, Vec<f32>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_rgb32f()
                        .ok_or("Could not create image Rgb32F image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                ColorType::Rgba32F => {
                    let mut img = DynamicImage::from(
                        ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(width as u32, height as u32),
                    );
                    let imgbuf = img
                        .as_mut_rgba32f()
                        .ok_or("Could not create image Rgba32F image")?;
                    imgbuf.copy_from_slice(&imgdata);
                    img
                }
                _ => {
                    return Err("Unsupported image type");
                }
            };
        Ok(img)
    }
}

/// Dynamic serial image enumeration. This data type encapsulates the specific serial image data types.
/// 
/// The enumeration variants are [`DynamicSerialImage::U8`], [`DynamicSerialImage::U16`], [`DynamicSerialImage::F32`].
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum DynamicSerialImage {
    /// 8-bit unsigned integer image data.
    U8(SerialImageData<u8>),
    /// 16-bit unsigned integer image data.
    U16(SerialImageData<u16>),
    /// 32-bit floating point image data.
    F32(SerialImageData<f32>),
}

impl DynamicSerialImage {
    /// Get the image metadata.
    pub fn get_metadata(&self) -> &ImageMetaData {
        match self {
            DynamicSerialImage::U8(value) => value.get_metadata(),
            DynamicSerialImage::U16(value) => value.get_metadata(),
            DynamicSerialImage::F32(value) => value.get_metadata(),
        }
    }

    /// Get a mutable reference to the image metadata.
    pub fn get_mut_metadata(&mut self) -> &mut ImageMetaData {
        match self {
            DynamicSerialImage::U8(value) => value.get_mut_metadata(),
            DynamicSerialImage::U16(value) => value.get_mut_metadata(),
            DynamicSerialImage::F32(value) => value.get_mut_metadata(),
        }
    }

    /// Update the image metadata.
    pub fn get_dynamic_image(self) -> DynamicImage {
        match self {
            DynamicSerialImage::U8(value) => value.try_into().unwrap(),
            DynamicSerialImage::U16(value) => value.try_into().unwrap(),
            DynamicSerialImage::F32(value) => value.try_into().unwrap(),
        }
    }
}

impl From<SerialImageData<u8>> for DynamicSerialImage {
    fn from(value: SerialImageData<u8>) -> Self {
        DynamicSerialImage::U8(value)
    }
}

impl From<SerialImageData<u16>> for DynamicSerialImage {
    fn from(value: SerialImageData<u16>) -> Self {
        DynamicSerialImage::U16(value)
    }
}

impl From<SerialImageData<f32>> for DynamicSerialImage {
    fn from(value: SerialImageData<f32>) -> Self {
        DynamicSerialImage::F32(value)
    }
}

impl From<&SerialImageData<u8>> for DynamicSerialImage {
    fn from(value: &SerialImageData<u8>) -> Self {
        DynamicSerialImage::U8(value.clone())
    }
}

impl From<&SerialImageData<u16>> for DynamicSerialImage {
    fn from(value: &SerialImageData<u16>) -> Self {
        DynamicSerialImage::U16(value.clone())
    }
}

impl From<&SerialImageData<f32>> for DynamicSerialImage {
    fn from(value: &SerialImageData<f32>) -> Self {
        DynamicSerialImage::F32(value.clone())
    }
}

impl TryInto<SerialImageData<u8>> for DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageData<u8>, &'static str> {
        match self {
            DynamicSerialImage::U8(value) => Ok(value),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u8>"),
        }
    }
}

impl TryInto<SerialImageData<u8>> for &DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageData<u8>, &'static str> {
        match self {
            DynamicSerialImage::U8(value) => Ok(value.clone()),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u16>"),
        }
    }
}

impl TryInto<SerialImageData<u16>> for DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageData<u16>, &'static str> {
        match self {
            DynamicSerialImage::U16(value) => Ok(value),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u8>"),
        }
    }
}

impl TryInto<SerialImageData<u16>> for &DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageData<u16>, &'static str> {
        match self {
            DynamicSerialImage::U16(value) => Ok(value.clone()),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u16>"),
        }
    }
}

impl TryInto<SerialImageData<f32>> for DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageData<f32>, &'static str> {
        match self {
            DynamicSerialImage::F32(value) => Ok(value),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u8>"),
        }
    }
}

impl TryInto<SerialImageData<f32>> for &DynamicSerialImage {
    type Error = &'static str;
    fn try_into(self) -> Result<SerialImageData<f32>, &'static str> {
        match self {
            DynamicSerialImage::F32(value) => Ok(value.clone()),
            _ => Err("Could not convert DynamicSerialImage to SerialImageData<u16>"),
        }
    }
}

