mod serialimagedata;

pub use serialimagedata::{
    ImageMetaData, SerialImageData, SerialImagePixel, SerialImageStorageTypes, DynamicSerialImage
};

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use image::{DynamicImage, ImageBuffer, Luma};

    use serde_json::{self};

    use crate::{ImageMetaData, SerialImageData, SerialImagePixel, DynamicSerialImage};

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
        let img = DynamicImage::from(ImageBuffer::<Luma<u16>, Vec<u16>>::new(10, 10));
        let width = img.width();
        let height = img.height();
        let imgdata = img.into_luma16().into_vec();
        let img = SerialImageData::new(
            meta,
            imgdata,
            width as usize,
            height as usize,
            SerialImagePixel::U16(1),
        ).unwrap();
        let img: DynamicSerialImage = img.try_into().unwrap();
        let val = serde_json::to_string(&img).unwrap();
        println!("{}", val);
    }
}
