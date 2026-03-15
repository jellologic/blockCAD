/// Custom serde serialization for nalgebra Point3 as `[x, y, z]` array.
/// nalgebra's default serializes Point3 as `{"coords": [x, y, z]}` which is ugly in JSON.
///
/// Usage: `#[serde(with = "crate::geometry::serde_helpers::point3_as_array")]`
pub mod point3_as_array {
    use nalgebra::Point3;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(point: &Point3<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [point.x, point.y, point.z].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Point3<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [x, y, z] = <[f64; 3]>::deserialize(deserializer)?;
        Ok(Point3::new(x, y, z))
    }
}

/// Custom serde for Point2 as `[x, y]` array.
pub mod point2_as_array {
    use nalgebra::Point2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(point: &Point2<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [point.x, point.y].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Point2<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [x, y] = <[f64; 2]>::deserialize(deserializer)?;
        Ok(Point2::new(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Point2, Point3};

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestPt3 {
        #[serde(with = "point3_as_array")]
        point: Point3<f64>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestPt2 {
        #[serde(with = "point2_as_array")]
        point: Point2<f64>,
    }

    #[test]
    fn point3_serializes_as_array() {
        let t = TestPt3 {
            point: Point3::new(1.0, 2.0, 3.0),
        };
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"point":[1.0,2.0,3.0]}"#);
    }

    #[test]
    fn point3_roundtrip() {
        let t = TestPt3 {
            point: Point3::new(1.5, -2.5, 0.0),
        };
        let json = serde_json::to_string(&t).unwrap();
        let t2: TestPt3 = serde_json::from_str(&json).unwrap();
        assert_eq!(t, t2);
    }

    #[test]
    fn point2_serializes_as_array() {
        let t = TestPt2 {
            point: Point2::new(1.0, 2.0),
        };
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"point":[1.0,2.0]}"#);
    }
}
