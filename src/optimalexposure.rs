#![warn(missing_docs)]
use std::time::Duration;

/// Configuration used to find the optimum exposure.
#[derive(Debug, Clone, Copy)]
pub struct OptimumExposureConfig {
    percentile_pix: f32,
    pixel_tgt: f32,
    pixel_uncertainty: f32,
    min_allowed_exp: Duration,
    max_allowed_exp: Duration,
    max_allowed_bin: u16,
    pixel_exclusion: u32,
}

impl OptimumExposureConfig {
    /// Create a new configuration.
    ///
    /// # Arguments
    ///  * `percentile_pix` - The percentile of the pixel values to use as the target pixel value, in fraction.
    ///  * `pixel_tgt` - The target pixel value, in fraction.
    ///  * `pixel_tol` - The uncertainty of the target pixel value, in fraction.
    ///  * `pixel_exclusion` - The number of pixels to exclude from the top of the image.
    ///  * `min_exposure` - The minimum allowed exposure time.
    ///  * `max_exposure` - The maximum allowed exposure time.
    ///  * `max_bin` - The maximum allowed binning.
    ///
    /// # Returns
    ///  * `Some(OptimumExposureConfig)` - The configuration.
    ///  * `None` - The configuration is invalid.
    pub fn new(
        percentile_pix: f32,
        pixel_tgt: f32,
        pixel_tol: f32,
        pixel_exclusion: u32,
        min_exopsure: Duration,
        max_exposure: Duration,
        max_bin: u16,
    ) -> Option<Self> {
        if min_exopsure >= max_exposure {
            return None;
        }
        Some(Self {
            percentile_pix,
            pixel_tgt,
            pixel_uncertainty: pixel_tol,
            min_allowed_exp: min_exopsure,
            max_allowed_exp: max_exposure,
            max_allowed_bin: max_bin,
            pixel_exclusion,
        })
    }

    /// Find the optimum exposure time and binning to reach a target pixel value.
    /// The algorithm does not use any hysteresis and uses simple scaling.
    ///
    /// # Arguments
    ///  * `mut img` - The image luminance data as a vector of u16 that is consumed.
    ///  * `exposure` - The exposure duration used to obtain this image luminance data.
    ///  * `bin` - The binning used to obtain this image luminance data.
    ///
    /// # Errors
    ///  - Errors are returned as static string slices.
    pub fn find_optimum_exposure(
        &self,
        mut img: Vec<u16>,
        exposure: Duration,
        bin: u8,
    ) -> Result<(Duration, u16), &'static str> {
        let mut target_exposure;

        let mut change_bin = true;

        let pixel_tgt = self.pixel_tgt;
        let pixel_uncertainty = self.pixel_uncertainty;
        let percentile_pix = self.percentile_pix;
        let min_allowed_exp = self.min_allowed_exp;
        let max_allowed_exp = self.max_allowed_exp;
        let max_allowed_bin = self.max_allowed_bin;
        let pixel_exclusion = self.pixel_exclusion;

        if pixel_tgt < 1.6e-5f32 || pixel_tgt > 1f32 {
            return Err("Target pixel value must be between 1.6e-5 and 1");
        }

        if pixel_uncertainty < 1.6e-5f32 || pixel_uncertainty > 1f32 {
            return Err("Pixel uncertainty must be between 1.6e-5 and 1");
        }

        if percentile_pix < 0f32 || percentile_pix > 1f32 {
            return Err("Percentile must be between 0 and 1");
        }

        if min_allowed_exp >= max_allowed_exp {
            return Err("Minimum allowed exposure must be less than maximum allowed exposure");
        }

        if pixel_exclusion > img.len() as u32 {
            return Err("Pixel exclusion must be less than the number of pixels");
        }

        let max_allowed_bin = if max_allowed_bin < 2 {
            1
        } else {
            max_allowed_bin
        };

        let pixel_tgt = pixel_tgt * 65535f32;
        let pixel_uncertainty = pixel_uncertainty * 65535f32;

        if max_allowed_bin < 2 {
            change_bin = false;
        }
        let mut bin = bin as u16;
        img.sort();
        let mut coord: usize;
        if percentile_pix > 0.99999 {
            coord = img.len() - 1 as usize;
        } else {
            coord = (percentile_pix * (img.len() - 1) as f32).floor() as usize;
        }
        if coord < pixel_exclusion as usize {
            coord = img.len() - 1 - pixel_exclusion as usize;
        }
        let imgvec = img.to_vec();
        let val = imgvec.get(coord);
        let val = match val {
            Some(v) => *v as f64,
            None => 1e-5 as f64,
        };

        if (pixel_tgt as f64 - val).abs() < pixel_uncertainty as f64 {
            return Ok((exposure, bin));
        }

        let val = {
            if val <= 1e-5 {
                1e-5
            } else {
                val
            }
        };

        target_exposure = Duration::from_secs_f64(
            (pixel_tgt as f64 * exposure.as_micros() as f64 * 1e-6 / val as f64).abs(),
        );

        if change_bin {
            let mut tgt_exp = target_exposure;
            let mut bin_ = bin;
            if tgt_exp < max_allowed_exp {
                while tgt_exp < max_allowed_exp && bin_ > 2 {
                    bin_ /= 2;
                    tgt_exp *= 4;
                }
            } else {
                while tgt_exp > max_allowed_exp && bin_ * 2 <= max_allowed_bin {
                    bin_ *= 2;
                    tgt_exp /= 4;
                }
            }
            target_exposure = tgt_exp;
            bin = bin_;
        }

        if target_exposure > max_allowed_exp {
            target_exposure = max_allowed_exp;
        }

        if target_exposure < min_allowed_exp {
            target_exposure = min_allowed_exp;
        }

        if bin < 1 {
            bin = 1;
        }
        if bin > max_allowed_bin {
            bin = max_allowed_bin;
        }

        Ok((target_exposure, bin))
    }
}
