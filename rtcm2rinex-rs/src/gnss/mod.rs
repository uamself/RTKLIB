/*!
 * GNSS模块
 * 
 * 该模块包含GNSS（全球导航卫星系统）相关的类型和函数。
 */

// 导出子模块
pub mod time;
pub mod coord;
pub mod orbit;
pub mod tide;
pub mod atmos;

// 重新导出常用类型
pub use self::time::GnssTime;

/// GNSS系统类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GnssSystem {
    /// GPS
    Gps = 0,
    
    /// GLONASS
    Glonass = 1,
    
    /// Galileo
    Galileo = 2,
    
    /// QZSS
    Qzss = 3,
    
    /// BeiDou
    BeiDou = 4,
    
    /// SBAS
    Sbas = 5,
    
    /// NavIC/IRNSS
    NavIc = 6,
}
