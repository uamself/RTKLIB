/*!
 * 卫星系统支持模块
 * 
 * 为不同的GNSS系统提供专门的信号、导航消息解析和星历解码支持
 */

use crate::gnss::{GnssSystem, orbit::*};
use crate::rinex::nav::*;

/// GPS信号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpsSignal {
    L1CA,   // L1 C/A
    L1P,    // L1 P(Y)
    L2P,    // L2 P(Y)
    L2C,    // L2C
    L5,     // L5
}

/// BeiDou信号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeidouSignal {
    B1I,    // B1I (B1-2)
    B1C,    // B1C (B1-P)
    B2I,    // B2I (B2-2)
    B3I,    // B3I (B3-2)
}

/// Galileo信号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalileoSignal {
    E1B,    // E1-B (I/NAV)
    E1C,    // E1-C 
    E5aI,   // E5a-I (F/NAV)
    E5bI,   // E5b-I (I/NAV)
}

/// QZSS信号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QzssSignal {
    L1CA,   // L1 C/A
    L1C,    // L1C
    L2C,    // L2C
    L5,     // L5
}

/// IRNSS信号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrnssSignal {
    L5,     // L5
    S,      // S-band
}

/// 卫星系统特定的信号枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatelliteSignal {
    Gps(GpsSignal),
    Beidou(BeidouSignal),
    Galileo(GalileoSignal),
    Qzss(QzssSignal),
    Irnss(IrnssSignal),
}

/// 卫星系统错误类型
#[derive(Debug, thiserror::Error)]
pub enum SatelliteSystemError {
    #[error("不支持的信号类型")]
    UnsupportedSignal,
    
    #[error("导航消息解析错误: {0}")]
    NavigationParseError(String),
    
    #[error("星历数据无效")]
    InvalidEphemeris,
    
    #[error("计算错误: {0}")]
    ComputationError(String),
}

/// 卫星系统处理器特征
pub trait SatelliteSystemProcessor {
    /// 计算卫星位置和钟差
    fn compute_satellite_position(&self, eph: &NavData, time: f64) -> Result<([f64; 3], [f64; 3], f64), SatelliteSystemError>;
    
    /// 获取支持的信号列表
    fn get_supported_signals(&self) -> Vec<SatelliteSignal>;
    
    /// 获取系统时间偏移
    fn get_time_offset(&self) -> f64;
}

/// GPS系统处理器
pub struct GpsProcessor;

impl SatelliteSystemProcessor for GpsProcessor {
    fn compute_satellite_position(&self, eph: &NavData, time: f64) -> Result<([f64; 3], [f64; 3], f64), SatelliteSystemError> {
        if let NavData::GPS(gps_eph) = eph {
            let kepler = gps_nav_to_kepler(gps_eph);
            let pos = kepler_to_ecef(&kepler, time);
            let vel = kepler_to_ecef_vel(&kepler, time);
            let clk = gps_clock_correction(gps_eph, time);
            Ok((pos, vel, clk))
        } else {
            Err(SatelliteSystemError::InvalidEphemeris)
        }
    }
    
    fn get_supported_signals(&self) -> Vec<SatelliteSignal> {
        vec![
            SatelliteSignal::Gps(GpsSignal::L1CA),
            SatelliteSignal::Gps(GpsSignal::L1P),
            SatelliteSignal::Gps(GpsSignal::L2P),
            SatelliteSignal::Gps(GpsSignal::L2C),
            SatelliteSignal::Gps(GpsSignal::L5),
        ]
    }
    
    fn get_time_offset(&self) -> f64 {
        0.0 // GPS时间作为参考
    }
}

/// BeiDou系统处理器  
pub struct BeidouProcessor;

impl SatelliteSystemProcessor for BeidouProcessor {
    fn compute_satellite_position(&self, eph: &NavData, time: f64) -> Result<([f64; 3], [f64; 3], f64), SatelliteSystemError> {
        if let NavData::BeiDou(bds_eph) = eph {
            let kepler = beidou_nav_to_kepler(bds_eph);
            let pos = kepler_to_ecef(&kepler, time);
            let vel = kepler_to_ecef_vel(&kepler, time);
            let clk = beidou_clock_correction(bds_eph, time);
            Ok((pos, vel, clk))
        } else {
            Err(SatelliteSystemError::InvalidEphemeris)
        }
    }
    
    fn get_supported_signals(&self) -> Vec<SatelliteSignal> {
        vec![
            SatelliteSignal::Beidou(BeidouSignal::B1I),
            SatelliteSignal::Beidou(BeidouSignal::B1C),
            SatelliteSignal::Beidou(BeidouSignal::B2I),
            SatelliteSignal::Beidou(BeidouSignal::B3I),
        ]
    }
    
    fn get_time_offset(&self) -> f64 {
        14.0 // BeiDou时间相对于GPS时间的偏移
    }
}

/// 将GPS导航数据转换为开普勒轨道参数
fn gps_nav_to_kepler(nav: &GpsNavData) -> KeplerOrbit {
    KeplerOrbit {
        a: nav.sqrt_a * nav.sqrt_a,
        e: nav.e,
        i0: nav.i0,
        omega: nav.omega0,
        w: nav.omega,
        m0: nav.m0,
        delta_n: nav.delta_n,
        toe: nav.toe,
        cuc: nav.cuc,
        cus: nav.cus,
        crc: nav.crc,
        crs: nav.crs,
        cic: nav.cic,
        cis: nav.cis,
        omega_dot: nav.omega_dot,
        idot: nav.idot,
        af0: nav.af0,
        af1: nav.af1,
        af2: nav.af2,
        toc: nav.toe,
        tgd: nav.tgd,
    }
}

/// GPS时钟修正计算
fn gps_clock_correction(nav: &GpsNavData, time: f64) -> f64 {
    let dt = time - nav.toe;
    nav.af0 + nav.af1 * dt + nav.af2 * dt * dt
}

/// 将BeiDou导航数据转换为开普勒轨道参数
fn beidou_nav_to_kepler(nav: &BeidouNavData) -> KeplerOrbit {
    KeplerOrbit {
        a: nav.sqrt_a * nav.sqrt_a,
        e: nav.e,
        i0: nav.i0,
        omega: nav.omega0,
        w: nav.omega,
        m0: nav.m0,
        delta_n: nav.delta_n,
        toe: nav.toe,
        cuc: nav.cuc,
        cus: nav.cus,
        crc: nav.crc,
        crs: nav.crs,
        cic: nav.cic,
        cis: nav.cis,
        omega_dot: nav.omega_dot,
        idot: nav.idot,
        af0: nav.af0,
        af1: nav.af1,
        af2: nav.af2,
        toc: nav.toe,
        tgd: nav.tgd1,
    }
}

/// BeiDou时钟修正计算（包含BDT到GPST的转换）
fn beidou_clock_correction(nav: &BeidouNavData, time: f64) -> f64 {
    let dt = time - nav.toe;
    let bdt_offset = 14.0; // BDT相对于GPST的偏移
    nav.af0 + nav.af1 * dt + nav.af2 * dt * dt - bdt_offset
}

/// Galileo系统处理器
pub struct GalileoProcessor;

impl SatelliteSystemProcessor for GalileoProcessor {
    fn compute_satellite_position(&self, eph: &NavData, time: f64) -> Result<([f64; 3], [f64; 3], f64), SatelliteSystemError> {
        if let NavData::Galileo(gal_eph) = eph {
            let kepler = galileo_nav_to_kepler(gal_eph);
            let pos = kepler_to_ecef(&kepler, time);
            let vel = kepler_to_ecef_vel(&kepler, time);
            let clk = galileo_clock_correction(gal_eph, time);
            Ok((pos, vel, clk))
        } else {
            Err(SatelliteSystemError::InvalidEphemeris)
        }
    }
    
    fn get_supported_signals(&self) -> Vec<SatelliteSignal> {
        vec![
            SatelliteSignal::Galileo(GalileoSignal::E1B),
            SatelliteSignal::Galileo(GalileoSignal::E1C),
            SatelliteSignal::Galileo(GalileoSignal::E5aI),
            SatelliteSignal::Galileo(GalileoSignal::E5bI),
        ]
    }
    
    fn get_time_offset(&self) -> f64 {
        0.0 // Galileo系统时间与GPS时间对齐
    }
}

/// QZSS系统处理器
pub struct QzssProcessor;

impl SatelliteSystemProcessor for QzssProcessor {
    fn compute_satellite_position(&self, eph: &NavData, time: f64) -> Result<([f64; 3], [f64; 3], f64), SatelliteSystemError> {
        if let NavData::QZSS(qzss_eph) = eph {
            let kepler = qzss_nav_to_kepler(qzss_eph);
            let pos = kepler_to_ecef(&kepler, time);
            let vel = kepler_to_ecef_vel(&kepler, time);
            let clk = qzss_clock_correction(qzss_eph, time);
            Ok((pos, vel, clk))
        } else {
            Err(SatelliteSystemError::InvalidEphemeris)
        }
    }
    
    fn get_supported_signals(&self) -> Vec<SatelliteSignal> {
        vec![
            SatelliteSignal::Qzss(QzssSignal::L1CA),
            SatelliteSignal::Qzss(QzssSignal::L1C),
            SatelliteSignal::Qzss(QzssSignal::L2C),
            SatelliteSignal::Qzss(QzssSignal::L5),
        ]
    }
    
    fn get_time_offset(&self) -> f64 {
        0.0 // QZSS时间与GPS时间对齐
    }
}

/// IRNSS系统处理器
pub struct IrnssProcessor;

impl SatelliteSystemProcessor for IrnssProcessor {
    fn compute_satellite_position(&self, eph: &NavData, time: f64) -> Result<([f64; 3], [f64; 3], f64), SatelliteSystemError> {
        if let NavData::IRNSS(irnss_eph) = eph {
            let kepler = irnss_nav_to_kepler(irnss_eph);
            let pos = kepler_to_ecef(&kepler, time);
            let vel = kepler_to_ecef_vel(&kepler, time);
            let clk = irnss_clock_correction(irnss_eph, time);
            Ok((pos, vel, clk))
        } else {
            Err(SatelliteSystemError::InvalidEphemeris)
        }
    }
    
    fn get_supported_signals(&self) -> Vec<SatelliteSignal> {
        vec![
            SatelliteSignal::Irnss(IrnssSignal::L5),
            SatelliteSignal::Irnss(IrnssSignal::S),
        ]
    }
    
    fn get_time_offset(&self) -> f64 {
        0.0 // IRNSS时间与GPS时间对齐
    }
}

/// 卫星系统工厂
pub struct SatelliteSystemFactory;

impl SatelliteSystemFactory {
    /// 创建卫星系统处理器
    pub fn create_processor(system: GnssSystem) -> Box<dyn SatelliteSystemProcessor> {
        match system {
            GnssSystem::Gps => Box::new(GpsProcessor),
            GnssSystem::Glonass => Box::new(GpsProcessor), // 暂用GPS处理器
            GnssSystem::Galileo => Box::new(GalileoProcessor),
            GnssSystem::BeiDou => Box::new(BeidouProcessor),
            GnssSystem::Qzss => Box::new(QzssProcessor),
            GnssSystem::NavIc => Box::new(IrnssProcessor),
            GnssSystem::Sbas => Box::new(GpsProcessor),
        }
    }
}

/// 将Galileo导航数据转换为开普勒轨道参数
fn galileo_nav_to_kepler(nav: &GalileoNavData) -> KeplerOrbit {
    KeplerOrbit {
        a: nav.sqrt_a * nav.sqrt_a,
        e: nav.e,
        i0: nav.i0,
        omega: nav.omega0,
        w: nav.omega,
        m0: nav.m0,
        delta_n: nav.delta_n,
        toe: nav.toe,
        cuc: nav.cuc,
        cus: nav.cus,
        crc: nav.crc,
        crs: nav.crs,
        cic: nav.cic,
        cis: nav.cis,
        omega_dot: nav.omega_dot,
        idot: nav.idot,
        af0: nav.af0,
        af1: nav.af1,
        af2: nav.af2,
        toc: nav.toe,
        tgd: nav.bgd_e5a_e1,
    }
}

/// Galileo时钟修正计算
fn galileo_clock_correction(nav: &GalileoNavData, time: f64) -> f64 {
    let dt = time - nav.toe;
    nav.af0 + nav.af1 * dt + nav.af2 * dt * dt
}

/// 将QZSS导航数据转换为开普勒轨道参数
fn qzss_nav_to_kepler(nav: &QzssNavData) -> KeplerOrbit {
    KeplerOrbit {
        a: nav.sqrt_a * nav.sqrt_a,
        e: nav.e,
        i0: nav.i0,
        omega: nav.omega0,
        w: nav.omega,
        m0: nav.m0,
        delta_n: nav.delta_n,
        toe: nav.toe,
        cuc: nav.cuc,
        cus: nav.cus,
        crc: nav.crc,
        crs: nav.crs,
        cic: nav.cic,
        cis: nav.cis,
        omega_dot: nav.omega_dot,
        idot: nav.idot,
        af0: nav.af0,
        af1: nav.af1,
        af2: nav.af2,
        toc: nav.toe,
        tgd: nav.tgd,
    }
}

/// QZSS时钟修正计算
fn qzss_clock_correction(nav: &QzssNavData, time: f64) -> f64 {
    let dt = time - nav.toe;
    nav.af0 + nav.af1 * dt + nav.af2 * dt * dt
}

/// 将IRNSS导航数据转换为开普勒轨道参数
fn irnss_nav_to_kepler(nav: &IrnssNavData) -> KeplerOrbit {
    KeplerOrbit {
        a: nav.sqrt_a * nav.sqrt_a,
        e: nav.e,
        i0: nav.i0,
        omega: nav.omega0,
        w: nav.omega,
        m0: nav.m0,
        delta_n: nav.delta_n,
        toe: nav.toe,
        cuc: nav.cuc,
        cus: nav.cus,
        crc: nav.crc,
        crs: nav.crs,
        cic: nav.cic,
        cis: nav.cis,
        omega_dot: nav.omega_dot,
        idot: nav.idot,
        af0: nav.af0,
        af1: nav.af1,
        af2: nav.af2,
        toc: nav.toe,
        tgd: nav.tgd,
    }
}

/// IRNSS时钟修正计算
fn irnss_clock_correction(nav: &IrnssNavData, time: f64) -> f64 {
    let dt = time - nav.toe;
    nav.af0 + nav.af1 * dt + nav.af2 * dt * dt
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_gps_processor() {
        let processor = GpsProcessor;
        let signals = processor.get_supported_signals();
        assert!(signals.len() > 0);
        assert_eq!(processor.get_time_offset(), 0.0);
    }

    #[test]
    fn test_beidou_processor() {
        let processor = BeidouProcessor;
        let signals = processor.get_supported_signals();
        assert!(signals.len() > 0);
        assert_eq!(processor.get_time_offset(), 14.0);
    }

    #[test]
    fn test_galileo_processor() {
        let processor = GalileoProcessor;
        let signals = processor.get_supported_signals();
        assert!(signals.len() > 0);
        assert_eq!(processor.get_time_offset(), 0.0);
    }

    #[test]
    fn test_qzss_processor() {
        let processor = QzssProcessor;
        let signals = processor.get_supported_signals();
        assert!(signals.len() > 0);
        assert_eq!(processor.get_time_offset(), 0.0);
    }

    #[test]
    fn test_irnss_processor() {
        let processor = IrnssProcessor;
        let signals = processor.get_supported_signals();
        assert!(signals.len() > 0);
        assert_eq!(processor.get_time_offset(), 0.0);
    }

    #[test]
    fn test_satellite_system_factory() {
        let gps_proc = SatelliteSystemFactory::create_processor(GnssSystem::Gps);
        assert_eq!(gps_proc.get_time_offset(), 0.0);

        let beidou_proc = SatelliteSystemFactory::create_processor(GnssSystem::BeiDou);
        assert_eq!(beidou_proc.get_time_offset(), 14.0);

        let galileo_proc = SatelliteSystemFactory::create_processor(GnssSystem::Galileo);
        assert_eq!(galileo_proc.get_time_offset(), 0.0);
    }

    #[test]
    fn test_signal_types() {
        let gps_l1ca = SatelliteSignal::Gps(GpsSignal::L1CA);
        let beidou_b1i = SatelliteSignal::Beidou(BeidouSignal::B1I);
        let galileo_e1b = SatelliteSignal::Galileo(GalileoSignal::E1B);
        
        // 确保信号类型可以正确匹配
        match gps_l1ca {
            SatelliteSignal::Gps(GpsSignal::L1CA) => assert!(true),
            _ => assert!(false),
        }
        
        match beidou_b1i {
            SatelliteSignal::Beidou(BeidouSignal::B1I) => assert!(true),
            _ => assert!(false),
        }
        
        match galileo_e1b {
            SatelliteSignal::Galileo(GalileoSignal::E1B) => assert!(true),
            _ => assert!(false),
        }
    }
}
