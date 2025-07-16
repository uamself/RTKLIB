// GNSS 轨道计算模块
// 提供开普勒轨道参数到ECEF坐标的转换等基础功能
// 仅实现最基础的Keplerian轨道解算，便于后续扩展

use std::f64::consts::PI;

/// WGS84 地球引力常数 (m^3/s^2)
pub const GM: f64 = 3.986005e14;
/// WGS84 地球自转角速度 (rad/s)
pub const OMEGA_E: f64 = 7.2921151467e-5;

/// 卫星开普勒轨道参数（RINEX星历常用参数）
#[derive(Debug, Clone, Copy)]
pub struct KeplerOrbit {
    pub a: f64,      // 半长轴 (m)
    pub e: f64,      // 偏心率
    pub i0: f64,     // 轨道倾角 (rad)
    pub omega: f64,  // 升交点赤经 (rad)
    pub w: f64,      // 近地点幅角 (rad)
    pub m0: f64,     // 平近点角 (rad)
    pub delta_n: f64,// 平均角速度改正项 (rad/s)
    pub toe: f64,    // 参考历元 (s)
    pub cuc: f64,    // 纬度幅角改正项 (rad)
    pub cus: f64,    // 纬度幅角改正项 (rad)
    pub crc: f64,    // 轨道半径改正项 (m)
    pub crs: f64,    // 轨道半径改正项 (m)
    pub cic: f64,    // 轨道倾角改正项 (rad)
    pub cis: f64,    // 轨道倾角改正项 (rad)
    pub omega_dot: f64, // 升交点赤经变化率 (rad/s)
    pub idot: f64,      // 轨道倾角变化率 (rad/s)
}

/// 多系统卫星类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatSystem {
    GPS,
    GLONASS,
    Galileo,
    BeiDou,
    QZSS,
    SBAS,
    NavIC,
}

/// 卫星星历通用结构体（可扩展多系统参数）
#[derive(Debug, Clone, Copy)]
pub struct Ephemeris {
    pub sys: SatSystem, // 卫星系统
    // GPS/Galileo/BeiDou/QZSS等参数
    pub kepler: Option<KeplerOrbit>,
    // GLONASS参数（可扩展）
    // ...
    // 其他系统参数
    // ...
}

/// 根据Kepler轨道参数和观测时刻，计算卫星在ECEF下的位置 (单位: 米)
/// t: 观测时刻（秒，通常为GPS周内秒）
pub fn kepler_to_ecef(eph: &KeplerOrbit, t: f64) -> [f64; 3] {
    // 1. 计算平均角速度
    let n0 = (GM / eph.a.powi(3)).sqrt();
    let n = n0 + eph.delta_n;
    // 2. 计算平近点角
    let tk = t - eph.toe;
    let mut m = eph.m0 + n * tk;
    m = m.rem_euclid(2.0 * PI);
    // 3. 迭代解开普勒方程，求偏近点角E
    let mut e = m;
    for _ in 0..10 {
        let e_next = m + eph.e * e.sin();
        if (e_next - e).abs() < 1e-12 { break; }
        e = e_next;
    }
    // 4. 计算真近点角v
    let v = ((1.0 - eph.e.powi(2)).sqrt() * e.sin()).atan2(e.cos() - eph.e);
    // 5. 计算辐角u, 轨道半径r, 轨道倾角i
    let phi = v + eph.w;
    let u = phi + eph.cus * phi.sin() + eph.cuc * phi.cos();
    let r = eph.a * (1.0 - eph.e * e.cos()) + eph.crs * phi.sin() + eph.crc * phi.cos();
    let i = eph.i0 + eph.idot * tk + eph.cis * phi.sin() + eph.cic * phi.cos();
    // 6. 升交点赤经
    let omega = eph.omega + (eph.omega_dot - OMEGA_E) * tk - OMEGA_E * eph.toe;
    // 7. 卫星在轨道平面坐标系下的位置
    let x_orb = r * u.cos();
    let y_orb = r * u.sin();
    // 8. 转换到ECEF
    let x = x_orb * omega.cos() - y_orb * omega.sin() * i.cos();
    let y = x_orb * omega.sin() + y_orb * omega.cos() * i.cos();
    let z = y_orb * i.sin();
    [x, y, z]
}

/// 卫星位置、速度、钟差主接口，对齐RTKLIB satpos/satclk
/// 输入：星历、观测时刻（秒）
/// 输出：位置（ECEF, m）、速度（ECEF, m/s）、钟差（秒）
pub fn eph2posvelclk(eph: &Ephemeris, t: f64) -> ([f64; 3], [f64; 3], f64) {
    match eph.sys {
        SatSystem::GPS | SatSystem::Galileo | SatSystem::BeiDou | SatSystem::QZSS => {
            if let Some(kepler) = &eph.kepler {
                let pos = kepler_to_ecef(kepler, t);
                let vel = kepler_to_ecef_vel(kepler, t); // 速度接口预设
                let clk = sat_clock_correction(kepler, t); // 钟差接口预设
                (pos, vel, clk)
            } else {
                ([0.0; 3], [0.0; 3], 0.0)
            }
        }
        SatSystem::GLONASS => {
            // TODO: 实现GLONASS星历解算，对齐RTKLIB
            ([0.0; 3], [0.0; 3], 0.0)
        }
        _ => ([0.0; 3], [0.0; 3], 0.0),
    }
}

/// 卫星速度计算（接口预设，后续实现）
pub fn kepler_to_ecef_vel(_eph: &KeplerOrbit, _t: f64) -> [f64; 3] {
    // TODO: 实现Kepler轨道速度，对齐RTKLIB
    [0.0, 0.0, 0.0]
}

/// 卫星钟差计算（接口预设，后续实现）
pub fn sat_clock_correction(_eph: &KeplerOrbit, _t: f64) -> f64 {
    // TODO: 实现卫星钟差，对齐RTKLIB
    0.0
}

/// 相对论修正（接口预设，后续实现）
pub fn relativity_correction(_eph: &KeplerOrbit, _t: f64) -> f64 {
    // TODO: 实现相对论修正，对齐RTKLIB
    0.0
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_kepler_to_ecef_basic() {
        // 构造一个近似圆形轨道的参数
        let eph = KeplerOrbit {
            a: 26560e3,
            e: 0.01,
            i0: 55.0_f64.to_radians(),
            omega: 1.0,
            w: 0.5,
            m0: 0.0,
            delta_n: 0.0,
            toe: 0.0,
            cuc: 0.0,
            cus: 0.0,
            crc: 0.0,
            crs: 0.0,
            cic: 0.0,
            cis: 0.0,
            omega_dot: 0.0,
            idot: 0.0,
        };
        let t = 3600.0; // 1小时后
        let pos = kepler_to_ecef(&eph, t);
        // 只检查数值范围
        assert!(pos.iter().all(|&v| v.abs() < 3.0e7));
    }
}
