#![cfg_attr(docsrs, feature(doc_cfg))]

/*!
# serialimage
This crate extends the [`image`](https://crates.io/crates/image) crate with serializable `DynamicImage`s: the `DynamicSerialImage`s. 
Additionally, it implements an `ImageMetaData` struct to pack additional metadata information.
Note, however, the metadata information is lost on conversion from `DynamicSerialImage` to `DynamicImage`.

The `DynamicSerialImage` struct stores the image data internally in separate channels without additional overhead. 
Similar to the `image` crate, the internal image buffer (`SerialImageBuffer` for `serialimage`) supports base data 
types of `u8`, `u16` and `f32`. `SerialImageBuffer<u8>` and `SerialImageBuffer<u16>` structs support both grayscale 
and RGB images. The `SerialImageBuffer<f32>` struct only supports RGB images. Alpha channels are supported for all three types.

Conversions between `image` and `serialimage` data types incur memory copy overheads only when the channel 
count is > 1, i.e. the images are RGB or contain transparency data due to the differences in memory layout.


## Usage
Add the following to your `Cargo.toml`:
```toml
[dependencies]
serialimage = "3.0.0"
```
and the following to your source code:
```no_run
use serialimage::{DynamicSerialImage, ImageMetaData};
```

Then, you can create a new image metadata object:
```no_run
let meta = ImageMetaData::new(...);
```

Then, a `DynamicSerialImage` can be created from a `DynamicImage`. For example, with a `DynamicImage` from a `Luma<u16>` pixel type image buffer,
```no_run
let img = DynamicImage::from(ImageBuffer::<Luma<u16>, Vec<u16>>::new(10, 10)); // create DynamicImage
let mut img = DynamicSerialImage::from(img); // create DynamicSerialImage
img.set_metadata(meta); // set the metadata
let imgstr = serde_json::to_string(&img).unwrap(); // serialize
let simg: DynamicSerialImage = serde_json::from_str(&imgstr).unwrap(); // deserialize
assert_eq!(img, simg);
```
Now `img` can be sent on its merry way with full serialization

## Traits
`DynamicSerialImage` and `SerialImageBuffer` implements the `TryFrom` and `TryInto` traits 
for `image::DynamicImage` and `image::ImageBuffer`.

## Optional Features
Additionally, the `Serial` image types optionally support saving as FITS images (method `savefits()`). 
This feature is not enabled by default, and is available behind the `fitsio` feature flag. 
This feature flag can be enabled to allow for FITS image storage.

Add the following to your `Cargo.toml` to enable this:
```toml
[dependencies]
serialimage = { version = "3.0.0", features = ["fitsio"] }
```

The FITS I/O is hidden behind a feature flag to avoid compilation errors on `wasm` targets.
 
*/

mod dynamicserialimage;
mod imagemetadata;
mod serialimage;
mod optimalexposure;

pub use serialimage::*;

pub use dynamicserialimage::*;

pub use imagemetadata::*;

pub use optimalexposure::*;

#[cfg(test)]
mod tests {
    #[cfg_attr(not(feature = "fitsio"), ignore)]
    #[cfg(feature = "fitsio")]
    use std::path::Path;

    use image::{DynamicImage, ImageBuffer, Luma};

    use serde_json::{self};

    use rand::{thread_rng, Rng};

    use crate::{DynamicSerialImage, ImageMetaData, SerialImageBuffer};

    #[test]
    fn test() {
        test_luma_u8();
        test_rgb_u8();
        test_rgb_f32();
        test_rgb_u16();
        test_readme();
    }

    fn test_luma_u8() {
        // 1. Generate vector of randoms
        let mut rng = thread_rng();
        let width = 10;
        let height = 10;
        let mut imgdata = Vec::<u8>::with_capacity(width * height);
        for _ in 0..width * height {
            imgdata.push(rng.gen_range(0..=255));
        }
        let img = SerialImageBuffer::from_vec(width, height, imgdata).unwrap();
        let img: DynamicSerialImage = img.into();
        let val = serde_json::to_string(&img).unwrap();
        let simg: DynamicSerialImage = serde_json::from_str(&val).unwrap();
        assert_eq!(img, simg);
        let dimg = DynamicImage::from(&simg);
        assert_eq!(dimg.width(), width as u32);
        let img = DynamicImage::from(simg);
        assert_eq!(img.width(), width as u32);
        let img = DynamicSerialImage::from(dimg);
        assert_eq!(img.width(), width);
        print!("\n\n\n");
    }

    fn test_readme() {
        let meta = ImageMetaData::default();
        let img = DynamicImage::from(ImageBuffer::<Luma<u16>, Vec<u16>>::new(10, 10)); // create DynamicImage
        let mut img = DynamicSerialImage::from(img); // create DynamicSerialImage
        img.set_metadata(meta); // set the metadata
        let imgstr = serde_json::to_string(&img).unwrap(); // serialize
        let simg: DynamicSerialImage = serde_json::from_str(&imgstr).unwrap(); // deserialize
        assert_eq!(img, simg);
    }

    fn test_rgb_u8() {
        // 1. Generate vector of randoms
        let mut rng = thread_rng();
        let width = 800;
        let height = 600;
        let mut imgdata = Vec::<u8>::with_capacity(width * height * 3);
        for _ in 0..width * height {
            imgdata.push(rng.gen_range(0..=255));
            imgdata.push(rng.gen_range(0..=255));
            imgdata.push(rng.gen_range(0..=255));
        }
        let img = SerialImageBuffer::from_vec(width, height, imgdata).unwrap();
        let img: DynamicSerialImage = img.into();
        #[cfg(feature = "fitsio")]
        img.savefits(Path::new("./"), "rgb_u8", None, false, true)
            .unwrap();
        let val = serde_json::to_string(&img).unwrap();
        let simg: DynamicSerialImage = serde_json::from_str(&val).unwrap();
        assert_eq!(img, simg);
        let dimg = DynamicImage::from(&simg);
        assert_eq!(dimg.width(), width as u32);
        let img = DynamicImage::from(simg);
        assert_eq!(img.width(), width as u32);
        let img = DynamicSerialImage::from(dimg);
        assert_eq!(img.width(), width);
        #[cfg(feature = "fitsio")]
        img.savefits(Path::new("./"), "rgb_u8_deser", None, false, true)
            .unwrap();
    }

    fn test_rgb_f32() {
        let mut rng = thread_rng();
        let width = 800;
        let height = 600;
        let mut imgdata = Vec::<f32>::with_capacity(width * height * 3);
        for _ in 0..width * height {
            imgdata.push(rng.gen_range(0.0..=1.0));
            imgdata.push(rng.gen_range(0.0..=1.0));
            imgdata.push(rng.gen_range(0.0..=1.0));
        }
        let img = SerialImageBuffer::from_vec(width, height, imgdata).unwrap();
        let img: DynamicSerialImage = img.into();
        let val = serde_json::to_string(&img).unwrap();
        let simg: DynamicSerialImage = serde_json::from_str(&val).unwrap();
        assert_eq!(img, simg);
        let dimg = DynamicImage::from(&simg);
        assert_eq!(dimg.width(), width as u32);
        let img = DynamicImage::from(simg);
        assert_eq!(img.width(), width as u32);
        let img = DynamicSerialImage::from(dimg);
        assert_eq!(img.width(), width);
        #[cfg(feature = "fitsio")]
        img.savefits(Path::new("./"), "rgb_f32", None, false, true)
            .unwrap();
    }

    fn test_rgb_u16() {
        let mut rng = thread_rng();
        let width = 800;
        let height = 600;
        let mut imgdata = Vec::<u16>::with_capacity(width * height * 3);
        for _ in 0..width * height {
            imgdata.push(rng.gen_range(0..=65535));
            imgdata.push(rng.gen_range(0..=65535));
            imgdata.push(rng.gen_range(0..=65535));
        }
        let img = SerialImageBuffer::from_vec(width, height, imgdata).unwrap();
        let img: DynamicSerialImage = img.into();
        img.save("test_rgb.png").unwrap();
        #[cfg(feature = "fitsio")]
        img.savefits(Path::new("./"), "rgb_u16", None, false, true)
            .unwrap();
        let val = serde_json::to_string(&img).unwrap();
        let simg: DynamicSerialImage = serde_json::from_str(&val).unwrap();
        assert_eq!(img, simg);
        let dimg = DynamicImage::from(&simg);
        assert_eq!(dimg.width(), width as u32);
        let img = DynamicImage::from(simg);
        assert_eq!(img.width(), width as u32);
        let img = DynamicSerialImage::from(dimg);
        assert_eq!(img.width(), width);
        let img: DynamicSerialImage = img.into_luma().into();
        let img = img.resize(1024, 1024, image::imageops::FilterType::Nearest);
        img.save("test_luma.png").unwrap();
        #[cfg(feature = "fitsio")]
        img.savefits(Path::new("./"), "luma_u16", None, false, true)
            .unwrap();
    }
}
