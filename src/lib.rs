#![doc = document_features::document_features!()]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

mod dynamicserialimage;
mod imagemetadata;
mod serialimage;

pub use serialimage::*;

pub use dynamicserialimage::*;

pub use imagemetadata::*;

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
        println!("Test Luma u8");
        // 1. Generate vector of randoms
        let mut rng = thread_rng();
        let width = 10;
        let height = 10;
        let mut imgdata = Vec::<u8>::with_capacity(width as usize * height as usize);
        for _ in 0..width * height {
            imgdata.push(rng.gen_range(0..=255));
        }
        let img = SerialImageBuffer::from_vec(width, height, imgdata).unwrap();
        let img: DynamicSerialImage = img.into();
        let val = serde_json::to_string(&img).unwrap();
        println!("{}", val);
        let simg: DynamicSerialImage = serde_json::from_str(&val).unwrap();
        assert_eq!(img, simg);
        let dimg = DynamicImage::from(&simg);
        assert_eq!(dimg.width(), width as u32);
        let img = DynamicImage::from(simg);
        assert_eq!(img.width(), width as u32);
        let img = DynamicSerialImage::from(dimg);
        assert_eq!(img.width(), width as usize);
        println!("xxxxxxxxxxxxxxxxxxxxxx");
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
        println!("Test RGB u8");
        // 1. Generate vector of randoms
        let mut rng = thread_rng();
        let width = 10;
        let height = 10;
        let mut imgdata = Vec::<u8>::with_capacity(width as usize * height as usize * 3);
        for _ in 0..width * height {
            imgdata.push(rng.gen_range(0..=255));
            imgdata.push(rng.gen_range(0..=255));
            imgdata.push(rng.gen_range(0..=255));
        }
        let img = SerialImageBuffer::from_vec(width, height, imgdata).unwrap();
        let img: DynamicSerialImage = img.into();
        let val = serde_json::to_string(&img).unwrap();
        println!("{}", val);
        let simg: DynamicSerialImage = serde_json::from_str(&val).unwrap();
        assert_eq!(img, simg);
        let dimg = DynamicImage::from(&simg);
        assert_eq!(dimg.width(), width as u32);
        let img = DynamicImage::from(simg);
        assert_eq!(img.width(), width as u32);
        let img = DynamicSerialImage::from(dimg);
        assert_eq!(img.width(), width as usize);
        #[cfg(feature = "fitsio")]
        img.savefits(Path::new("./"), "", None, false, true)
            .unwrap();
        println!("xxxxxxxxxxxxxxxxxxxxxx");
    }

    fn test_rgb_f32() {
        let mut rng = thread_rng();
        let width = 800;
        let height = 600;
        let mut imgdata = Vec::<f32>::with_capacity(width as usize * height as usize * 3);
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
        assert_eq!(img.width(), width as usize);
        #[cfg(feature = "fitsio")]
        img.savefits(Path::new("./"), "", None, false, true)
            .unwrap();
    }

    fn test_rgb_u16() {
        let mut rng = thread_rng();
        let width = 800;
        let height = 600;
        let mut imgdata = Vec::<u16>::with_capacity(width as usize * height as usize * 3);
        for _ in 0..width * height {
            imgdata.push(rng.gen_range(0..=65535));
            imgdata.push(rng.gen_range(0..=65535));
            imgdata.push(rng.gen_range(0..=65535));
        }
        let img = SerialImageBuffer::from_vec(width, height, imgdata).unwrap();
        let img: DynamicSerialImage = img.into();
        img.save("test_rgb.png").unwrap();
        let val = serde_json::to_string(&img).unwrap();
        let simg: DynamicSerialImage = serde_json::from_str(&val).unwrap();
        assert_eq!(img, simg);
        let dimg = DynamicImage::from(&simg);
        assert_eq!(dimg.width(), width as u32);
        let img = DynamicImage::from(simg);
        assert_eq!(img.width(), width as u32);
        let img = DynamicSerialImage::from(dimg);
        assert_eq!(img.width(), width as usize);
        let img: DynamicSerialImage = img.into_luma().into();
        img.save("test_luma.png").unwrap();
        #[cfg(feature = "fitsio")]
        img.savefits(Path::new("./"), "", None, false, true)
            .unwrap();
    }
}
