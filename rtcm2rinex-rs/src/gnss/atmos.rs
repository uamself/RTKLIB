// GNSS 大气延迟模型模块
// 包含Klobuchar电离层延迟模型和Saastamoinen对流层延迟模型
// 所有角度均为弧度，长度单位为米

use std::f64::consts::PI;

/// IONEX网格数据结构
#[derive(Debug, Clone)]
pub struct IonexGrid {
    pub lat_min: f64,    // 最小纬度（弧度）
    pub lat_max: f64,    // 最大纬度（弧度）
    pub lon_min: f64,    // 最小经度（弧度）
    pub lon_max: f64,    // 最大经度（弧度）
    pub dlat: f64,       // 纬度间隔（弧度）
    pub dlon: f64,       // 经度间隔（弧度）
    pub height: f64,     // 电离层薄壳高度（米）
    pub tec: Vec<Vec<f64>>, // TEC网格数据 [lat_idx][lon_idx]
}

/// SBAS电离层参数
#[derive(Debug, Clone)]
pub struct SbasIonoParams {
    pub igp_mask: Vec<bool>,  // IGP掩码
    pub igp_delay: Vec<f64>,  // IGP延迟
    pub givei: Vec<u8>,       // GIVEI指标
}

/// Klobuchar电离层延迟模型
/// 输入：
///   time: GPS周内秒
///   pos_llh: 接收机位置 [lat, lon, h]（弧度、弧度、米）
///   azel: [方位角, 高度角]（弧度）
///   alpha, beta: 电离层参数（RINEX导航文件头部，4个元素）
/// 返回：电离层延迟（米）
pub fn klobuchar_ionodelay(time: f64, pos_llh: [f64; 3], azel: [f64; 2], alpha: [f64; 4], beta: [f64; 4]) -> f64 {
    let (lat, lon) = (pos_llh[0], pos_llh[1]);
    let (az, el) = (azel[0], azel[1]);
    
    // 地心角
    let psi = 0.0137 / (el / PI + 0.11) - 0.022;
    
    // 穿刺点地理坐标
    let phi_i = (lat / PI + psi * az.cos()).clamp(-0.416, 0.416) * PI;
    let lam_i = lon + psi * az.sin() / (phi_i / PI).cos();
    
    // 地磁纬度
    let phi_m = phi_i / PI + 0.064 * (lam_i - 1.617).cos();
    
    // 地方时
    let t = 43200.0 * lam_i / PI + time;
    let t = t.rem_euclid(86400.0);
    
    // 振幅
    let amp = alpha[0] + alpha[1] * phi_m + alpha[2] * phi_m.powi(2) + alpha[3] * phi_m.powi(3);
    let amp = amp.max(0.0);
    
    // 周期
    let per = beta[0] + beta[1] * phi_m + beta[2] * phi_m.powi(2) + beta[3] * phi_m.powi(3);
    let per = per.max(72000.0);
    
    // 相位
    let x = 2.0 * PI * (t - 50400.0) / per;
    
    // 倾斜因子
    let f = 1.0 + 16.0 * (0.53 - el / PI).powi(3);
    
    // 计算延迟
    let delay = if x.abs() < 1.57 {
        5e-9 + amp * (1.0 - x.powi(2) / 2.0 + x.powi(4) / 24.0)
    } else {
        5e-9
    };
    
    delay * f * 299792458.0 // 转换为米
}

/// Saastamoinen对流层延迟模型
/// 输入：
///   pos_llh: 接收机位置 [lat, lon, h]（弧度、弧度、米）
///   el: 卫星高度角（弧度）
///   pres: 气压（hPa）
///   temp: 温度（K）
///   humi: 相对湿度（0~1）
/// 返回：对流层延迟（米）
pub fn saastamoinen_tropodelay(pos_llh: [f64; 3], el: f64, pres: f64, temp: f64, humi: f64) -> f64 {
    let h = pos_llh[2].max(0.0);
    let zenith = PI / 2.0 - el;
    
    // 高度修正
    let p = pres * (1.0 - 0.0000226 * h).powf(5.225);
    let t = temp - 0.0065 * h;
    
    // 水汽压
    let e = humi * 6.108 * (-37.2465 + 0.213166 * t - 0.000256908 * t * t).exp();
    
    // Saastamoinen公式
    let tropo = 0.002277 / zenith.cos() * (p + (1255.0 / t + 0.05) * e - 1.16 * (zenith.tan()).powi(2));
    tropo
}

/// Niell对流层映射函数（对齐RTKLIB）
/// 输入：
///   pos_llh: 接收机位置 [lat, lon, h]（弧度、弧度、米）
///   el: 卫星高度角（弧度）
///   doy: 年积日
/// 返回：[干延迟映射因子, 湿延迟映射因子]
pub fn niell_tropmap(pos_llh: [f64; 3], el: f64, doy: f64) -> [f64; 2] {
    let lat = pos_llh[0].abs();
    let h = pos_llh[2].max(0.0);
    
    // 纬度和季节变化系数
    let a_avg = [1.2769934e-3, 1.2683230e-3, 1.2465397e-3, 1.2196049e-3, 1.2045996e-3];
    let b_avg = [2.9153695e-3, 2.9152299e-3, 2.9288445e-3, 2.9022565e-3, 2.9024912e-3];
    let c_avg = [62.610505e-3, 62.837393e-3, 63.721774e-3, 63.824265e-3, 64.258455e-3];
    
    let a_amp = [0.0, 1.2709626e-5, 2.6523662e-5, 3.4000452e-5, 4.1202191e-5];
    let b_amp = [0.0, 2.1414979e-5, 3.0160779e-5, 7.2562722e-5, 11.723375e-5];
    let c_amp = [0.0, 9.0128400e-5, 4.3497037e-5, 84.795348e-5, 170.37206e-5];
    
    // 插值系数
    let latd = lat * 180.0 / PI;
    let mut i = 0;
    if latd >= 15.0 { i = 1; }
    if latd >= 30.0 { i = 2; }
    if latd >= 45.0 { i = 3; }
    if latd >= 60.0 { i = 4; }
    
    let cosy = (doy / 365.25 * 2.0 * PI).cos();
    
    // 干延迟映射函数参数
    let aht = a_avg[i] - a_amp[i] * cosy;
    let bht = b_avg[i] - b_amp[i] * cosy;
    let cht = c_avg[i] - c_amp[i] * cosy;
    
    let sine = el.sin();
    let beta = bht / (sine + cht);
    let gamma = aht / (sine + beta);
    let topcon = (1.0 + aht / (1.0 + bht / (1.0 + cht)));
    
    let mf_h = topcon / (sine + gamma);
    
    // 湿延迟映射函数（简化）
    let mf_w = 1.0 / (sine + 0.00143 / (sine + 0.0445));
    
    // 高度修正
    let ht_corr = 1.0 / sine - 1.0 / (sine + 0.0032);
    let ht_corr_coeff = 1.0 + 1e-6 * (h - 0.0) * ht_corr;
    
    [mf_h * ht_corr_coeff, mf_w]
}

/// 简化对流层映射函数（对齐RTKLIB tropmap）
/// 输入：高度角（弧度）
/// 返回：对流层映射系数（无量纲）
pub fn tropmap(el: f64) -> f64 {
    // 简化映射函数
    let sin_el = el.sin().max(1e-6);
    1.0 / sin_el
}

/// IONEX电离层延迟计算（网格插值）
pub fn ionex_ionodelay(time: f64, pos_llh: [f64; 3], azel: [f64; 2], ionex_grid: &IonexGrid) -> f64 {
    let (lat, lon) = (pos_llh[0], pos_llh[1]);
    let el = azel[1];
    
    // 计算穿刺点坐标（假设电离层薄壳高度）
    let re = 6371000.0; // 地球半径
    let hion = ionex_grid.height;
    let psi = PI / 2.0 - el - ((re / (re + hion)) * (PI / 2.0 - el).cos()).asin();
    let lat_pp = (lat.sin() * psi.cos() + lat.cos() * psi.sin() * azel[0].cos()).asin();
    let lon_pp = lon + (psi.sin() * azel[0].sin() / lat_pp.cos()).asin();
    
    // 网格插值
    let lat_idx = ((lat_pp - ionex_grid.lat_min) / ionex_grid.dlat).floor() as usize;
    let lon_idx = ((lon_pp - ionex_grid.lon_min) / ionex_grid.dlon).floor() as usize;
    
    if lat_idx >= ionex_grid.tec.len() || lat_idx + 1 >= ionex_grid.tec.len() ||
       lon_idx >= ionex_grid.tec[0].len() || lon_idx + 1 >= ionex_grid.tec[0].len() {
        return 0.0;
    }
    
    // 双线性插值
    let dlat_frac = (lat_pp - ionex_grid.lat_min) / ionex_grid.dlat - lat_idx as f64;
    let dlon_frac = (lon_pp - ionex_grid.lon_min) / ionex_grid.dlon - lon_idx as f64;
    
    let tec_00 = ionex_grid.tec[lat_idx][lon_idx];
    let tec_01 = ionex_grid.tec[lat_idx][lon_idx + 1];
    let tec_10 = ionex_grid.tec[lat_idx + 1][lon_idx];
    let tec_11 = ionex_grid.tec[lat_idx + 1][lon_idx + 1];
    
    let tec_interp = tec_00 * (1.0 - dlat_frac) * (1.0 - dlon_frac) +
                     tec_01 * (1.0 - dlat_frac) * dlon_frac +
                     tec_10 * dlat_frac * (1.0 - dlon_frac) +
                     tec_11 * dlat_frac * dlon_frac;
    
    // 倾斜因子
    let slant_factor = 1.0 / (1.0 - ((re / (re + hion)) * (PI / 2.0 - el).cos()).powi(2)).sqrt();
    
    // TEC转延迟（GPS L1频率：40.3 * TEC / f^2）
    let f1 = 1575.42e6; // GPS L1频率
    40.3 * tec_interp * slant_factor / (f1 * f1) * 299792458.0
}

/// SBAS电离层延迟校正
pub fn sbas_ionodelay(time: f64, pos_llh: [f64; 3], azel: [f64; 2], sbas_params: &SbasIonoParams) -> f64 {
    let (lat, lon) = (pos_llh[0], pos_llh[1]);
    let el = azel[1];
    
    // 计算穿刺点坐标（350km高度）
    let re = 6371000.0;
    let hion = 350000.0;
    let psi = PI / 2.0 - el - ((re / (re + hion)) * (PI / 2.0 - el).cos()).asin();
    let lat_pp = (lat.sin() * psi.cos() + lat.cos() * psi.sin() * azel[0].cos()).asin();
    let lon_pp = lon + (psi.sin() * azel[0].sin() / lat_pp.cos()).asin();
    
    // IGP网格索引（简化计算）
    let lat_deg = lat_pp * 180.0 / PI;
    let lon_deg = lon_pp * 180.0 / PI;
    
    // 简化的IGP插值（实际应根据SBAS标准实现）
    let igp_idx = ((lat_deg + 55.0) / 5.0) as usize * 72 + ((lon_deg + 180.0) / 5.0) as usize;
    
    if igp_idx >= sbas_params.igp_delay.len() || !sbas_params.igp_mask[igp_idx] {
        return 0.0;
    }
    
    // 倾斜因子
    let slant_factor = 1.0 / (1.0 - ((re / (re + hion)) * (PI / 2.0 - el).cos()).powi(2)).sqrt();
    
    sbas_params.igp_delay[igp_idx] * slant_factor
}

/// 对流层延迟误差估计（对齐RTKLIB tropvar）
pub fn tropo_error(el: f64) -> f64 {
    // 基于高度角的对流层误差模型
    let sin_el = el.sin().max(0.1);
    let var_h = (0.12 * 1.001 / sin_el.sqrt()).powi(2); // 干延迟误差
    let var_w = (0.1 / sin_el.sqrt()).powi(2);           // 湿延迟误差
    (var_h + var_w).sqrt()
}

/// 电离层延迟误差估计（对齐RTKLIB ionovar）
pub fn iono_error(el: f64) -> f64 {
    // 基于高度角的电离层误差模型
    let sin_el = el.sin().max(0.1);
    let var = (5.0 / sin_el).powi(2); // 5m基础误差
    var.sqrt()
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_klobuchar_ionodelay() {
        let time = 45000.0;
        let pos_llh = [35.0_f64.to_radians(), 135.0_f64.to_radians(), 50.0];
        let azel = [120.0_f64.to_radians(), 30.0_f64.to_radians()];
        let alpha = [0.1118e-7, -0.7451e-8, -0.5960e-7, 0.1192e-6];
        let beta = [0.1167e6, -0.2294e6, -0.1311e6, 0.1049e7];
        let delay = klobuchar_ionodelay(time, pos_llh, azel, alpha, beta);
        assert!(delay > 0.0 && delay < 100.0);
    }

    #[test]
    fn test_saastamoinen_tropodelay() {
        let pos_llh = [35.0_f64.to_radians(), 135.0_f64.to_radians(), 100.0];
        let el = 30.0_f64.to_radians();
        let pres = 1013.25;
        let temp = 293.15;
        let humi = 0.5;
        let delay = saastamoinen_tropodelay(pos_llh, el, pres, temp, humi);
        assert!(delay > 1.0 && delay < 10.0);
    }

    #[test]
    fn test_niell_tropmap() {
        let pos_llh = [35.0_f64.to_radians(), 135.0_f64.to_radians(), 100.0];
        let el = 30.0_f64.to_radians();
        let doy = 100.0;
        let mf = niell_tropmap(pos_llh, el, doy);
        assert!(mf[0] > 1.0 && mf[0] < 10.0); // 干延迟映射因子
        assert!(mf[1] > 1.0 && mf[1] < 10.0); // 湿延迟映射因子
    }

    #[test]
    fn test_error_models() {
        let el = 30.0_f64.to_radians();
        let tropo_err = tropo_error(el);
        let iono_err = iono_error(el);
        
        assert!(tropo_err > 0.0 && tropo_err < 1.0);
        assert!(iono_err > 0.0 && iono_err < 50.0);
    }
} 