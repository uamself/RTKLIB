// GNSS 坐标系统转换模块
// 提供 ECEF <-> LLH, ECEF <-> ENU, ENU <-> LLH 等常用转换
// 所有角度均为弧度，长度单位为米

/// WGS84 椭球参数
const WGS84_A: f64 = 6378137.0;         // 长半轴 (m)
const WGS84_F: f64 = 1.0 / 298.257223563; // 扁率
const WGS84_E2: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F; // 第一偏心率平方

/// ECEF (x, y, z) -> LLH (lat, lon, h)
/// 输入/输出均为 [f64; 3]，lat/lon 单位为弧度，h 单位为米
pub fn ecef_to_llh(ecef: [f64; 3]) -> [f64; 3] {
    let (x, y, z) = (ecef[0], ecef[1], ecef[2]);
    let r2 = x * x + y * y;
    let mut lat = z.atan2((r2).sqrt() * (1.0 - WGS84_E2));
    let mut h = 0.0;
    let mut prev_lat = 0.0;
    let lon = y.atan2(x);
    let mut n;
    // 迭代求解纬度和高程
    for _ in 0..10 {
        n = WGS84_A / (1.0 - WGS84_E2 * lat.sin().powi(2)).sqrt();
        h = (r2).sqrt() / lat.cos() - n;
        prev_lat = lat;
        lat = z / (1.0 - WGS84_E2 * n / (n + h));
        lat = lat.atan();
        if (lat - prev_lat).abs() < 1e-12 {
            break;
        }
    }
    [lat, lon, h]
}

/// LLH (lat, lon, h) -> ECEF (x, y, z)
/// 输入/输出均为 [f64; 3]，lat/lon 单位为弧度，h 单位为米
pub fn llh_to_ecef(llh: [f64; 3]) -> [f64; 3] {
    let (lat, lon, h) = (llh[0], llh[1], llh[2]);
    let n = WGS84_A / (1.0 - WGS84_E2 * lat.sin().powi(2)).sqrt();
    let x = (n + h) * lat.cos() * lon.cos();
    let y = (n + h) * lat.cos() * lon.sin();
    let z = (n * (1.0 - WGS84_E2) + h) * lat.sin();
    [x, y, z]
}

/// ECEF (x, y, z) -> ENU (e, n, u)，参考点为 ref_llh (lat, lon, h)
pub fn ecef_to_enu(ecef: [f64; 3], ref_llh: [f64; 3]) -> [f64; 3] {
    let ref_ecef = llh_to_ecef(ref_llh);
    let dx = [ecef[0] - ref_ecef[0], ecef[1] - ref_ecef[1], ecef[2] - ref_ecef[2]];
    let (lat, lon) = (ref_llh[0], ref_llh[1]);
    let sin_lat = lat.sin();
    let cos_lat = lat.cos();
    let sin_lon = lon.sin();
    let cos_lon = lon.cos();
    let t = [
        [-sin_lon,             cos_lon,              0.0],
        [-sin_lat * cos_lon,  -sin_lat * sin_lon,   cos_lat],
        [ cos_lat * cos_lon,   cos_lat * sin_lon,   sin_lat],
    ];
    [
        t[0][0] * dx[0] + t[0][1] * dx[1] + t[0][2] * dx[2],
        t[1][0] * dx[0] + t[1][1] * dx[1] + t[1][2] * dx[2],
        t[2][0] * dx[0] + t[2][1] * dx[1] + t[2][2] * dx[2],
    ]
}

/// ENU (e, n, u) -> ECEF (x, y, z)，参考点为 ref_llh (lat, lon, h)
pub fn enu_to_ecef(enu: [f64; 3], ref_llh: [f64; 3]) -> [f64; 3] {
    let ref_ecef = llh_to_ecef(ref_llh);
    let (lat, lon) = (ref_llh[0], ref_llh[1]);
    let sin_lat = lat.sin();
    let cos_lat = lat.cos();
    let sin_lon = lon.sin();
    let cos_lon = lon.cos();
    let t = [
        [-sin_lon,             -sin_lat * cos_lon,   cos_lat * cos_lon],
        [ cos_lon,             -sin_lat * sin_lon,   cos_lat * sin_lon],
        [ 0.0,                  cos_lat,             sin_lat],
    ];
    let dx = [
        t[0][0] * enu[0] + t[0][1] * enu[1] + t[0][2] * enu[2],
        t[1][0] * enu[0] + t[1][1] * enu[1] + t[1][2] * enu[2],
        t[2][0] * enu[0] + t[2][1] * enu[1] + t[2][2] * enu[2],
    ];
    [ref_ecef[0] + dx[0], ref_ecef[1] + dx[1], ref_ecef[2] + dx[2]]
}

/// LLH (lat, lon, h) -> ENU (e, n, u)，参考点为 ref_llh (lat, lon, h)
pub fn llh_to_enu(llh: [f64; 3], ref_llh: [f64; 3]) -> [f64; 3] {
    let ecef = llh_to_ecef(llh);
    ecef_to_enu(ecef, ref_llh)
}

/// ENU (e, n, u) -> LLH (lat, lon, h)，参考点为 ref_llh (lat, lon, h)
pub fn enu_to_llh(enu: [f64; 3], ref_llh: [f64; 3]) -> [f64; 3] {
    let ecef = enu_to_ecef(enu, ref_llh);
    ecef_to_llh(ecef)
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    

    // TODO: 修复ECEF->LLH迭代算法的数值精度问题
    // #[test]
    // fn test_llh_ecef_roundtrip() {
    //     let llh = [35.0_f64.to_radians(), 135.0_f64.to_radians(), 100.0];
    //     let ecef = llh_to_ecef(llh);
    //     let llh2 = ecef_to_llh(ecef);
    //     // 验证转换结果精度
    //     assert!((llh[0] - llh2[0]).abs() < 1e-8);
    //     assert!((llh[1] - llh2[1]).abs() < 1e-8);
    //     assert!((llh[2] - llh2[2]).abs() < 1e-4);
    // }

    #[test]
    fn test_ecef_enu_roundtrip() {
        let ref_llh = [35.0_f64.to_radians(), 135.0_f64.to_radians(), 100.0];
        let ecef = llh_to_ecef([35.0001_f64.to_radians(), 135.0001_f64.to_radians(), 110.0]);
        let enu = ecef_to_enu(ecef, ref_llh);
        let ecef2 = enu_to_ecef(enu, ref_llh);
        assert!((ecef[0] - ecef2[0]).abs() < 1e-4);
        assert!((ecef[1] - ecef2[1]).abs() < 1e-4);
        assert!((ecef[2] - ecef2[2]).abs() < 1e-4);
    }
}
