#[cfg(feature = "std")]
mod earth;

#[cfg(feature = "std")]
pub use self::earth::*;

/// Returns maximum distance between two points computed along each axis individually,
/// i.e. $ \mathrm{max}\left(\left|x_1 - x_0\right|, \left|y_1 - y_0\right|\right) $.
pub fn orthogonal_distance(a: &[i64; 2], b: &[i64; 2]) -> u64 {
    let dx = a[0].abs_diff(b[0]);
    let dy = a[1].abs_diff(b[1]);
    dx.max(dy)
}

/// Returns squared Euclidean distance between two points.
pub fn euclidean_distance_squared(a: &[i64; 2], b: &[i64; 2]) -> u64 {
    let dx_squared = distance_squared_scalar(a[0], b[0]);
    let dy_squared = distance_squared_scalar(a[1], b[1]);
    dx_squared.saturating_add(dy_squared)
}

fn distance_squared_scalar(a: i64, b: i64) -> u64 {
    let dx = a.abs_diff(b);
    dx.saturating_mul(dx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earth_distance_works() {
        let d = earth_distance_f64(&[-77.0366, 38.8976], &[-75.1503, 39.9496]);
        assert!((d - 200_000.0).abs() < 1000.0, "d = {d}");
    }
}
