use rgml::prelude::{Real, Vector2};

/// How far texels are "spread" out by MSAA
const SPREAD: Real = 1.0 / 3.0;

/// MSAA sample counts
#[repr(C)]
pub enum MSAASample {
    /// One sample, texel shift is zero
    X1,
    /// Two samples, texels are shifted diagonally by SPREAD
    X2,
    /// Four samples, texels are shifted in a grid by SPREAD
    X4,
    /// Eight samples, texels are shifted in a double layered grid by both SPREAD and SPREAD / 2
    X8,
}

impl MSAASample {
    /// Returns the texel shift factors for the given [MSAASample] type
    pub fn get_offsets(samples: MSAASample) -> Vec<Vector2> {
        return match samples {
            MSAASample::X8 => vec![
                Vector2::new(SPREAD, SPREAD) / 2.0,
                Vector2::new(SPREAD, -SPREAD) / 2.0,
                Vector2::new(-SPREAD, SPREAD) / 2.0,
                Vector2::new(-SPREAD, -SPREAD) / 2.0,
                Vector2::new(SPREAD, SPREAD),
                Vector2::new(SPREAD, -SPREAD),
                Vector2::new(-SPREAD, SPREAD),
                Vector2::new(-SPREAD, -SPREAD),
            ],
            MSAASample::X4 => vec![
                Vector2::new(SPREAD, SPREAD),
                Vector2::new(SPREAD, -SPREAD),
                Vector2::new(-SPREAD, SPREAD),
                Vector2::new(-SPREAD, -SPREAD),
            ],
            MSAASample::X2 => vec![Vector2::from_scalar(SPREAD), Vector2::from_scalar(-SPREAD)],
            _ => vec![Vector2::default()],
        };
    }
}
