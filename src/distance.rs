const WGS_84_A: f64 = 6_378_137.0;
const WGS_84_B: f64 = 6_356_752.314_2;

fn _earth_radius(sin_latitude: f64, cos_latitude: f64) -> f64 {
    let r1 = WGS_84_A;
    let r2 = WGS_84_B;
    let r1_cos = r1 * cos_latitude;
    let r1_squared_cos = r1 * r1_cos;
    let r2_sin = r2 * sin_latitude;
    let r2_squared_sin = r2 * r2_sin;
    ((r1_squared_cos * r1_squared_cos + r2_squared_sin * r2_squared_sin)
        / (r1_cos * r1_cos + r2_sin * r2_sin))
        .sqrt()
}

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
fn _normalize(a: [f64; 3]) -> [f64; 3] {
    let l = length(a);
    if l == 0.0 {
        return a;
    }
    let k = 1.0 / l;
    [a[0] * k, a[1] * k, a[2] * k]
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

pub fn earth_distance(a: &[i64; 2], b: &[i64; 2]) -> u64 {
    earth_distance_f64(&to_f64(a), &to_f64(b)) as u64
}

pub fn earth_distance_f64(a: &[f64; 2], b: &[f64; 2]) -> f64 {
    const R_AVG: f64 = (WGS_84_A + WGS_84_B) * 0.5;
    let n1 = to_normal_vector(a);
    let n2 = to_normal_vector(b);
    R_AVG * length(cross(n1, n2)).atan2(dot(n1, n2))
}

pub fn orthogonal_distance(a: &[i64; 2], b: &[i64; 2]) -> u64 {
    let dx = a[0].abs_diff(b[0]);
    let dy = a[1].abs_diff(b[1]);
    dx.max(dy)
}

pub fn distance_squared(a: &[i64; 2], b: &[i64; 2]) -> u64 {
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
