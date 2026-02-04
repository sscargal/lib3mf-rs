use serde::{Deserialize, Serialize};

/// Units of measurement for the 3MF model.
///
/// Defines how vertex coordinates and dimensions are interpreted in real-world
/// measurements. The unit applies to all geometric coordinates in the model
/// (vertices, transformations, radii, etc.).
///
/// Per the 3MF specification, the default unit is Millimeter if not specified.
///
/// # Examples
///
/// ```
/// use lib3mf_core::model::Unit;
///
/// // Default is millimeter
/// let unit = Unit::default();
/// assert_eq!(unit, Unit::Millimeter);
///
/// // Convert between units
/// let inches = Unit::Inch.convert(1.0, Unit::Millimeter);
/// assert!((inches - 25.4).abs() < 1e-5);
///
/// // Get scale factor to meters
/// assert_eq!(Unit::Millimeter.scale_factor(), 0.001);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    /// Micrometers - 0.000001 meters (1 μm)
    Micron,
    /// Millimeters - 0.001 meters (1 mm) - Default per 3MF spec
    #[default]
    Millimeter,
    /// Centimeters - 0.01 meters (1 cm)
    Centimeter,
    /// Inches - 0.0254 meters (1 in)
    Inch,
    /// Feet - 0.3048 meters (1 ft)
    Foot,
    /// Meters - 1.0 meters (1 m)
    Meter,
}

impl Unit {
    /// Returns the scale factor to convert this unit to meters.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib3mf_core::model::Unit;
    ///
    /// assert_eq!(Unit::Millimeter.scale_factor(), 0.001);
    /// assert_eq!(Unit::Meter.scale_factor(), 1.0);
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `value` - The value in this unit
    /// * `target` - The unit to convert to
    ///
    /// # Returns
    ///
    /// The value converted to the target unit.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib3mf_core::model::Unit;
    ///
    /// // 1 inch = 25.4 mm
    /// let mm = Unit::Inch.convert(1.0, Unit::Millimeter);
    /// assert!((mm - 25.4).abs() < 1e-5);
    ///
    /// // 1000 mm = 1 meter
    /// let m = Unit::Millimeter.convert(1000.0, Unit::Meter);
    /// assert!((m - 1.0).abs() < 1e-5);
    /// ```
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
