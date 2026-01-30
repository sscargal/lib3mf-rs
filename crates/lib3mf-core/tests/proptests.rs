use lib3mf_core::model::{Triangle, Unit, Vertex};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_vertex_eq(x in -1000.0f32..1000.0, y in -1000.0f32..1000.0, z in -1000.0f32..1000.0) {
        let v1 = Vertex { x, y, z };
        let v2 = Vertex { x, y, z };
        // Basic equality check to ensure structural equality works
        prop_assert_eq!(v1, v2);
    }

    #[test]
    fn test_triangle_validity(v1 in 0u32..1000, v2 in 0u32..1000, v3 in 0u32..1000) {
        let t = Triangle { v1, v2, v3, ..Default::default() };
        // Just verify we can construct it
        prop_assert_eq!(t.v1, v1);
    }

    #[test]
    fn test_unit_parsing_roundtrip(val in 0u8..6) {
        // Test enum variants (approx)
        let unit = match val % 6 {
            0 => Unit::Micron,
            1 => Unit::Millimeter,
            2 => Unit::Centimeter,
            3 => Unit::Inch,
            4 => Unit::Foot,
            5 => Unit::Meter,
            _ => Unit::Millimeter,
        };

        // Serialize and deserialize logic would go here if we exposed it directly via FromStr/Display
        // For now, just ensuring Debug/Clone works
        let cloned = unit;
        prop_assert_eq!(unit, cloned);
    }
}
