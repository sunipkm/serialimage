mod dynamicserialimage;
mod imagemetadata;
mod serialimage;

pub use serialimage::*;

pub use dynamicserialimage::*;

pub use imagemetadata::*;

#[cfg(test)]
mod tests {
    use image::DynamicImage;

    use serde_json::{self};

    use rand::{thread_rng, Rng};

    use crate::{DynamicSerialImage, SerialImageBuffer};

    #[test]
    fn test() {
        test_luma_u8();
        test_rgb_u8();
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
        println!("xxxxxxxxxxxxxxxxxxxxxx");
    }
}
