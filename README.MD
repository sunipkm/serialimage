# serialimagedata
A `SerialImageData` object encapsulates the generic image types defined in the [`image`](https://crates.io/crates/image) crate, along with any metadata, in order to obtain a portable, pixel format independent image that is serializable.

## Usage
Add the following to your `Cargo.toml`:
```toml
[dependencies]
serialimageadata = "2.0.0"
```
and the following to your source code:
```rs
use serialimageadata::{ImageMetaData, SerialImageData, SerialImagePixel, SerialImageStorageTypes};
```

Then, you can create a new image metadata object:
```rs
let meta = ImageMetaData::new(...);
```

Then, a `SerialImageData` structure can be created from a vector buffer. For example, if the buffer `imgdata` has 8-bit RGB values and its width and height are known,
```rs
let img = SerialImageData::new(meta, imgdata, width, height, SerialImagePixel::U8(3)); // Indicate that the data storage is backed by u8 data, with 3 elements per pixel.
```

## Traits
`SerialImageData` implements the `TryFrom` and `TryInto` traits for `DynamicImage`.