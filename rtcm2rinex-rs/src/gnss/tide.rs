// GNSS 潮汐位移修正模块
// 对齐 RTKLIB tides.c:tidedisp 实现，支持固体潮、海洋潮加载、极移潮
// 所有输入输出均为国际单位制，角度为弧度，长度为米

use bitflags::bitflags;
use crate::gnss::coord::{ecef_to_llh};
use std::f64::consts::PI;

/// 地球自转参数（ERP）
#[derive(Debug, Clone, Copy, Default)]
pub struct Erp {
    pub xp: f64,   // 极移x分量（弧秒）
    pub yp: f64,   // 极移y分量（弧秒）
    pub ut1_utc: f64, // UT1-UTC（秒）
    pub lod: f64,  // 长度日（秒）
    pub dpsi: f64, // 黄道章动（弧秒）
    pub deps: f64, // 黄道倾角章动（弧秒）
}

/// 潮汐修正选项，对齐RTKLIB opt参数
bitflags! {
    pub struct TideOpt: u8 {
        const SOLID = 0b0001;   // 固体地球潮
        const OCEAN = 0b0010;   // 海洋潮加载
        const POLE  = 0b0100;   // 极移潮
        const PERM  = 0b1000;   // 消除永久变形
    }
}

/// tidedisp主入口：计算地球潮汐位移修正
///
/// # 参数
/// - time: 观测时刻（UTC，秒）
/// - site_ecef: 站点ECEF坐标 [x, y, z] (米)
/// - opt: 潮汐修正选项（可组合）
/// - erp: 地球自转参数（可选，极移潮/太阳月亮位置用）
/// - odisp: 海洋潮加载参数（可选，66维数组，见RTKLIB注释）
///
/// # 返回
/// - dr: 位移修正量 [dx, dy, dz] (米)
///
/// # 说明
/// 该函数为潮汐修正统一入口，内部自动分派各分项实现。
pub fn tidedisp(
    time: f64,
    site_ecef: [f64; 3],
    opt: TideOpt,
    erp: Option<&Erp>,
    odisp: Option<&[f64; 66]>,
) -> [f64; 3] {
    // 参数基础检查
    if site_ecef.iter().all(|&v| v.abs() < 1e-6) {
        return [0.0, 0.0, 0.0];
    }
    let mut dr = [0.0; 3];
    // 固体地球潮
    if opt.contains(TideOpt::SOLID) {
        let solid = tide_solid(time, site_ecef, opt, erp);
        for i in 0..3 { dr[i] += solid[i]; }
    }
    // 海洋潮加载
    if opt.contains(TideOpt::OCEAN) {
        if let Some(odisp) = odisp {
            let ocean = tide_ocean(time, site_ecef, odisp);
            for i in 0..3 { dr[i] += ocean[i]; }
        }
    }
    // 极移潮
    if opt.contains(TideOpt::POLE) {
        if let Some(erp) = erp {
            let pole = tide_pole(time, site_ecef, erp);
            for i in 0..3 { dr[i] += pole[i]; }
        }
    }
    dr
}

/// 固体地球潮修正（主流程，对齐RTKLIB tides.c: tide_solid）
fn tide_solid(time: f64, site_ecef: [f64; 3], opt: TideOpt, erp: Option<&Erp>) -> [f64; 3] {
    // 1. ECEF -> LLH
    let pos_llh = ecef_to_llh(site_ecef);
    // 2. 计算太阳、月亮在ECEF下的位置和GMST
    let (rsun, rmoon, gmst) = sun_moon_pos_gmst(time, erp);
    // 3. 计算站点局部ENU变换矩阵（此处可直接用coord已有函数）
    // 4. 按RTKLIB公式计算固体地球潮修正量
    // TODO: 详细实现，严格对齐RTKLIB tides.c: tide_solid
    // 目前仅返回零向量
    [0.0, 0.0, 0.0]
}

/// 计算太阳、月亮在ECEF下的位置和GMST（接口预设，后续实现）
fn sun_moon_pos_gmst(_time: f64, _erp: Option<&Erp>) -> ([f64; 3], [f64; 3], f64) {
    // TODO: 实现太阳、月亮位置和GMST计算，对齐RTKLIB sunmoonpos
    ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.0)
}

/// 海洋潮加载修正（主流程，对齐RTKLIB tides.c: tide_oload）
fn tide_ocean(time: f64, site_ecef: [f64; 3], odisp: &[f64; 66]) -> [f64; 3] {
    // 1. ECEF -> LLH
    let pos_llh = ecef_to_llh(site_ecef);
    // 2. 解析观测时刻为历元（年、月、日、时、分、秒）
    let (year, month, day, hour, min, sec) = epoch_from_time(time);
    // 3. 计算天文参数（分潮角等）
    let args = tide_oload_args(year, month, day, hour, min, sec);
    // 4. 叠加各分潮加载效应，输出ENU分量
    // TODO: 详细实现，严格对齐RTKLIB tides.c: tide_oload
    // 目前仅返回零向量
    [0.0, 0.0, 0.0]
}

/// 解析观测时刻为历元（年、月、日、时、分、秒）（接口预设，后续实现）
fn epoch_from_time(_time: f64) -> (i32, i32, i32, i32, i32, f64) {
    // TODO: 实现时间戳到历元转换，对齐RTKLIB time2epoch
    (2000, 1, 1, 0, 0, 0.0)
}

/// 计算海洋潮加载天文参数（接口预设，后续实现）
fn tide_oload_args(_year: i32, _month: i32, _day: i32, _hour: i32, _min: i32, _sec: f64) -> [f64; 11] {
    // TODO: 实现分潮角等天文参数，对齐RTKLIB tide_oload
    [0.0; 11]
}

/// 极移潮修正（主流程，对齐RTKLIB tides.c: tide_pole）
fn tide_pole(time: f64, site_ecef: [f64; 3], erp: &Erp) -> [f64; 3] {
    // 1. ECEF -> LLH
    let pos_llh = ecef_to_llh(site_ecef);
    // 2. 计算IERS平均极移
    let (xp_bar, yp_bar) = iers_mean_pole(time);
    // 3. 计算极移分量
    let m1 = erp.xp - xp_bar; // 单位：弧秒
    let m2 = erp.yp - yp_bar; // 单位：弧秒
    // 4. 按RTKLIB公式计算极移潮ENU分量
    // TODO: 详细实现，严格对齐RTKLIB tides.c: tide_pole
    // 目前仅返回零向量
    [0.0, 0.0, 0.0]
}

/// 计算IERS平均极移（接口预设，后续实现）
fn iers_mean_pole(_time: f64) -> (f64, f64) {
    // TODO: 实现IERS平均极移计算，对齐RTKLIB iers_mean_pole
    (0.0, 0.0)
}

// 单元测试框架预设
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tidedisp_zero() {
        // 零输入应返回零修正
        let dr = tidedisp(0.0, [0.0, 0.0, 0.0], TideOpt::empty(), None, None);
        assert_eq!(dr, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_tidedisp_solid_only() {
        // 仅固体潮分支占位测试
        let dr = tidedisp(1.0, [1000.0, 2000.0, 3000.0], TideOpt::SOLID, None, None);
        assert_eq!(dr, [0.0, 0.0, 0.0]);
    }
} 