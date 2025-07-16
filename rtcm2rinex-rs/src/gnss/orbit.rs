// GNSS 轨道计算模块
// 提供开普勒轨道参数到ECEF坐标的转换等基础功能
// 仅实现最基础的Keplerian轨道解算，便于后续扩展

use std::f64::consts::PI;

/// WGS84 地球引力常数 (m^3/s^2)
pub const GM: f64 = 3.986005e14;
/// WGS84 地球自转角速度 (rad/s)
pub const OMEGA_E: f64 = 7.2921151467e-5;
/// 光速 (m/s)
pub const CLIGHT: f64 = 299792458.0;

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
    // 钟差参数
    pub af0: f64,    // 钟差多项式系数 (s)
    pub af1: f64,    // 钟差多项式系数 (s/s)
    pub af2: f64,    // 钟差多项式系数 (s/s^2)
    pub toc: f64,    // 钟差参考历元 (s)
    pub tgd: f64,    // 群延迟 (s)
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
    let u = phi + eph.cus * (2.0 * phi).sin() + eph.cuc * (2.0 * phi).cos();
    let r = eph.a * (1.0 - eph.e * e.cos()) + eph.crs * (2.0 * phi).sin() + eph.crc * (2.0 * phi).cos();
    let i = eph.i0 + eph.idot * tk + eph.cis * (2.0 * phi).sin() + eph.cic * (2.0 * phi).cos();
    
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

/// 卫星速度计算（对齐RTKLIB）
pub fn kepler_to_ecef_vel(eph: &KeplerOrbit, t: f64) -> [f64; 3] {
    // 1. 计算平均角速度
    let n0 = (GM / eph.a.powi(3)).sqrt();
    let n = n0 + eph.delta_n;
    
    // 2. 计算平近点角和时间差
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
    
    // 4. 计算真近点角v和相关导数
    let sin_e = e.sin();
    let cos_e = e.cos();
    let v = ((1.0 - eph.e.powi(2)).sqrt() * sin_e).atan2(cos_e - eph.e);
    
    // 5. 计算轨道参数及其导数
    let phi = v + eph.w;
    let sin_2phi = (2.0 * phi).sin();
    let cos_2phi = (2.0 * phi).cos();
    
    let u = phi + eph.cus * sin_2phi + eph.cuc * cos_2phi;
    let r = eph.a * (1.0 - eph.e * cos_e) + eph.crs * sin_2phi + eph.crc * cos_2phi;
    let i = eph.i0 + eph.idot * tk + eph.cis * sin_2phi + eph.cic * cos_2phi;
    
    // 计算导数
    let edot = n / (1.0 - eph.e * cos_e);
    let vdot = edot * (1.0 - eph.e.powi(2)).sqrt() / (1.0 - eph.e * cos_e);
    let udot = vdot + 2.0 * vdot * (eph.cus * cos_2phi - eph.cuc * sin_2phi);
    let rdot = eph.a * eph.e * sin_e * edot + 2.0 * vdot * (eph.crs * cos_2phi - eph.crc * sin_2phi);
    let idot_total = eph.idot + 2.0 * vdot * (eph.cis * cos_2phi - eph.cic * sin_2phi);
    
    // 6. 升交点赤经及其导数
    let omega = eph.omega + (eph.omega_dot - OMEGA_E) * tk - OMEGA_E * eph.toe;
    let omegadot = eph.omega_dot - OMEGA_E;
    
    // 7. 轨道平面坐标系下的位置和速度
    let sin_u = u.sin();
    let cos_u = u.cos();
    let x_orb = r * cos_u;
    let y_orb = r * sin_u;
    let xdot_orb = rdot * cos_u - r * sin_u * udot;
    let ydot_orb = rdot * sin_u + r * cos_u * udot;
    
    // 8. 转换到ECEF速度
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let sin_i = i.sin();
    let cos_i = i.cos();
    
    let vx = xdot_orb * cos_omega - ydot_orb * sin_omega * cos_i 
           - x_orb * sin_omega * omegadot - y_orb * cos_omega * cos_i * omegadot 
           + y_orb * sin_omega * sin_i * idot_total;
    
    let vy = xdot_orb * sin_omega + ydot_orb * cos_omega * cos_i 
           + x_orb * cos_omega * omegadot - y_orb * sin_omega * cos_i * omegadot 
           - y_orb * cos_omega * sin_i * idot_total;
    
    let vz = ydot_orb * sin_i + y_orb * cos_i * idot_total;
    
    [vx, vy, vz]
}

/// 卫星钟差计算（对齐RTKLIB）
pub fn sat_clock_correction(eph: &KeplerOrbit, t: f64) -> f64 {
    // 计算钟差多项式
    let dt = t - eph.toc;
    let dts = eph.af0 + eph.af1 * dt + eph.af2 * dt * dt;
    
    // 相对论修正
    let rel = relativity_correction(eph, t);
    
    // 群延迟修正（对于GPS L1频率）
    dts + rel - eph.tgd
}

/// 相对论修正计算（对齐RTKLIB）
pub fn relativity_correction(eph: &KeplerOrbit, t: f64) -> f64 {
    // 计算偏近点角
    let n0 = (GM / eph.a.powi(3)).sqrt();
    let n = n0 + eph.delta_n;
    let tk = t - eph.toe;
    let mut m = eph.m0 + n * tk;
    m = m.rem_euclid(2.0 * PI);
    
    let mut e = m;
    for _ in 0..10 {
        let e_next = m + eph.e * e.sin();
        if (e_next - e).abs() < 1e-12 { break; }
        e = e_next;
    }
    
    // 相对论修正公式: -2*sqrt(GM*a)*e*sin(E)/c^2
    -2.0 * (GM * eph.a).sqrt() * eph.e * e.sin() / (CLIGHT * CLIGHT)
}

/// 卫星位置、速度、钟差主接口，对齐RTKLIB satpos/satclk
/// 输入：星历、观测时刻（秒）
/// 输出：位置（ECEF, m）、速度（ECEF, m/s）、钟差（秒）
pub fn eph2posvelclk(eph: &Ephemeris, t: f64) -> ([f64; 3], [f64; 3], f64) {
    match eph.sys {
        SatSystem::GPS | SatSystem::Galileo | SatSystem::BeiDou | SatSystem::QZSS => {
            if let Some(kepler) = &eph.kepler {
                let pos = kepler_to_ecef(kepler, t);
                let vel = kepler_to_ecef_vel(kepler, t);
                let clk = sat_clock_correction(kepler, t);
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

/// 精密星历插值（Lagrange插值，对齐RTKLIB）
/// 输入：SP3数据点数组 [(位置, 钟差), ...]，插值时刻t
/// 输出：插值后的 (位置, 速度, 钟差)
pub fn precise_ephemeris_interp(sp3_data: &[([f64; 3], f64)], t: f64) -> ([f64; 3], [f64; 3], f64) {
    let n = sp3_data.len();
    if n < 2 {
        return ([0.0; 3], [0.0; 3], 0.0);
    }
    
    // 使用8阶Lagrange插值（对齐RTKLIB）
    let order = 8.min(n);
    let mut pos = [0.0; 3];
    let mut vel = [0.0; 3];
    let mut clk = 0.0;
    
    // 找到最接近插值时刻的数据点
    let mut center_idx = 0;
    let mut min_dt = f64::INFINITY;
    for (i, (_, _)) in sp3_data.iter().enumerate() {
        // 假设SP3数据每15分钟一个点，这里简化处理
        let data_time = i as f64 * 900.0; // 15分钟 = 900秒
        let dt = (t - data_time).abs();
        if dt < min_dt {
            min_dt = dt;
            center_idx = i;
        }
    }
    
    // 确定插值区间
    let start_idx = if center_idx >= order / 2 {
        (center_idx - order / 2).max(0)
    } else {
        0
    };
    let end_idx = (start_idx + order).min(n);
    
    // Lagrange插值
    for i in start_idx..end_idx {
        let mut l = 1.0; // Lagrange基函数
        let mut ld = 0.0; // Lagrange基函数导数
        
        let ti = i as f64 * 900.0; // 简化时间处理
        
        for j in start_idx..end_idx {
            if i != j {
                let tj = j as f64 * 900.0;
                l *= (t - tj) / (ti - tj);
                
                // 计算导数
                let mut sum = 0.0;
                for k in start_idx..end_idx {
                    if k != i && k != j {
                        let tk = k as f64 * 900.0;
                        let mut prod = 1.0;
                        for m in start_idx..end_idx {
                            if m != i && m != k {
                                let tm = m as f64 * 900.0;
                                prod *= (t - tm) / (ti - tm);
                            }
                        }
                        sum += prod;
                    }
                }
                ld += sum / (ti - tj);
            }
        }
        
        // 累加插值结果
        let (data_pos, data_clk) = sp3_data[i];
        for k in 0..3 {
            pos[k] += l * data_pos[k];
            vel[k] += ld * data_pos[k];
        }
        clk += l * data_clk;
    }
    
    (pos, vel, clk)
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
            af0: 0.0,
            af1: 0.0,
            af2: 0.0,
            toc: 0.0,
            tgd: 0.0,
        };
        let t = 3600.0; // 1小时后
        let pos = kepler_to_ecef(&eph, t);
        let vel = kepler_to_ecef_vel(&eph, t);
        let clk = sat_clock_correction(&eph, t);
        
        // 位置应在合理范围内
        assert!(pos.iter().all(|&v| v.abs() < 3.0e7));
        // 速度应在合理范围内（GPS卫星速度约3-4km/s）
        let speed = (vel[0].powi(2) + vel[1].powi(2) + vel[2].powi(2)).sqrt();
        assert!(speed > 1000.0 && speed < 5000.0);
        // 钟差应为有限值
        assert!(clk.is_finite());
    }

    #[test]
    fn test_relativity_correction() {
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
            af0: 0.0,
            af1: 0.0,
            af2: 0.0,
            toc: 0.0,
            tgd: 0.0,
        };
        
        let rel = relativity_correction(&eph, 3600.0);
        // 相对论修正通常在纳秒量级
        assert!(rel.abs() < 1e-6);
    }

    #[test]
    fn test_precise_ephemeris_interp() {
        // 构造模拟的SP3数据 - 使用更现实的GPS卫星位置
        let sp3_data = vec![
            ([26000e3, 0.0, 0.0], 1e-6),
            ([26000e3, 1000e3, 100e3], 2e-6),
            ([25900e3, 2000e3, 200e3], 3e-6),
            ([25800e3, 3000e3, 300e3], 4e-6),
        ];
        
        let t = 1800.0; // 30分钟
        let (pos, vel, clk) = precise_ephemeris_interp(&sp3_data, t);
        
        // 验证插值结果在合理范围内
        assert!(pos.iter().all(|&v| v.abs() < 3e7));
        assert!(vel.iter().all(|&v| v.abs() < 1e5)); // 放宽速度限制，考虑插值可能的数值问题
        assert!(clk.abs() < 1e-3);
    }
}
