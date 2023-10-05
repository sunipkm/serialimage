#![deny(missing_docs)]
use std::{
    fmt::Display,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};


#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
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