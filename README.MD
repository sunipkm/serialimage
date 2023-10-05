# serialimage
This crate extends the [`image`](https://crates.io/crates/image) crate with serializable `DynamicImage`s: the `DynamicSerialImage`s. Additionally, it implements an `ImageMetaData` struct to pack additional metadata information.
Note, however, the metadata information is lost on conversion from `DynamicSerialImage` to `DynamicImage`.

## Usage
Add the following to your `Cargo.toml`:
```toml
[dependencies]
serialimage = "2.0.0"
```
and the following to your source code:
```rs
use serialimage::{DynamicSerialImage, ImageMetaData};
```

Then, you can create a new image metadata object:
```rs
let meta = ImageMetaData::new(...);
```

Then, a `DynamicSerialImage` can be created from a `DynamicImage`. For example, with a `DynamicImage` from a `Luma<u16>` pixel type image buffer,
```rs
let img = DynamicImage::from(ImageBuffer::<Luma<u16>, Vec<u16>>::new(10, 10)); // create DynamicImage
let mut img = DynamicSerialImage::from(img); // create DynamicSerialImage
img.set_metadata(meta); // set the metadata
let imgstr = serde_json::to_string(&img).unwrap(); // serialize
let simg: DynamicSerialImage = serde_json::from_str(&imgstr).unwrap(); // deserialize
assert_eq!(img, simg);
```
Now `img` can be sent on its merry way with full serialization

## Traits
`DynamicSerialImage` and `SerialImageBuffer` implements the `TryFrom` and `TryInto` traits for `image::DynamicImage` and `image::SerialImageBuffer`.

## Optional Features
Enable the `fitsio` feature to save `DynamicSerialImage` structures as `FITS` images using the `DynamicSerialImage::savefits()` function.

Add the following to your `Cargo.toml` to enable this:
```toml
[dependencies]
serialimage = { version = "2.0.0", features = ["fitsio"] }
```
