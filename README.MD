# serialimage
This crate extends the [`image`](https://crates.io/crates/image) crate with serializable `DynamicImage`s: the `DynamicSerialImage`s. 
Additionally, it implements an `ImageMetaData` struct to pack additional metadata information.
Note, however, the metadata information is lost on conversion from `DynamicSerialImage` to `DynamicImage`.

The `DynamicSerialImage` struct stores the image data internally in separate channels without additional overhead. 
Similar to the `image` crate, the internal image buffer (`SerialImageBuffer` for `serialimage`) supports 
base data types of `u8`, `u16` and `f32`. `SerialImageBuffer<u8>` and `SerialImageBuffer<u16>` structs 
support both grayscale and RGB images. The `SerialImageBuffer<f32>` struct only supports RGB images. 
Alpha channels are supported for all three types.

Conversions between `image` and `serialimage` data types incur memory copy overheads only when 
the channel count is > 1, i.e. the images are RGB or contain transparency data due to the differences 
in memory layout.


## Usage
Add the following to your `Cargo.toml`:
```toml
[dependencies]
serialimage = "4.1"
```
and the following to your source code:
```rust
use serialimage::{DynamicSerialImage, ImageMetaData};
```

Then, you can create a new image metadata object:
```rust
let meta = ImageMetaData::new(...);
```

Then, a `DynamicSerialImage` can be created from a `DynamicImage`. For example, with a `DynamicImage` 
from a `Luma<u16>` pixel type image buffer,
```rust
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
This feature is not enabled by default, and is available behind the `fitsio` feature flag. This 
feature flag can be enabled to allow for FITS image storage.

Add the following to your `Cargo.toml` to enable this:
```toml
[dependencies]
serialimage = { version = "4.1", features = ["fitsio"] }
```

The FITS I/O is hidden behind a feature flag to avoid compilation errors on `wasm` targets.
