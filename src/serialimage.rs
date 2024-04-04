#![warn(missing_docs)]

use image::{imageops::FilterType, DynamicImage, ImageBuffer, Luma, LumaA, Rgb};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[cfg(feature = "fitsio")]
use fitsio::{
    errors::Error as FitsError,
    images::{ImageDescription, ImageType, WriteImage},
    FitsFile,
};
#[cfg(feature = "fitsio")]
use std::{
    fs::remove_file,
    io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub use image::Primitive;

use super::ImageMetaData;

/// Optional vector type alias.
pub type OptionVec<T> = Option<Vec<T>>;
/// Optional vector tuple type alias.
pub type TupleOptionVec<T> = (
    Option<Vec<T>>,
    Option<Vec<T>>,
    Option<Vec<T>>,
    Option<Vec<T>>,
    Option<Vec<T>>,
);

/// Valid types for the serial image data structure: [`u8`], [`u16`], [`f32`].

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct SerialImageInternal<T: Primitive> {
    luma: OptionVec<T>,
    red: OptionVec<T>,
    green: OptionVec<T>,
    blue: OptionVec<T>,
    alpha: OptionVec<T>,
    pixel_elems: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
/// A serializable image data container for [`u8`], [`u16`] and [`f32`] pixel types.
///
/// Image data is organized in channels. For example, a grayscale image stores data in the luma channel, while a color image stores data in the red, green and blue channels. Transparency is stored in the alpha channel.
pub struct SerialImageBuffer<T: Primitive> {
    meta: Option<ImageMetaData>,
    data: SerialImageInternal<T>,
    width: usize,
    height: usize,
}

impl<T: Primitive> SerialImageBuffer<T> {
    /// Create a new serializable image buffer from vector data.
    ///
    /// # Arguments
    ///  - `width`: Image width.
    ///  - `height`: Image height.
    ///  - `data`: Image data.
    ///
    /// Note:
    ///  - If `width * height == data.len()`, the image is assumed to be a grayscale image.
    ///  - If `width * height * 2 == data.len()`, the image is assumed to be a grayscale image with alpha channel, with the odd pixels being the luma channel and the even pixels being the alpha channel.
    ///  - If `width * height * 3 == data.len()`, the image is assumed to be a color image, with the first pixel in the red channel, the second pixel in the green channel, and the third pixel in the blue channel and so on.
    ///  - If `width * height * 4 == data.len()`, the image is assumed to be a color image with alpha channel, with the first pixel in the red channel, the second pixel in the green channel, the third pixel in the blue channel and the fourth pixel in the alpha channel and so on.
    ///
    ///
    /// # Errors
    ///  - If `width * height == 0`.
    ///  - If number of pixel elements is not in `[1..=4]`.
    ///  - If the length of the channel data stored in the image is not equal to `width * height * pixel elements`. Number of pixel elements are inferred using the length of the data vector.
    ///
    pub fn from_vec(width: usize, height: usize, data: Vec<T>) -> Result<Self, &'static str> {
        if width * height == 0 {
            return Err("Width and height must be greater than zero");
        }
        let pixel_elems = data.len() / (width * height);
        if data.len() != width * height * pixel_elems {
            return Err("Data length must be equal to width * height * pixel elements");
        }
        if pixel_elems > 4 || pixel_elems == 0 {
            return Err("Invalid number of pixel elements");
        }

        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems as u8);

        Ok(Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems: pixel_elems as u8,
            },
            width,
            height,
        })
    }

    fn from_vec_unsafe(size: usize, data: Vec<T>, elems: u8) -> TupleOptionVec<T> {
        if elems == 1 {
            (Some(data), None, None, None, None)
        } else if elems == 2 {
            let mut luma = Vec::with_capacity(size);
            let mut alpha = Vec::with_capacity(size);
            for i in 0..size {
                luma.push(data[i * 2]);
                alpha.push(data[i * 2 + 1]);
            }
            return (Some(luma), None, None, None, Some(alpha));
        } else if elems == 3 {
            let mut red = Vec::with_capacity(size);
            let mut green = Vec::with_capacity(size);
            let mut blue = Vec::with_capacity(size);
            for i in 0..size {
                red.push(data[i * 3]);
                green.push(data[i * 3 + 1]);
                blue.push(data[i * 3 + 2]);
            }
            return (None, Some(red), Some(green), Some(blue), None);
        } else if elems == 4 {
            let mut red = Vec::with_capacity(size);
            let mut green = Vec::with_capacity(size);
            let mut blue = Vec::with_capacity(size);
            let mut alpha = Vec::with_capacity(size);
            for i in 0..size {
                red.push(data[i * 4]);
                green.push(data[i * 4 + 1]);
                blue.push(data[i * 4 + 2]);
                alpha.push(data[i * 4 + 3]);
            }
            return (None, Some(red), Some(green), Some(blue), Some(alpha));
        } else {
            panic!("Invalid number of elements");
        }
    }

    /// Get the image metadata.
    pub fn get_metadata(&self) -> Option<ImageMetaData> {
        self.meta.clone()
    }

    /// Update the image metadata.
    ///
    /// # Arguments
    ///  - `meta`: Image metadata.
    pub fn set_metadata(&mut self, meta: Option<ImageMetaData>) {
        self.meta = meta;
    }

    /// Get the luminosity channel data.
    pub fn get_luma(&self) -> Option<&Vec<T>> {
        self.data.luma.as_ref()
    }

    /// Get a mutable reference to the luminosity channel data.
    pub fn get_mut_luma(&mut self) -> Option<&mut Vec<T>> {
        self.data.luma.as_mut()
    }

    /// Get the red channel data.
    pub fn get_red(&self) -> Option<&Vec<T>> {
        self.data.red.as_ref()
    }

    /// Get a mutable reference to the red channel data.
    pub fn get_mut_red(&mut self) -> Option<&mut Vec<T>> {
        self.data.red.as_mut()
    }

    /// Get the green channel data.
    pub fn get_green(&self) -> Option<&Vec<T>> {
        self.data.green.as_ref()
    }

    /// Get a mutable reference to the green channel data.
    pub fn get_mut_green(&mut self) -> Option<&mut Vec<T>> {
        self.data.green.as_mut()
    }

    /// Get the blue channel data.
    pub fn get_blue(&self) -> Option<&Vec<T>> {
        self.data.blue.as_ref()
    }

    /// Get a mutable reference to the blue channel data.
    pub fn get_mut_blue(&mut self) -> Option<&mut Vec<T>> {
        self.data.blue.as_mut()
    }

    /// Get the alpha channel data.
    pub fn get_alpha(&self) -> Option<&Vec<T>> {
        self.data.alpha.as_ref()
    }

    /// Get a mutable reference to the alpha channel data.
    pub fn get_mut_alpha(&mut self) -> Option<&mut Vec<T>> {
        self.data.alpha.as_mut()
    }

    /// Get image width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get image height.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the number of pixel elements.
    pub fn pixel_elems(&self) -> u8 {
        self.data.pixel_elems
    }

    /// Check if the image is grayscale.
    pub fn is_luma(&self) -> bool {
        self.data.pixel_elems == 1
    }

    /// Check if the image is RGB.
    pub fn is_rgb(&self) -> bool {
        self.data.pixel_elems == 3
    }

    /// Consume the image buffer and return a contiguous vector.
    ///
    /// Note:
    ///  - If the image is grayscale, the vector contains the luma channel data.
    ///  - If the image is grayscale with alpha channel, odd pixels are luminoisty and even pixels are alpha.
    ///  - If the image is RGB, the first element of the vector is red, the second element is green and the third element is blue and so on.
    ///  - If the image is RGB with alpha channel, the first element of the vector is red, the second element is green, the third element is blue and the fourth element is alpha and so on.
    pub fn into_vec(self) -> Vec<T> {
        let mut data =
            Vec::with_capacity(self.width * self.height * self.data.pixel_elems as usize);

        if self.width * self.height == 0 {
            return Vec::new();
        } else if self.data.pixel_elems == 1 {
            return self.data.luma.unwrap();
        } else if self.data.pixel_elems == 2 {
            let luma = self.data.luma.unwrap();
            let alpha = self.data.alpha.unwrap();
            for i in 0..self.width * self.height {
                data.push(luma[i]);
                data.push(alpha[i]);
            }
        } else if self.data.pixel_elems == 3 {
            let red = self.data.red.unwrap();
            let green = self.data.green.unwrap();
            let blue = self.data.blue.unwrap();
            for i in 0..self.width * self.height {
                data.push(red[i]);
                data.push(green[i]);
                data.push(blue[i]);
            }
        } else if self.data.pixel_elems == 4 {
            let red = self.data.red.unwrap();
            let green = self.data.green.unwrap();
            let blue = self.data.blue.unwrap();
            let alpha = self.data.alpha.unwrap();
            for i in 0..self.width * self.height {
                data.push(red[i]);
                data.push(green[i]);
                data.push(blue[i]);
                data.push(alpha[i]);
            }
        } else {
            panic!("Invalid number of elements");
        }

        data
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "fitsio")))]
#[cfg(feature = "fitsio")]
impl<T: Primitive + WriteImage> SerialImageBuffer<T> {
    /// Save the image data to a FITS file.
    ///
    /// # Arguments
    ///  * `dir_prefix` - The directory where the file will be saved.
    ///  * `file_prefix` - The prefix of the file name. The file name will be of the form `{file_prefix}_{timestamp}.fits`.
    ///  * `progname` - The name of the program that generated the image.
    ///  * `compress` - Whether to compress the FITS file.
    ///  * `overwrite` - Whether to overwrite the file if it already exists.
    ///  * `image_type` - The type of the image data (e.g. [`ImageType::UnsignedByte`])
    ///
    /// # Errors
    ///  * [`fitsio::errors::Error`] with the error description.
    fn savefits_generic(
        &self,
        dir_prefix: &Path,
        file_prefix: &str,
        progname: Option<&str>,
        compress: bool,
        overwrite: bool,
        image_type: ImageType,
    ) -> Result<PathBuf, FitsError> {
        if !dir_prefix.exists() {
            return Err(FitsError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Directory {:?} does not exist", dir_prefix),
            )));
        }
        let meta = self.get_metadata();
        let timestamp;
        let cameraname;
        if let Some(metadata) = &meta {
            timestamp = metadata
                .timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_millis();
            cameraname = metadata.camera_name.clone();
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
        let imgsize = [height, width];
        let data_type = image_type;

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

        let hdu = {
            {
                let primary = if self.is_luma() { "LUMINANCE" } else { "RED" };
                let hdu = fptr.create_image(primary, &img_desc)?;
                let channels;
                if self.is_luma() {
                    hdu.write_image(&mut fptr, self.get_luma().unwrap())?;
                    hdu.write_key(&mut fptr, "CHANNELS", 1)?;
                    channels = 1;
                } else if self.is_rgb() {
                    hdu.write_image(&mut fptr, self.get_red().unwrap())?;
                    let ghdu = fptr.create_image("GREEN", &img_desc)?;
                    ghdu.write_image(&mut fptr, self.get_green().unwrap())?;
                    let bhdu = fptr.create_image("BLUE", &img_desc)?;
                    bhdu.write_image(&mut fptr, self.get_blue().unwrap())?;
                    hdu.write_key(&mut fptr, "CHANNELS", 3)?;
                    channels = 3;
                } else {
                    return Err(FitsError::Message(format!(
                        "Unsupported image type {:?}",
                        data_type
                    )));
                }
                if let Some(alpha) = self.get_alpha() {
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
        if let Some(meta) = meta {
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

impl SerialImageBuffer<u8> {
    /// Create a new serializable image buffer.
    ///
    /// # Arguments
    ///  - `meta`: Image metadata (optional).
    ///  - `luma`: Luminosity data for a grayscale image. Set to `None` if it is a color image.
    ///  - `red`: Red channel data. Set to `None` if it is a grayscale image.
    ///  - `green`: Green channel data. Set to `None` if it is a grayscale image.
    ///  - `blue`: Blue channel data. Set to `None` if it is a grayscale image.
    ///  - `alpha`: Alpha channel data (optional).
    ///
    /// # Errors
    ///  - If `width * height == 0`.
    ///  - If all color channels are not specified.
    ///  - If `luma` and color channels are specified at the same time.
    ///  - If the length of the channel data stored in the image is not equal to `width * height`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        meta: Option<ImageMetaData>,
        luma: Option<Vec<u8>>,
        red: Option<Vec<u8>>,
        green: Option<Vec<u8>>,
        blue: Option<Vec<u8>>,
        alpha: Option<Vec<u8>>,
        width: usize,
        height: usize,
    ) -> Result<Self, &'static str> {
        if width * height == 0 {
            return Err("Width and height must be greater than zero");
        }
        let colors = red.is_some() as u8 + green.is_some() as u8 + blue.is_some() as u8;
        if colors > 0 && colors != 3 {
            return Err("All color channels must be specified.");
        }
        if luma.is_some() && colors > 0 {
            return Err("Luma and color channels cannot be specified at the same time");
        }
        if luma.is_some() && luma.as_ref().unwrap().len() != width * height {
            return Err("Length of luma channel must be equal to width * height");
        }
        if red.is_some() && red.as_ref().unwrap().len() != width * height {
            return Err("Length of red channel must be equal to width * height");
        }
        if green.is_some() && green.as_ref().unwrap().len() != width * height {
            return Err("Length of green channel must be equal to width * height");
        }
        if blue.is_some() && blue.as_ref().unwrap().len() != width * height {
            return Err("Length of blue channel must be equal to width * height");
        }
        if alpha.is_some() && alpha.as_ref().unwrap().len() != width * height {
            return Err("Length of alpha channel must be equal to width * height");
        }
        let pixel_elems = colors + luma.is_some() as u8 + alpha.is_some() as u8;
        Ok(Self {
            meta,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        })
    }

    /// Convert the image to grayscale, while discarding the alpha channel. The transformation used is `0.2162 * red + 0.7152 * green + 0.0722 * blue` for converting RGB to grayscale (see [here](https://stackoverflow.com/a/56678483)).
    pub fn into_luma(&self) -> SerialImageBuffer<u16> {
        let luma;
        if self.is_luma() {
            let sluma = self.data.luma.as_ref().unwrap();
            luma = sluma.iter().map(|x| ((*x as u16) << 8)).collect();
        } else if self.is_rgb() {
            let sred = self.data.red.as_ref().unwrap();
            let sgreen = self.data.green.as_ref().unwrap();
            let sblue = self.data.blue.as_ref().unwrap();
            luma = sred
                .iter()
                .zip(sgreen.iter())
                .zip(sblue.iter())
                .map(|((r, g), b)| {
                    R_LUT_U16[((*r as u16) << 8) as usize]
                        + G_LUT_U16[((*g as u16) << 8) as usize]
                        + B_LUT_U16[((*b as u16) << 8) as usize]
                })
                .collect();
        } else {
            panic!("Cannot convert image");
        }

        SerialImageBuffer::<u16>::new(
            self.meta.clone(),
            Some(luma),
            None,
            None,
            None,
            None,
            self.width,
            self.height,
        )
        .unwrap()
    }

    /// Convert the image to grayscale, while preserving the alpha channel. The transformation used is `0.2162 * red + 0.7152 * green + 0.0722 * blue` for converting RGB to grayscale (see [here](https://stackoverflow.com/a/56678483)).
    pub fn into_luma_alpha(&self) -> SerialImageBuffer<u16> {
        let img = self.into_luma();
        let alpha = self
            .data
            .alpha
            .as_ref()
            .map(|x| x.iter().map(|x| ((*x as u16) << 8)).collect());
        SerialImageBuffer::<u16>::new(
            img.meta,
            img.data.luma,
            None,
            None,
            None,
            alpha,
            self.width,
            self.height,
        )
        .unwrap()
    }

    /// Resize this image using the specified filter algorithm.
    /// Returns a new image. The image's aspect ratio is preserved.
    /// The image is scaled to the maximum possible size that fits
    /// within the bounds specified by `nwidth` and `nheight`.
    pub fn resize(self, nwidth: usize, nheight: usize, filter: FilterType ) -> Self {
        let meta = self.meta.clone();
        let img: DynamicImage = self.into();
        let img = img.resize(nwidth as u32, nheight as u32, filter);
        let mut img: Self = img.try_into().unwrap();
        img.set_metadata(meta);
        img
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
        self.savefits_generic(
            dir_prefix,
            file_prefix,
            progname,
            compress,
            overwrite,
            ImageType::UnsignedByte,
        )
    }
}

impl SerialImageBuffer<u16> {
    /// Create a new serializable image buffer.
    ///
    /// # Arguments
    ///  - `meta`: Image metadata (optional).
    ///  - `luma`: Luminosity data for a grayscale image. Set to `None` if it is a color image.
    ///  - `red`: Red channel data. Set to `None` if it is a grayscale image.
    ///  - `green`: Green channel data. Set to `None` if it is a grayscale image.
    ///  - `blue`: Blue channel data. Set to `None` if it is a grayscale image.
    ///  - `alpha`: Alpha channel data (optional).
    ///
    /// # Errors
    ///  - If `width * height == 0`.
    ///  - If all color channels are not specified.
    ///  - If `luma` and color channels are specified at the same time.
    ///  - If the length of the channel data stored in the image is not equal to `width * height`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        meta: Option<ImageMetaData>,
        luma: Option<Vec<u16>>,
        red: Option<Vec<u16>>,
        green: Option<Vec<u16>>,
        blue: Option<Vec<u16>>,
        alpha: Option<Vec<u16>>,
        width: usize,
        height: usize,
    ) -> Result<Self, &'static str> {
        if width * height == 0 {
            return Err("Width and height must be greater than zero");
        }
        let colors = red.is_some() as u8 + green.is_some() as u8 + blue.is_some() as u8;
        if colors > 0 && colors != 3 {
            return Err("All color channels must be specified.");
        }
        if luma.is_some() && colors > 0 {
            return Err("Luma and color channels cannot be specified at the same time");
        }
        if luma.is_some() && luma.as_ref().unwrap().len() != width * height {
            return Err("Length of luma channel must be equal to width * height");
        }
        if red.is_some() && red.as_ref().unwrap().len() != width * height {
            return Err("Length of red channel must be equal to width * height");
        }
        if green.is_some() && green.as_ref().unwrap().len() != width * height {
            return Err("Length of green channel must be equal to width * height");
        }
        if blue.is_some() && blue.as_ref().unwrap().len() != width * height {
            return Err("Length of blue channel must be equal to width * height");
        }
        if alpha.is_some() && alpha.as_ref().unwrap().len() != width * height {
            return Err("Length of alpha channel must be equal to width * height");
        }
        let pixel_elems = colors + luma.is_some() as u8 + alpha.is_some() as u8;
        Ok(Self {
            meta,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        })
    }

    /// Convert the image to grayscale, while discarding the alpha channel. The transformation used is `0.2162 * red + 0.7152 * green + 0.0722 * blue` for converting RGB to grayscale (see [here](https://stackoverflow.com/a/56678483)).
    pub fn into_luma(&self) -> SerialImageBuffer<u16> {
        let luma;
        if self.is_luma() {
            luma = self.data.luma.as_ref().unwrap().clone();
        } else if self.is_rgb() {
            let sred = self.data.red.as_ref().unwrap();
            let sgreen = self.data.green.as_ref().unwrap();
            let sblue = self.data.blue.as_ref().unwrap();
            luma = sred
                .iter()
                .zip(sgreen.iter())
                .zip(sblue.iter())
                .map(|((r, g), b)| {
                    R_LUT_U16[*r as usize] + G_LUT_U16[*g as usize] + B_LUT_U16[*b as usize]
                })
                .collect();
        } else {
            panic!("Cannot convert image");
        }
        SerialImageBuffer::<u16>::new(
            self.meta.clone(),
            Some(luma),
            None,
            None,
            None,
            None,
            self.width,
            self.height,
        )
        .unwrap()
    }

    /// Convert the image to grayscale, while preserving the alpha channel. The transformation used is `0.2162 * red + 0.7152 * green + 0.0722 * blue` for converting RGB to grayscale (see [here](https://stackoverflow.com/a/56678483)).
    pub fn into_luma_alpha(&self) -> SerialImageBuffer<u16> {
        let img = self.into_luma();
        SerialImageBuffer::<u16>::new(
            img.meta,
            img.data.luma,
            None,
            None,
            None,
            self.data.alpha.clone(),
            self.width,
            self.height,
        )
        .unwrap()
    }

    /// Resize this image using the specified filter algorithm.
    /// Returns a new image. The image's aspect ratio is preserved.
    /// The image is scaled to the maximum possible size that fits
    /// within the bounds specified by `nwidth` and `nheight`.
    pub fn resize(self, nwidth: usize, nheight: usize, filter: FilterType ) -> Self {
        let meta = self.meta.clone();
        let img: DynamicImage = self.into();
        let img = img.resize(nwidth as u32, nheight as u32, filter);
        let mut img: Self = img.try_into().unwrap();
        img.set_metadata(meta);
        img
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
        self.savefits_generic(
            dir_prefix,
            file_prefix,
            progname,
            compress,
            overwrite,
            ImageType::UnsignedShort,
        )
    }
}

impl SerialImageBuffer<f32> {
    /// Create a new serializable image buffer.
    ///
    /// # Arguments
    ///  - `meta`: Image metadata (optional).
    ///  - `red`: Red channel data. Set to `None` if it is a grayscale image.
    ///  - `green`: Green channel data. Set to `None` if it is a grayscale image.
    ///  - `blue`: Blue channel data. Set to `None` if it is a grayscale image.
    ///  - `alpha`: Alpha channel data (optional).
    ///
    /// # Errors
    ///  - If `width * height == 0`.
    ///  - If the length of the channel data stored in the image is not equal to `width * height`.
    pub fn new(
        meta: Option<ImageMetaData>,
        red: Vec<f32>,
        green: Vec<f32>,
        blue: Vec<f32>,
        alpha: Option<Vec<f32>>,
        width: usize,
        height: usize,
    ) -> Result<Self, &'static str> {
        if width * height == 0 {
            return Err("Width and height must be greater than zero");
        }
        if red.len() != width * height {
            return Err("Length of red channel must be equal to width * height");
        }
        if green.len() != width * height {
            return Err("Length of green channel must be equal to width * height");
        }
        if blue.len() != width * height {
            return Err("Length of blue channel must be equal to width * height");
        }
        if alpha.is_some() && alpha.as_ref().unwrap().len() != width * height {
            return Err("Length of alpha channel must be equal to width * height");
        }
        let elems = if alpha.is_some() { 4 } else { 3 };
        Ok(Self {
            meta,
            data: SerialImageInternal {
                luma: None,
                red: Some(red),
                green: Some(green),
                blue: Some(blue),
                alpha,
                pixel_elems: elems,
            },
            width,
            height,
        })
    }

    /// Convert the image to grayscale, discarding the alpha channel. The transformation used is `0.2162 * red + 0.7152 * green + 0.0722 * blue` for converting RGB to grayscale (see [here](https://stackoverflow.com/a/56678483)).
    pub fn into_luma(&self) -> SerialImageBuffer<u16> {
        let luma;
        if self.is_luma() {
            let sluma = self.data.luma.as_ref().unwrap();
            luma = sluma
                .iter()
                .map(|x| (*x * u16::MAX as f32).round() as u16)
                .collect();
        } else if self.is_rgb() {
            let sred = self.data.red.as_ref().unwrap();
            let sgreen = self.data.green.as_ref().unwrap();
            let sblue = self.data.blue.as_ref().unwrap();
            luma = sred
                .iter()
                .zip(sgreen.iter())
                .zip(sblue.iter())
                .map(|((r, g), b)| (0.2162 * *r + 0.7152 * *g + 0.0722 * *b).round() as u16)
                .collect();
        } else {
            panic!("Cannot convert image");
        }
        SerialImageBuffer::<u16>::new(
            self.meta.clone(),
            Some(luma),
            None,
            None,
            None,
            None,
            self.width,
            self.height,
        )
        .unwrap()
    }

    /// Convert the image to grayscale, while preserving the alpha channel. The transformation used is `0.2162 * red + 0.7152 * green + 0.0722 * blue` for converting RGB to grayscale (see [here](https://stackoverflow.com/a/56678483)).
    pub fn into_luma_alpha(&self) -> SerialImageBuffer<u16> {
        let img = self.into_luma();
        let alpha = self.data.alpha.as_ref().map(|x| x.iter()
                    .map(|x| (*x * u16::MAX as f32).round() as u16)
                    .collect());
        SerialImageBuffer::<u16>::new(
            img.meta,
            img.data.luma,
            None,
            None,
            None,
            alpha,
            self.width,
            self.height,
        )
        .unwrap()
    }

    /// Resize this image using the specified filter algorithm.
    /// Returns a new image. The image's aspect ratio is preserved.
    /// The image is scaled to the maximum possible size that fits
    /// within the bounds specified by `nwidth` and `nheight`.
    pub fn resize(self, nwidth: usize, nheight: usize, filter: FilterType ) -> Self {
        let meta = self.meta.clone();
        let img: DynamicImage = self.into();
        let img = img.resize(nwidth as u32, nheight as u32, filter);
        let mut img: Self = img.try_into().unwrap();
        img.set_metadata(meta);
        img
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
        self.savefits_generic(
            dir_prefix,
            file_prefix,
            progname,
            compress,
            overwrite,
            ImageType::Float,
        )
    }
}

impl TryFrom<DynamicImage> for SerialImageBuffer<u8> {
    type Error = &'static str;

    fn try_from(image: DynamicImage) -> Result<Self, Self::Error> {
        let (width, height) = (image.width() as usize, image.height() as usize);
        let pixel_elems = image.color().channel_count();
        let luma;
        let red;
        let green;
        let blue;
        let alpha;

        match image {
            DynamicImage::ImageLuma8(img) => {
                luma = Some(img.into_raw());
                red = None;
                green = None;
                blue = None;
                alpha = None;
            }
            DynamicImage::ImageLumaA8(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.into_raw(), pixel_elems)
            }
            DynamicImage::ImageRgb8(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.into_raw(), pixel_elems)
            }
            DynamicImage::ImageRgba8(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.into_raw(), pixel_elems)
            }
            _ => {
                return Err("Image type not supported");
            }
        }

        Ok(Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        })
    }
}

impl TryFrom<DynamicImage> for SerialImageBuffer<u16> {
    type Error = &'static str;

    fn try_from(image: DynamicImage) -> Result<Self, Self::Error> {
        let (width, height) = (image.width() as usize, image.height() as usize);
        let pixel_elems = image.color().channel_count();
        let luma;
        let red;
        let green;
        let blue;
        let alpha;

        match image {
            DynamicImage::ImageLuma16(img) => {
                luma = Some(img.into_raw());
                red = None;
                green = None;
                blue = None;
                alpha = None;
            }
            DynamicImage::ImageLumaA16(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.into_raw(), pixel_elems)
            }
            DynamicImage::ImageRgb16(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.into_raw(), pixel_elems)
            }
            DynamicImage::ImageRgba16(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.into_raw(), pixel_elems)
            }
            _ => {
                return Err("Image type not supported");
            }
        }

        Ok(Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        })
    }
}

impl TryFrom<DynamicImage> for SerialImageBuffer<f32> {
    type Error = &'static str;

    fn try_from(image: DynamicImage) -> Result<Self, Self::Error> {
        let (width, height) = (image.width() as usize, image.height() as usize);
        let pixel_elems = image.color().channel_count();
        let luma;
        let red;
        let green;
        let blue;
        let alpha;

        match image {
            DynamicImage::ImageRgb32F(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.into_raw(), pixel_elems)
            }
            DynamicImage::ImageRgba32F(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.into_raw(), pixel_elems)
            }
            _ => {
                return Err("Image type not supported");
            }
        }

        Ok(Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<DynamicImage> for SerialImageBuffer<u8> {
    fn into(self) -> DynamicImage {
        let width = self.width;
        let height = self.height;
        let pixel_elems = self.data.pixel_elems;
        let data = self.into_vec();

        match pixel_elems {
            1 => {
                let img = ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageLuma8(img)
            }
            2 => {
                let img = ImageBuffer::<image::LumaA<u8>, Vec<u8>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageLumaA8(img)
            }
            3 => {
                let img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgb8(img)
            }
            4 => {
                let img = ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgba8(img)
            }
            _ => panic!("Pixel elements not supported"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<DynamicImage> for SerialImageBuffer<u16> {
    fn into(self) -> DynamicImage {
        let width = self.width;
        let height = self.height;
        let pixel_elems = self.data.pixel_elems;
        let data = self.into_vec();

        match pixel_elems {
            1 => {
                let img = ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageLuma16(img)
            }
            2 => {
                let img = ImageBuffer::<image::LumaA<u16>, Vec<u16>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageLumaA16(img)
            }
            3 => {
                let img = ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgb16(img)
            }
            4 => {
                let img = ImageBuffer::<image::Rgba<u16>, Vec<u16>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgba16(img)
            }
            _ => panic!("Pixel elements not supported"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<DynamicImage> for SerialImageBuffer<f32> {
    fn into(self) -> DynamicImage {
        let width = self.width;
        let height = self.height;
        let pixel_elems = self.data.pixel_elems;
        let data = self.into_vec();

        match pixel_elems {
            3 => {
                let img = ImageBuffer::<image::Rgb<f32>, Vec<f32>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgb32F(img)
            }
            4 => {
                let img = ImageBuffer::<image::Rgba<f32>, Vec<f32>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgba32F(img)
            }
            _ => panic!("Pixel elements not supported"),
        }
    }
}

impl TryFrom<&DynamicImage> for SerialImageBuffer<u8> {
    type Error = &'static str;

    fn try_from(image: &DynamicImage) -> Result<Self, Self::Error> {
        let (width, height) = (image.width() as usize, image.height() as usize);
        let pixel_elems = image.color().channel_count();
        let luma;
        let red;
        let green;
        let blue;
        let alpha;

        match image {
            DynamicImage::ImageLuma8(img) => {
                luma = Some(img.as_raw().clone());
                red = None;
                green = None;
                blue = None;
                alpha = None;
            }
            DynamicImage::ImageLumaA8(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.as_raw().clone(), pixel_elems);
            }
            DynamicImage::ImageRgb8(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.as_raw().clone(), pixel_elems);
            }
            DynamicImage::ImageRgba8(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.as_raw().clone(), pixel_elems);
            }
            _ => {
                return Err("Image type not supported");
            }
        }

        Ok(Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        })
    }
}

impl TryFrom<&DynamicImage> for SerialImageBuffer<u16> {
    type Error = &'static str;

    fn try_from(image: &DynamicImage) -> Result<Self, Self::Error> {
        let (width, height) = (image.width() as usize, image.height() as usize);
        let pixel_elems = image.color().channel_count();
        let luma;
        let red;
        let green;
        let blue;
        let alpha;

        match image {
            DynamicImage::ImageLuma16(img) => {
                luma = Some(img.as_raw().clone());
                red = None;
                green = None;
                blue = None;
                alpha = None;
            }
            DynamicImage::ImageLumaA16(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.as_raw().clone(), pixel_elems);
            }
            DynamicImage::ImageRgb16(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.as_raw().clone(), pixel_elems);
            }
            DynamicImage::ImageRgba16(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.as_raw().clone(), pixel_elems);
            }
            _ => {
                return Err("Image type not supported");
            }
        }

        Ok(Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        })
    }
}

impl TryFrom<&DynamicImage> for SerialImageBuffer<f32> {
    type Error = &'static str;

    fn try_from(image: &DynamicImage) -> Result<Self, Self::Error> {
        let (width, height) = (image.width() as usize, image.height() as usize);
        let pixel_elems = image.color().channel_count();
        let luma;
        let red;
        let green;
        let blue;
        let alpha;

        match image {
            DynamicImage::ImageRgb32F(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.clone().into_raw(), pixel_elems)
            }
            DynamicImage::ImageRgba32F(img) => {
                (luma, red, green, blue, alpha) =
                    Self::from_vec_unsafe(width * height, img.clone().into_raw(), pixel_elems)
            }
            _ => {
                return Err("Image type not supported");
            }
        }

        Ok(Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<DynamicImage> for &SerialImageBuffer<u8> {
    fn into(self) -> DynamicImage {
        let width = self.width;
        let height = self.height;
        let pixel_elems = self.data.pixel_elems;
        let data = self.clone().into_vec();

        match pixel_elems {
            1 => {
                let img = ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageLuma8(img)
            }
            2 => {
                let img = ImageBuffer::<image::LumaA<u8>, Vec<u8>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageLumaA8(img)
            }
            3 => {
                let img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgb8(img)
            }
            4 => {
                let img = ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgba8(img)
            }
            _ => panic!("Pixel elements not supported"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<DynamicImage> for &SerialImageBuffer<u16> {
    fn into(self) -> DynamicImage {
        let width = self.width;
        let height = self.height;
        let pixel_elems = self.data.pixel_elems;
        let data = self.clone().into_vec();

        match pixel_elems {
            1 => {
                let img = ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageLuma16(img)
            }
            2 => {
                let img = ImageBuffer::<image::LumaA<u16>, Vec<u16>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageLumaA16(img)
            }
            3 => {
                let img = ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgb16(img)
            }
            4 => {
                let img = ImageBuffer::<image::Rgba<u16>, Vec<u16>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgba16(img)
            }
            _ => panic!("Pixel elements not supported"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<DynamicImage> for &SerialImageBuffer<f32> {
    fn into(self) -> DynamicImage {
        let width = self.width;
        let height = self.height;
        let pixel_elems = self.data.pixel_elems;
        let data = self.clone().into_vec();

        match pixel_elems {
            3 => {
                let img = ImageBuffer::<image::Rgb<f32>, Vec<f32>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgb32F(img)
            }
            4 => {
                let img = ImageBuffer::<image::Rgba<f32>, Vec<f32>>::from_raw(
                    width as u32,
                    height as u32,
                    data,
                )
                .unwrap();
                DynamicImage::ImageRgba32F(img)
            }
            _ => panic!("Pixel elements not supported"),
        }
    }
}

impl<T: Primitive> TryInto<ImageBuffer<Luma<T>, Vec<T>>> for SerialImageBuffer<T> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<Luma<T>, Vec<T>>, Self::Error> {
        if self.data.pixel_elems != 1 {
            return Err("Image must have one element per pixel");
        }
        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }
        let img = ImageBuffer::<Luma<T>, Vec<T>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl<T: Primitive> TryInto<ImageBuffer<Luma<T>, Vec<T>>> for &SerialImageBuffer<T> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<Luma<T>, Vec<T>>, Self::Error> {
        if self.data.pixel_elems != 1 {
            return Err("Image must have one element per pixel");
        }

        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }

        let img = ImageBuffer::<Luma<T>, Vec<T>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.clone().unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl<T: Primitive> TryInto<ImageBuffer<LumaA<T>, Vec<T>>> for SerialImageBuffer<T> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<LumaA<T>, Vec<T>>, Self::Error> {
        if self.data.pixel_elems != 2 {
            return Err("Image must have two elements per pixel");
        }

        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }

        let img = ImageBuffer::<LumaA<T>, Vec<T>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl<T: Primitive> TryInto<ImageBuffer<LumaA<T>, Vec<T>>> for &SerialImageBuffer<T> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<LumaA<T>, Vec<T>>, Self::Error> {
        if self.data.pixel_elems != 2 {
            return Err("Image must have two elements per pixel");
        }

        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }

        let img = ImageBuffer::<LumaA<T>, Vec<T>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.clone().unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl TryInto<ImageBuffer<Rgb<u8>, Vec<u8>>> for SerialImageBuffer<u8> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Self::Error> {
        if self.data.pixel_elems != 3 {
            return Err("Image must have three elements per pixel");
        }

        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }

        let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl TryInto<ImageBuffer<Rgb<u8>, Vec<u8>>> for &SerialImageBuffer<u8> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Self::Error> {
        if self.data.pixel_elems != 3 {
            return Err("Image must have three elements per pixel");
        }

        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }

        let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.clone().unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl TryInto<ImageBuffer<Rgb<u16>, Vec<u16>>> for SerialImageBuffer<u16> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<Rgb<u16>, Vec<u16>>, Self::Error> {
        if self.data.pixel_elems != 3 {
            return Err("Image must have three elements per pixel");
        }

        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }

        let img = ImageBuffer::<Rgb<u16>, Vec<u16>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl TryInto<ImageBuffer<Rgb<u16>, Vec<u16>>> for &SerialImageBuffer<u16> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<Rgb<u16>, Vec<u16>>, Self::Error> {
        if self.data.pixel_elems != 3 {
            return Err("Image must have three elements per pixel");
        }

        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }

        let img = ImageBuffer::<Rgb<u16>, Vec<u16>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.clone().unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl TryInto<ImageBuffer<Rgb<f32>, Vec<f32>>> for SerialImageBuffer<f32> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<Rgb<f32>, Vec<f32>>, Self::Error> {
        if self.data.pixel_elems != 3 {
            return Err("Image must have three elements per pixel");
        }

        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }

        let img = ImageBuffer::<Rgb<f32>, Vec<f32>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl TryInto<ImageBuffer<Rgb<f32>, Vec<f32>>> for &SerialImageBuffer<f32> {
    type Error = &'static str;

    fn try_into(self) -> Result<ImageBuffer<Rgb<f32>, Vec<f32>>, Self::Error> {
        if self.data.pixel_elems != 3 {
            return Err("Image must have three elements per pixel");
        }

        if self.width * self.height == 0 {
            return Err("Image must have non-zero dimensions");
        }

        let img = ImageBuffer::<Rgb<f32>, Vec<f32>>::from_raw(
            self.width as u32,
            self.height as u32,
            self.data.luma.clone().unwrap(),
        );
        if img.is_none() {
            return Err("Failed to convert to image buffer");
        }
        Ok(img.unwrap())
    }
}

impl<T: Primitive> From<ImageBuffer<Luma<T>, Vec<T>>> for SerialImageBuffer<T> {
    fn from(img: ImageBuffer<Luma<T>, Vec<T>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 1;
        let data = img.into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

impl<T: Primitive> From<&ImageBuffer<Luma<T>, Vec<T>>> for SerialImageBuffer<T> {
    fn from(img: &ImageBuffer<Luma<T>, Vec<T>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 1;
        let data = img.clone().into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

impl<T: Primitive> From<ImageBuffer<LumaA<T>, Vec<T>>> for SerialImageBuffer<T> {
    fn from(img: ImageBuffer<LumaA<T>, Vec<T>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 2;
        let data = img.into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

impl<T: Primitive> From<&ImageBuffer<LumaA<T>, Vec<T>>> for SerialImageBuffer<T> {
    fn from(img: &ImageBuffer<LumaA<T>, Vec<T>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 2;
        let data = img.clone().into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

impl From<ImageBuffer<Rgb<u8>, Vec<u8>>> for SerialImageBuffer<u8> {
    fn from(img: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 3;
        let data = img.into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

impl From<&ImageBuffer<Rgb<u8>, Vec<u8>>> for SerialImageBuffer<u8> {
    fn from(img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 3;
        let data = img.clone().into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

impl From<ImageBuffer<Rgb<u16>, Vec<u16>>> for SerialImageBuffer<u16> {
    fn from(img: ImageBuffer<Rgb<u16>, Vec<u16>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 3;
        let data = img.into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

impl From<&ImageBuffer<Rgb<u16>, Vec<u16>>> for SerialImageBuffer<u16> {
    fn from(img: &ImageBuffer<Rgb<u16>, Vec<u16>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 3;
        let data = img.clone().into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

impl From<ImageBuffer<Rgb<f32>, Vec<f32>>> for SerialImageBuffer<f32> {
    fn from(img: ImageBuffer<Rgb<f32>, Vec<f32>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 3;
        let data = img.clone().into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

impl From<&ImageBuffer<Rgb<f32>, Vec<f32>>> for SerialImageBuffer<f32> {
    fn from(img: &ImageBuffer<Rgb<f32>, Vec<f32>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let pixel_elems = 3;
        let data = img.clone().into_raw();
        let (luma, red, green, blue, alpha) =
            Self::from_vec_unsafe(width * height, data, pixel_elems);
        Self {
            meta: None,
            data: SerialImageInternal {
                luma,
                red,
                green,
                blue,
                alpha,
                pixel_elems,
            },
            width,
            height,
        }
    }
}

fn get_red_lut_16() -> [u16; u16::MAX as usize + 1] {
    let mut lut = [0u16; u16::MAX as usize + 1];
    let mut ctr = 0;
    while ctr < u16::MAX as usize + 1 {
        lut[ctr] = (ctr as f32 * 0.2126).round() as u16;
        ctr += 1;
    }
    lut
}

fn get_green_lut_16() -> [u16; u16::MAX as usize + 1] {
    let mut lut = [0u16; u16::MAX as usize + 1];
    let mut ctr = 0;
    while ctr < u16::MAX as usize + 1 {
        lut[ctr] = (ctr as f32 * 0.7152).round() as u16;
        ctr += 1;
    }
    lut
}

fn get_blue_lut_16() -> [u16; u16::MAX as usize + 1] {
    let mut lut = [0u16; u16::MAX as usize + 1];
    let mut ctr = 0;
    while ctr < u16::MAX as usize + 1 {
        lut[ctr] = (ctr as f32 * 0.0722).round() as u16;
        ctr += 1;
    }
    lut
}

static R_LUT_U16: Lazy<[u16; u16::MAX as usize + 1]> = Lazy::new(get_red_lut_16);
static G_LUT_U16: Lazy<[u16; u16::MAX as usize + 1]> = Lazy::new(get_green_lut_16);
static B_LUT_U16: Lazy<[u16; u16::MAX as usize + 1]> = Lazy::new(get_blue_lut_16);
