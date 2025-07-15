/*!
 * GNSS模块
 * 
 * 该模块包含GNSS（全球导航卫星系统）相关的类型和函数。
 */

// 导出子模块
pub mod time;
pub mod coord;
pub mod orbit;

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

impl GnssSystem {
    /// 从系统标识符获取GNSS系统
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'G' => Some(Self::Gps),
            'R' => Some(Self::Glonass),
            'E' => Some(Self::Galileo),
            'J' => Some(Self::Qzss),
            'C' => Some(Self::BeiDou),
            'S' => Some(Self::Sbas),
            'I' => Some(Self::NavIc),
            _ => None,
        }
    }
    
    /// 获取系统标识符
    pub fn to_char(&self) -> char {
        match self {
            Self::Gps => 'G',
            Self::Glonass => 'R',
            Self::Galileo => 'E',
            Self::Qzss => 'J',
            Self::BeiDou => 'C',
            Self::Sbas => 'S',
            Self::NavIc => 'I',
        }
    }
    
    /// 获取系统名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Gps => "GPS",
            Self::Glonass => "GLONASS",
            Self::Galileo => "Galileo",
            Self::Qzss => "QZSS",
            Self::BeiDou => "BeiDou",
            Self::Sbas => "SBAS",
            Self::NavIc => "NavIC/IRNSS",
        }
    }
}

/// 卫星ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SatelliteId {
    /// GNSS系统
    pub system: GnssSystem,
    
    /// 卫星PRN编号
    pub prn: u8,
}

impl SatelliteId {
    /// 创建新的卫星ID
    pub fn new(system: GnssSystem, prn: u8) -> Self {
        Self { system, prn }
    }
    
    /// 从卫星编号字符串解析卫星ID
    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() < 2 {
            return None;
        }
        
        let system = GnssSystem::from_char(s.chars().next()?)?;
        let prn = s[1..].parse::<u8>().ok()?;
        
        Some(Self { system, prn })
    }
    
    /// 格式化为字符串
    pub fn to_string(&self) -> String {
        format!("{}{:02}", self.system.to_char(), self.prn)
    }
}

impl std::fmt::Display for SatelliteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
} 