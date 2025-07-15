/*!
 * RINEX格式处理模块
 * 
 * 该模块实现了RINEX格式的生成和解析。
 */

/// RINEX选项
#[derive(Debug, Clone)]
pub struct RinexOptions {
    /// RINEX版本
    version: f64,
    
    /// 观测数据类型
    obs_types: Vec<String>,
    
    /// 导航系统类型
    nav_systems: Vec<char>,
    
    /// 其他选项
    extra_options: std::collections::HashMap<String, String>,
}

/// RINEX错误类型
#[derive(Debug, thiserror::Error)]
pub enum RinexError {
    #[error("Invalid RINEX format")]
    InvalidFormat,
    
    #[error("Unsupported RINEX version: {0}")]
    UnsupportedVersion(f64),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl RinexOptions {
    /// 创建新的RINEX选项
    pub fn new(version: f64) -> Self {
        Self {
            version,
            obs_types: Vec::new(),
            nav_systems: vec!['G'], // 默认GPS系统
            extra_options: std::collections::HashMap::new(),
        }
    }
    
    /// 添加观测数据类型
    pub fn add_obs_type(&mut self, obs_type: &str) {
        self.obs_types.push(obs_type.to_string());
    }
    
    /// 添加导航系统
    pub fn add_nav_system(&mut self, system: char) {
        if !self.nav_systems.contains(&system) {
            self.nav_systems.push(system);
        }
    }
    
    /// 获取RINEX版本
    pub fn get_version(&self) -> f64 {
        self.version
    }
    
    /// 获取导航系统列表
    pub fn get_nav_systems(&self) -> &[char] {
        &self.nav_systems
    }
    
    /// 获取观测数据类型列表
    pub fn get_obs_types(&self) -> &[String] {
        &self.obs_types
    }
}

// 导出子模块
pub mod header;
pub mod obs;
pub mod nav; 