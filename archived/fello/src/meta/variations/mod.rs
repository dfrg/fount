//! Variation axis and named instance collections.

pub mod axis;
pub mod instance;

use crate::setting::Setting;

/// Setting for selecting a user space position on a variation axis.
pub type VariationSetting = Setting<f32>;
