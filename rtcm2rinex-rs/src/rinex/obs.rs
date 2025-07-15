/*!
 * RINEX 观测数据模块
 *
 * 该模块处理RINEX观测数据的生成和解析。
 */

use std::collections::HashMap;
use std::io::{self, Write};
use chrono::{DateTime, Utc, Datelike, Timelike};

use super::RinexError;
use crate::gnss::time::GnssTime;

/// RINEX观测值质量指示符
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RinexObsQuality {
    /// 0: 正常观测值
    Good = 0,
    /// 1: 信号较弱
    Weak = 1,
    /// 2: 无法使用
    Bad = 2,
    /// 5: 通过平滑获得
    Smoothed = 5,
    /// 6: 通过外部精度控制获得
    ExternalQA = 6,
    /// 7: 另一个系统提供的数据
    OtherSystem = 7,
    /// 8: 模拟值
    Simulated = 8,
    /// 9: 不可用
    Unavailable = 9,
}

impl Default for RinexObsQuality {
    fn default() -> Self {
        RinexObsQuality::Good
    }
}

/// RINEX观测值状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RinexObsStatus {
    /// 锁定状态正常
    Normal = 0,
    /// 锁定丢失
    LossOfLock = 1,
    /// 半周模糊
    HalfCycleSlip = 2,
    /// 半周模糊和锁定丢失
    HalfCycleSlipAndLossOfLock = 3,
    /// 未知
    Unknown = 4,
}

impl Default for RinexObsStatus {
    fn default() -> Self {
        RinexObsStatus::Normal
    }
}

/// 周跳变变标记
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CycleSlip {
    /// 是否检测到周跳变
    pub detected: bool,
    /// 周跳变类型
    pub status: RinexObsStatus,
}

impl Default for CycleSlip {
    fn default() -> Self {
        Self {
            detected: false,
            status: RinexObsStatus::Normal,
        }
    }
}

/// 单个观测值
#[derive(Debug, Clone)]
pub struct RinexObservation {
    /// 观测值 (典型单位：C/L为m, D为Hz, S为dBHz)
    pub value: f64,
    /// 观测质量指示符
    pub quality: RinexObsQuality,
    /// 观测状态
    pub status: RinexObsStatus,
    /// 信号强度 (0-9, 0表示最弱信号)
    pub signal_strength: u8,
    /// 周跳变标记
    pub cycle_slip: CycleSlip,
}

impl Default for RinexObservation {
    fn default() -> Self {
        Self {
            value: 0.0,
            quality: RinexObsQuality::default(),
            status: RinexObsStatus::default(),
            signal_strength: 0,
            cycle_slip: CycleSlip::default(),
        }
    }
}

impl RinexObservation {
    /// 创建新的观测值
    pub fn new(value: f64) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
    
    /// 创建空(缺失)的观测值
    pub fn empty() -> Self {
        Self {
            value: 0.0,
            quality: RinexObsQuality::Unavailable,
            ..Default::default()
        }
    }
    
    /// 检查是否为有效观测值
    pub fn is_valid(&self) -> bool {
        self.quality != RinexObsQuality::Unavailable && self.quality != RinexObsQuality::Bad
    }
    
    /// 设置观测质量
    pub fn set_quality(&mut self, quality: RinexObsQuality) -> &mut Self {
        self.quality = quality;
        self
    }
    
    /// 设置信号强度
    pub fn set_signal_strength(&mut self, strength: u8) -> &mut Self {
        self.signal_strength = strength.min(9);
        self
    }
    
    /// 设置状态
    pub fn set_status(&mut self, status: RinexObsStatus) -> &mut Self {
        self.status = status;
        self
    }
    
    /// 设置周跳变
    pub fn set_cycle_slip(&mut self, detected: bool, status: RinexObsStatus) -> &mut Self {
        self.cycle_slip.detected = detected;
        self.cycle_slip.status = status;
        self
    }
    
    /// 获取LLI值（锁定丢失指示符）
    pub fn get_lli(&self) -> u8 {
        if self.cycle_slip.detected {
            self.cycle_slip.status as u8
        } else {
            0
        }
    }
    
    /// 格式化为RINEX 2.x格式的观测值字符串
    pub fn format_v2(&self) -> String {
        if self.is_valid() {
            format!("{:14.3}{:1}{:1}", 
                self.value, 
                if self.cycle_slip.detected { "1" } else { " " }, 
                if self.signal_strength < 1 { " " } else if self.signal_strength < 5 { "1" } else { "5" }
            )
        } else {
            "                ".to_string()
        }
    }
    
    /// 格式化为RINEX 3.x格式的观测值字符串
    pub fn format_v3(&self) -> String {
        if self.is_valid() {
            format!("{:14.3}{:1}{:1}", 
                self.value, 
                if self.cycle_slip.detected { "1" } else { " " }, 
                if self.signal_strength < 1 { " " } else if self.signal_strength < 5 { "1" } else { "5" }
            )
        } else {
            "                ".to_string()
        }
    }
}

/// 单个历元的观测数据
#[derive(Debug, Clone)]
pub struct RinexEpochData {
    /// 观测时间
    pub time: GnssTime,
    /// 卫星PRN号到观测值的映射
    pub observations: HashMap<String, HashMap<String, RinexObservation>>,
    /// 接收机时钟偏移
    pub clock_offset: Option<f64>,
    /// 历元标志
    pub epoch_flag: u8,
}

impl Default for RinexEpochData {
    fn default() -> Self {
        Self {
            time: GnssTime::default(),
            observations: HashMap::new(),
            clock_offset: None,
            epoch_flag: 0,
        }
    }
}

impl RinexEpochData {
    /// 创建新的历元数据
    pub fn new(time: GnssTime) -> Self {
        Self {
            time,
            ..Default::default()
        }
    }
    
    /// 添加观测值
    pub fn add_observation(&mut self, prn: &str, obs_type: &str, observation: RinexObservation) -> &mut Self {
        self.observations
            .entry(prn.to_string())
            .or_insert_with(HashMap::new)
            .insert(obs_type.to_string(), observation);
        self
    }
    
    /// 设置接收机时钟偏移
    pub fn set_clock_offset(&mut self, offset: f64) -> &mut Self {
        self.clock_offset = Some(offset);
        self
    }
    
    /// 获取卫星系统分组
    fn get_satellites_by_system(&self) -> HashMap<char, Vec<String>> {
        let mut result = HashMap::new();
        
        for prn in self.observations.keys() {
            if !prn.is_empty() {
                let sys = prn.chars().next().unwrap();
                result.entry(sys).or_insert_with(Vec::new).push(prn.clone());
            }
        }
        
        // 对每个系统的卫星进行排序
        for sats in result.values_mut() {
            sats.sort();
        }
        
        result
    }
    
    /// 写入RINEX 2.x格式的历元数据
    pub fn write_v2<W: Write>(&self, writer: &mut W, obs_types: &[String]) -> Result<(), RinexError> {
        // 写入历元行
        let time_utc: DateTime<Utc> = self.time.to_datetime();
        write!(writer, " {:2}{:3}{:3}{:3}{:3}{:11.7}{:3}{:>2}",
               time_utc.year() % 100,
               time_utc.month(),
               time_utc.day(),
               time_utc.hour(),
               time_utc.minute(),
               time_utc.second() as f64 + time_utc.nanosecond() as f64 / 1_000_000_000.0,
               self.epoch_flag,
               self.observations.len())?;
        
        // 写入卫星PRN列表
        for (i, prn) in self.observations.keys().enumerate() {
            write!(writer, "{:>3}", prn)?;
            if (i + 1) % 12 == 0 && i + 1 != self.observations.len() {
                writeln!(writer)?;
                write!(writer, "                                ")?;
            }
        }
        writeln!(writer)?;
        
        // 写入接收机时钟偏移(如果有)
        if let Some(offset) = self.clock_offset {
            writeln!(writer, "{:14.12}", offset)?;
        }
        
        // 每颗卫星的观测值
        for prn in self.observations.keys() {
            if let Some(sat_obs) = self.observations.get(prn) {
                // 每种观测类型
                for obs_type in obs_types {
                    if let Some(obs) = sat_obs.get(obs_type) {
                        write!(writer, "{}", obs.format_v2())?;
                    } else {
                        // 没有该类型的观测值
                        write!(writer, "                ")?;
                    }
                }
                writeln!(writer)?;
            }
        }
        
        Ok(())
    }
    
    /// 写入RINEX 3.x格式的历元数据
    pub fn write_v3<W: Write>(&self, writer: &mut W, obs_types_by_system: &HashMap<char, Vec<String>>) -> Result<(), RinexError> {
        // 写入历元行
        let time_utc: DateTime<Utc> = self.time.to_datetime();
        write!(writer, "> {:4} {:02} {:02} {:02} {:02} {:10.7}{:2}{:3}",
               time_utc.year(),
               time_utc.month(),
               time_utc.day(),
               time_utc.hour(),
               time_utc.minute(),
               time_utc.second() as f64 + time_utc.nanosecond() as f64 / 1_000_000_000.0,
               self.epoch_flag,
               self.observations.len())?;
               
        // 写入接收机时钟偏移(如果有)
        if let Some(offset) = self.clock_offset {
            write!(writer, "{:15.12}", offset)?;
        }
        writeln!(writer)?;
        
        // 按卫星系统分组
        let satellites_by_system = self.get_satellites_by_system();
        
        // 每个系统分别写入观测值
        for (sys, satellites) in &satellites_by_system {
            // 获取该系统的观测类型
            if let Some(obs_types) = obs_types_by_system.get(sys) {
                // 每颗卫星
                for prn in satellites {
                    if let Some(sat_obs) = self.observations.get(prn) {
                        // 写入卫星PRN
                        write!(writer, "{}", prn)?;
                        
                        // 每种观测类型
                        for obs_type in obs_types {
                            let full_type = format!("{}{}", sys, obs_type);
                            if let Some(obs) = sat_obs.get(&full_type) {
                                write!(writer, "{}", obs.format_v3())?;
                            } else {
                                // 没有该类型的观测值
                                write!(writer, "                ")?;
                            }
                        }
                        writeln!(writer)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 检测周跳变
    pub fn detect_cycle_slips(&mut self, previous_epoch: &RinexEpochData) {
        for (prn, sat_obs) in &mut self.observations {
            // 获取上一个历元的相同卫星观测值
            if let Some(prev_sat_obs) = previous_epoch.observations.get(prn) {
                for (obs_type, obs) in sat_obs.iter_mut() {
                    // 只处理载波相位观测值
                    if obs_type.starts_with('L') {
                        if let Some(prev_obs) = prev_sat_obs.get(obs_type) {
                            // 简单的周跳变检测 - 如果相位差异超过一定阈值
                            let phase_diff = (obs.value - prev_obs.value).abs();
                            if phase_diff > 10.0 {  // 10周跳变阈值，实际应根据频率等调整
                                obs.set_cycle_slip(true, RinexObsStatus::LossOfLock);
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// 合并多个历元数据
    pub fn merge(&mut self, other: &RinexEpochData) {
        for (prn, sat_obs) in &other.observations {
            for (obs_type, obs) in sat_obs {
                self.add_observation(prn, obs_type, obs.clone());
            }
        }
    }
    
    /// 获取指定卫星和观测类型的观测值
    pub fn get_observation(&self, prn: &str, obs_type: &str) -> Option<&RinexObservation> {
        self.observations.get(prn).and_then(|sat_obs| sat_obs.get(obs_type))
    }
    
    /// 获取观测卫星数量
    pub fn get_satellite_count(&self) -> usize {
        self.observations.len()
    }
    
    /// 获取指定系统的观测卫星数量
    pub fn get_satellite_count_by_system(&self, system: char) -> usize {
        self.observations.keys()
            .filter(|prn| !prn.is_empty() && prn.starts_with(system))
            .count()
    }
} 