const WGS_84_A: f64 = 6_378_137.0;
const WGS_84_B: f64 = 6_356_752.314_2;

#[inline]
fn to_normal_vector(location: &[f64; 2]) -> [f64; 3] {
    // https://en.wikipedia.org/wiki/N-vector
    let longitude = location[0];
    let latitude = location[1];
    let (sin_longitude, cos_longitude) = longitude.to_radians().sin_cos();
    let (sin_latitude, cos_latitude) = latitude.to_radians().sin_cos();
    [
        cos_latitude * cos_longitude,
        cos_latitude * sin_longitude,
        sin_latitude,
    ]
}

#[inline]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1], //
        a[2] * b[0] - a[0] * b[2], //
        a[0] * b[1] - a[1] * b[0], //
    ]
}

#[inline]
fn length(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn to_f64(location: &[i64; 2]) -> [f64; 2] {
    let longitude = location[0] as f64 * 1e-9;
    let latitude = location[1] as f64 * 1e-9;
    [longitude, latitude]
}

/// Returns geographical distance between two points.
///
/// The first coordinate is longitude, the second coordinate is latitude.
/// Both are in nanodegrees.
///
/// The distance is computed by converting each point to surface normals.
///
/// # References
///
/// - <https://en.wikipedia.org/wiki/N-vector>
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub fn earth_distance(a: &[i64; 2], b: &[i64; 2]) -> u64 {
    earth_distance_f64(&to_f64(a), &to_f64(b)) as u64
}

pub(crate) fn earth_distance_f64(a: &[f64; 2], b: &[f64; 2]) -> f64 {
    const R_AVG: f64 = (WGS_84_A + WGS_84_B) * 0.5;
    let n1 = to_normal_vector(a);
    let n2 = to_normal_vector(b);
    R_AVG * length(cross(n1, n2)).atan2(dot(n1, n2))
}
