// GNSS 大气延迟模型模块
// 包含Klobuchar电离层延迟模型和Saastamoinen对流层延迟模型
// 所有角度均为弧度，长度单位为米

use std::f64::consts::PI;

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
    let psi = 0.0137 / (el / PI + 0.11) - 0.022;
    let phi_i = (lat / PI + psi * az.cos()).clamp(-0.416, 0.416) * PI;
    let lam_i = lon + psi * az.sin() / (phi_i / PI).cos();
    let phi_m = phi_i / PI + 0.064 * az.cos() * (lam_i - lon) / PI;
    let t = 43200.0 * lam_i / PI + time;
    let t = t.rem_euclid(86400.0);
    let amp = alpha[0] + alpha[1] * phi_m + alpha[2] * phi_m.powi(2) + alpha[3] * phi_m.powi(3);
    let amp = amp.max(0.0);
    let per = beta[0] + beta[1] * phi_m + beta[2] * phi_m.powi(2) + beta[3] * phi_m.powi(3);
    let per = per.max(72000.0);
    let x = 2.0 * PI * (t - 50400.0) / per;
    let f = 1.0 + 16.0 * (0.53 - el / PI).powi(3);
    let mut delay;
    if x.abs() < 1.57 {
        delay = 5e-9 + amp * (1.0 - x.powi(2) / 2.0 + x.powi(4) / 24.0);
    } else {
        delay = 5e-9;
    }
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
    let p = pres * (1.0 - 0.0000226 * h).powf(5.225);
    let t = temp - 0.0065 * h;
    let e = humi * 6.108 * (-37.2465 + 0.213166 * t - 0.000256908 * t * t).exp();
    let tropo = 0.002277 / zenith.cos() * (p + (1255.0 / t + 0.05) * e - 1.16 * (zenith.tan()).powi(2));
    tropo
}

/// 对流层映射函数（对齐RTKLIB tropmap）
/// 输入：高度角（弧度）
/// 返回：对流层映射系数（无量纲）
pub fn tropmap(el: f64) -> f64 {
    // 简化Niell映射函数（可后续扩展为完整Niell/RTKLIB实现）
    let sin_el = el.sin().max(1e-6);
    1.0 / sin_el
}

/// SBAS/IONEX电离层延迟接口（预设，便于后续扩展）
pub fn sbas_ionodelay(_time: f64, _pos_llh: [f64; 3], _azel: [f64; 2], _sbas_params: &[f64]) -> f64 {
    // TODO: 实现SBAS电离层延迟，对齐RTKLIB sbas_ionocorr
    0.0
}

pub fn ionex_ionodelay(_time: f64, _pos_llh: [f64; 3], _azel: [f64; 2], _ionex_grid: &[f64]) -> f64 {
    // TODO: 实现IONEX电离层延迟，对齐RTKLIB ionocorr_ionex
    0.0
}

/// 对流层/电离层误差估计接口（预设，便于后续扩展）
pub fn tropo_error(_el: f64) -> f64 {
    // TODO: 实现对流层误差估计，对齐RTKLIB tropvar
    0.0
}

pub fn iono_error(_el: f64) -> f64 {
    // TODO: 实现电离层误差估计，对齐RTKLIB ionovar
    0.0
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_klobuchar_ionodelay() {
        let time = 45000.0;
        let pos_llh = [35.0_f64.to_radians(), 135.0_f64.to_radians(), 50.0];
        let azel = [120.0_f64.to_radians(), 30.0_f64.to_radians()];
        let alpha = [0.0, 0.0, 0.0, 0.0];
        let beta = [0.0, 0.0, 0.0, 0.0];
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
} 