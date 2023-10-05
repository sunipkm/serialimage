mod serialimage;

pub use serialimage::{
    ImageMetaData, SerialImageBuffer, SerialImagePixel, SerialImageStorageTypes, DynamicSerialImage, 
};

pub use image::DynamicImage;

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use image::{DynamicImage, ImageBuffer, Luma};

    use serde_json::{self};

    use rand::{Rng, thread_rng};

    use crate::{ImageMetaData, SerialImageBuffer, SerialImagePixel, DynamicSerialImage};

    #[test]
    fn test() {
        let meta = ImageMetaData::new(
            SystemTime::now(),
            Duration::from_secs(1),
            -10.0,
            1,
            1,
            "test",
            100,
            0,
        );
        test_meta(Some(meta));
        test_meta(None);
    }

    fn test_meta(meta: Option<ImageMetaData>) {
        println!("With metadata: {}", meta.is_some());
        let img = DynamicImage::from(ImageBuffer::<Luma<u16>, Vec<u16>>::new(10, 10));
        let width = img.width();
        let height = img.height();
        let imgdata = img.into_luma16().into_vec();
        let img = SerialImageBuffer::new(
            meta,
            imgdata,
            width as usize,
            height as usize,
            SerialImagePixel::U16(1),
        ).unwrap();
        let img: DynamicSerialImage = img.try_into().unwrap();
        let val = serde_json::to_string(&img).unwrap();
        println!("{}", val);
        let simg: DynamicSerialImage = serde_json::from_str(&val).unwrap();
        assert_eq!(img, simg);
        let dimg = DynamicImage::from(&simg);
        assert_eq!(dimg.width(), width);
        let img = DynamicImage::from(simg);
        assert_eq!(img.width(), width);
        let img = DynamicSerialImage::from(dimg);
        assert_eq!(img.width(), width as usize);
        println!("xxxxxxxxxxxxxxxxxxxxxx");
        print!("\n\n\n");
    }
}
