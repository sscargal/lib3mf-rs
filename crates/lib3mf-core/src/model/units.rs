use serde::{Deserialize, Serialize};

/// Units of measurement used in the 3MF model.
///
/// Affects how vertex coordinates are interpreted in real-world dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    /// 0.000001 meters
    Micron,
    /// 0.001 meters (Default)
    #[default]
    Millimeter,
    /// 0.01 meters
    Centimeter,
    /// 0.0254 meters
    Inch,
    /// 0.3048 meters
    Foot,
    /// 1.0 meters
    Meter,
}

impl Unit {
    /// Returns the scale factor to convert this unit to meters.
    pub fn scale_factor(&self) -> f64 {
        match self {
            Unit::Micron => 1e-6,
            Unit::Millimeter => 0.001,
            Unit::Centimeter => 0.01,
            Unit::Inch => 0.0254,
            Unit::Foot => 0.3048,
            Unit::Meter => 1.0,
        }
    }

    /// Converts a value from this unit to another target unit.
    pub fn convert(&self, value: f64, target: Unit) -> f64 {
        if *self == target {
            return value;
        }
        let meters = value * self.scale_factor();
        meters / target.scale_factor()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_factors() {
        assert!((Unit::Millimeter.scale_factor() - 0.001).abs() < 1e-10);
        assert!((Unit::Inch.scale_factor() - 0.0254).abs() < 1e-10);
    }

    #[test]
    fn test_conversion() {
        // 1 inch is 25.4 mm
        let val = Unit::Inch.convert(1.0, Unit::Millimeter);
        assert!((val - 25.4).abs() < 1e-5);

        // 1000 mm is 1 meter
        let val = Unit::Millimeter.convert(1000.0, Unit::Meter);
        assert!((val - 1.0).abs() < 1e-5);
    }
}
