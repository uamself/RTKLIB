/*!
 * RINEX 头部模块
 *
 * 该模块处理RINEX文件头部的生成和解析。
 */

use std::fmt::{self, Display, Formatter};
use std::io::Write;
use chrono::{DateTime, Utc, Datelike, Timelike};
use std::collections::HashMap;

use super::RinexError;

/// RINEX系统类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RinexSystem {
    /// GPS
    GPS,
    /// GLONASS
    GLONASS,
    /// Galileo
    Galileo,
    /// SBAS
    SBAS,
    /// QZSS
    QZSS,
    /// BeiDou
    BeiDou,
    /// IRNSS
    IRNSS,
    /// 混合
    Mixed,
}

impl Display for RinexSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RinexSystem::GPS => write!(f, "G"),
            RinexSystem::GLONASS => write!(f, "R"),
            RinexSystem::Galileo => write!(f, "E"),
            RinexSystem::SBAS => write!(f, "S"),
            RinexSystem::QZSS => write!(f, "J"),
            RinexSystem::BeiDou => write!(f, "C"),
            RinexSystem::IRNSS => write!(f, "I"),
            RinexSystem::Mixed => write!(f, "M"),
        }
    }
}

/// RINEX文件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RinexFileType {
    /// 观测数据
    Observation,
    /// 导航数据
    Navigation,
    /// 气象数据
    Meteorological,
    /// 时钟数据
    Clock,
}

impl Display for RinexFileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RinexFileType::Observation => write!(f, "O"),
            RinexFileType::Navigation => write!(f, "N"),
            RinexFileType::Meteorological => write!(f, "M"),
            RinexFileType::Clock => write!(f, "C"),
        }
    }
}

/// 电离层模型参数
#[derive(Debug, Clone)]
pub struct IonoParams {
    /// 系统类型
    pub system: RinexSystem,
    /// 电离层参数 (GPS: alpha0-3, beta0-3; Galileo: ai0-2, ai1-2, etc.)
    pub params: Vec<f64>,
}

/// 时间系统参数
#[derive(Debug, Clone)]
pub struct TimeSystemParams {
    /// 系统类型
    pub system: RinexSystem,
    /// 时间系统参数 (A0, A1, T, W, etc.)
    pub params: Vec<f64>,
    /// 参考时间 (如果有)
    pub ref_time: Option<DateTime<Utc>>,
    /// 参考系统 (如果有)
    pub ref_system: Option<String>,
}

/// GLONASS频率通道
#[derive(Debug, Clone)]
pub struct GlonassFreqChannel {
    /// 卫星PRN号
    pub prn: u8,
    /// 频率通道号
    pub freq_num: i8,
}

/// 相位中心偏移
#[derive(Debug, Clone)]
pub struct PhaseCenterOffset {
    /// 系统和信号类型 (如 "G01")
    pub signal: String,
    /// 北向偏移 (m)
    pub north: f64,
    /// 东向偏移 (m)
    pub east: f64,
    /// 上向偏移 (m)
    pub up: f64,
}

/// RINEX头部结构
#[derive(Debug, Clone)]
pub struct RinexHeader {
    /// RINEX版本
    pub version: f64,
    /// 文件类型
    pub file_type: RinexFileType,
    /// 卫星系统
    pub satellite_system: RinexSystem,
    /// 程序名称
    pub program: String,
    /// 机构名称
    pub run_by: String,
    /// 创建日期
    pub date_created: DateTime<Utc>,
    /// 标记名称
    pub marker_name: String,
    /// 标记编号
    pub marker_number: Option<String>,
    /// 观测者
    pub observer: String,
    /// 机构
    pub agency: String,
    /// 接收机编号
    pub receiver_number: String,
    /// 接收机类型
    pub receiver_type: String,
    /// 接收机版本
    pub receiver_version: String,
    /// 天线编号
    pub antenna_number: String,
    /// 天线类型
    pub antenna_type: String,
    /// 近似位置X
    pub approx_position_x: f64,
    /// 近似位置Y
    pub approx_position_y: f64,
    /// 近似位置Z
    pub approx_position_z: f64,
    /// 天线高度H
    pub antenna_height: f64,
    /// 天线偏移E
    pub antenna_e: f64,
    /// 天线偏移N
    pub antenna_n: f64,
    /// 观测类型
    pub obs_types: Vec<String>,
    /// 第一次观测时间
    pub first_obs_time: Option<DateTime<Utc>>,
    /// 最后一次观测时间
    pub last_obs_time: Option<DateTime<Utc>>,
    /// 接收机时钟偏移标志
    pub rcv_clock_offs_applied: bool,
    /// 电离层参数
    pub iono_params: Vec<IonoParams>,
    /// 时间系统参数
    pub time_system_params: Vec<TimeSystemParams>,
    /// 闰秒信息
    pub leap_seconds: Option<i32>,
    /// GLONASS频率通道
    pub glonass_channels: Vec<GlonassFreqChannel>,
    /// 相位中心偏移
    pub phase_center_offsets: Vec<PhaseCenterOffset>,
    /// 其他头部项
    pub custom_fields: Vec<(String, String)>,
}

impl Default for RinexHeader {
    fn default() -> Self {
        Self {
            version: 3.04,
            file_type: RinexFileType::Observation,
            satellite_system: RinexSystem::Mixed,
            program: "rtcm2rinex".to_string(),
            run_by: "UNKNOWN".to_string(),
            date_created: Utc::now(),
            marker_name: "UNKNOWN".to_string(),
            marker_number: None,
            observer: "UNKNOWN".to_string(),
            agency: "UNKNOWN".to_string(),
            receiver_number: "UNKNOWN".to_string(),
            receiver_type: "UNKNOWN".to_string(),
            receiver_version: "UNKNOWN".to_string(),
            antenna_number: "UNKNOWN".to_string(),
            antenna_type: "UNKNOWN".to_string(),
            approx_position_x: 0.0,
            approx_position_y: 0.0,
            approx_position_z: 0.0,
            antenna_height: 0.0,
            antenna_e: 0.0,
            antenna_n: 0.0,
            obs_types: Vec::new(),
            first_obs_time: None,
            last_obs_time: None,
            rcv_clock_offs_applied: false,
            iono_params: Vec::new(),
            time_system_params: Vec::new(),
            leap_seconds: None,
            glonass_channels: Vec::new(),
            phase_center_offsets: Vec::new(),
            custom_fields: Vec::new(),
        }
    }
}

impl RinexHeader {
    /// 创建新的RINEX头部
    pub fn new(version: f64, file_type: RinexFileType, satellite_system: RinexSystem) -> Self {
        Self {
            version,
            file_type,
            satellite_system,
            ..Default::default()
        }
    }

    /// 向RINEX头部添加观测类型
    pub fn add_obs_type(&mut self, obs_type: &str) {
        if !self.obs_types.contains(&obs_type.to_string()) {
            self.obs_types.push(obs_type.to_string());
        }
    }

    /// 设置站点位置
    pub fn set_position(&mut self, x: f64, y: f64, z: f64) {
        self.approx_position_x = x;
        self.approx_position_y = y;
        self.approx_position_z = z;
    }

    /// 添加电离层参数
    pub fn add_iono_params(&mut self, system: RinexSystem, params: Vec<f64>) {
        self.iono_params.push(IonoParams { system, params });
    }

    /// 添加时间系统参数
    pub fn add_time_system_params(&mut self, system: RinexSystem, params: Vec<f64>, 
                                 ref_time: Option<DateTime<Utc>>, ref_system: Option<String>) {
        self.time_system_params.push(TimeSystemParams { 
            system, 
            params, 
            ref_time, 
            ref_system 
        });
    }

    /// 添加GLONASS频率通道
    pub fn add_glonass_channel(&mut self, prn: u8, freq_num: i8) {
        self.glonass_channels.push(GlonassFreqChannel { prn, freq_num });
    }

    /// 添加相位中心偏移
    pub fn add_phase_center_offset(&mut self, signal: &str, north: f64, east: f64, up: f64) {
        self.phase_center_offsets.push(PhaseCenterOffset { 
            signal: signal.to_string(), 
            north, 
            east, 
            up 
        });
    }

    /// 写入RINEX头部到文件
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), RinexError> {
        // 写入版本行
        writeln!(writer, "{:9.2}           {} {:11}RINEX VERSION / TYPE", 
                self.version, self.file_type, self.satellite_system)?;

        // 写入程序信息
        let date_str = self.date_created.format("%Y%m%d %H%M%S UTC").to_string();
        writeln!(writer, "{:<20}{:<20}{:<20}PGM / RUN BY / DATE", 
                &self.program, &self.run_by, &date_str)?;

        // 写入注释行
        writeln!(writer, "{:<60}COMMENT", 
                &format!("Converted by rtcm2rinex v{}", env!("CARGO_PKG_VERSION")))?;

        // 写入标记名称
        writeln!(writer, "{:<60}MARKER NAME", &self.marker_name)?;

        // 可选标记编号
        if let Some(ref marker_number) = self.marker_number {
            writeln!(writer, "{:<60}MARKER NUMBER", marker_number)?;
        }

        // 观测者和机构
        writeln!(writer, "{:<20}{:<40}OBSERVER / AGENCY", 
                &self.observer, &self.agency)?;

        // 接收机信息
        writeln!(writer, "{:<20}{:<20}{:<20}REC # / TYPE / VERS", 
                &self.receiver_number, &self.receiver_type, &self.receiver_version)?;

        // 天线信息
        writeln!(writer, "{:<20}{:<40}ANT # / TYPE", 
                &self.antenna_number, &self.antenna_type)?;

        // 近似位置
        writeln!(writer, "{:14.4}{:14.4}{:14.4}{}APPROX POSITION XYZ", 
                self.approx_position_x, self.approx_position_y, self.approx_position_z, "")?;

        // 天线高度
        writeln!(writer, "{:14.4}{:14.4}{:14.4}{}ANTENNA: DELTA H/E/N", 
                self.antenna_height, self.antenna_e, self.antenna_n, "")?;

        // 观测类型 - 根据RINEX版本使用不同格式
        if self.version >= 3.0 {
            // RINEX 3.x格式 - 按系统分类
            let mut types_by_system: HashMap<char, Vec<String>> = HashMap::new();
            
            for obs_type in &self.obs_types {
                if obs_type.len() >= 3 {
                    let sys = obs_type.chars().next().unwrap();
                    let code = &obs_type[1..];
                    types_by_system.entry(sys).or_insert_with(Vec::new).push(code.to_string());
                }
            }

            // 每个系统写入一行
            for (sys, codes) in &types_by_system {
                // 第一行
                write!(writer, "{}{:>3}", sys, codes.len())?;
                
                // 每行最多13个观测类型
                for (i, code) in codes.iter().enumerate() {
                    if i > 0 && i % 13 == 0 {
                        // 继续新行
                        writeln!(writer, "{:56}SYS / # / OBS TYPES", "")?;
                        write!(writer, "{}{:>3}", sys, "")?;
                    }
                    write!(writer, " {:>3}", code)?;
                }
                
                // 补齐空格并结束行
                let remaining = 13 - (codes.len() % 13);
                if remaining < 13 {
                    for _ in 0..remaining {
                        write!(writer, "    ")?;
                    }
                }
                writeln!(writer, "{:>56}", "SYS / # / OBS TYPES")?;
            }
        } else {
            // RINEX 2.x格式 - 所有类型合并
            // 移除系统前缀
            let mut obs_types_v2: Vec<String> = Vec::new();
            for obs_type in &self.obs_types {
                if obs_type.len() >= 3 {
                    let code = &obs_type[1..];
                    if !obs_types_v2.contains(&code.to_string()) {
                        obs_types_v2.push(code.to_string());
                    }
                }
            }
            
            // 写入观测类型数量
            write!(writer, "{:6}", obs_types_v2.len())?;
            
            // 每行最多9个观测类型
            for (i, code) in obs_types_v2.iter().enumerate() {
                if i > 0 && i % 9 == 0 {
                    // 继续新行
                    writeln!(writer, "{:54}# / TYPES OF OBSERV", "")?;
                    write!(writer, "      ")?;
                }
                write!(writer, "{:>6}", code)?;
            }
            
            // 补齐空格并结束行
            let remaining = 9 - (obs_types_v2.len() % 9);
            if remaining < 9 {
                for _ in 0..remaining {
                    write!(writer, "      ")?;
                }
            }
            writeln!(writer, "{:>54}", "# / TYPES OF OBSERV")?;
        }

        // 写入信号相位偏移 (RINEX 3.x)
        if self.version >= 3.0 && !self.phase_center_offsets.is_empty() {
            for offset in &self.phase_center_offsets {
                let line = format!("{:<4} {:8} {:8} {:8}                                SYS / PHASE SHIFT", 
                        offset.signal, offset.north, offset.east, offset.up);
                writeln!(writer, "{}", line)?;
            }
        }

        // 写入GLONASS频率通道 (RINEX 3.x)
        if self.version >= 3.0 && !self.glonass_channels.is_empty() {
            write!(writer, "{:>3}", self.glonass_channels.len())?;
            
            for (i, channel) in self.glonass_channels.iter().enumerate() {
                if i > 0 && i % 8 == 0 {
                    // 继续新行
                    writeln!(writer, "{:>56}", "GLONASS SLOT / FRQ #")?;
                    write!(writer, "   ")?;
                }
                write!(writer, " R{:02}{:>2}", channel.prn, channel.freq_num)?;
            }
            
            // 补齐空格并结束行
            let remaining = 8 - (self.glonass_channels.len() % 8);
            if remaining < 8 {
                for _ in 0..remaining {
                    write!(writer, "     ")?;
                }
            }
            writeln!(writer, "{:>56}", "GLONASS SLOT / FRQ #")?;
        }

        // 写入电离层参数
        for iono in &self.iono_params {
            let sys_char = match iono.system {
                RinexSystem::GPS => "G",
                RinexSystem::Galileo => "E",
                RinexSystem::BeiDou => "C",
                RinexSystem::QZSS => "J",
                RinexSystem::IRNSS => "I",
                _ => continue, // 其他系统不支持
            };
            
            let label = match iono.system {
                RinexSystem::GPS => "IONOSPHERIC CORR",
                RinexSystem::Galileo => "IONOSPHERIC CORR",
                RinexSystem::BeiDou => "IONOSPHERIC CORR",
                RinexSystem::QZSS => "IONOSPHERIC CORR",
                RinexSystem::IRNSS => "IONOSPHERIC CORR",
                _ => continue,
            };
            
            // 写入参数 (最多4个参数)
            if iono.params.len() >= 4 {
                writeln!(writer, "{}  {:12.4E}{:12.4E}{:12.4E}{:12.4E}  {}", 
                        sys_char, iono.params[0], iono.params[1], iono.params[2], iono.params[3], label)?;
                
                // 如果有更多参数 (如Galileo)
                if iono.params.len() >= 8 {
                    writeln!(writer, "{}  {:12.4E}{:12.4E}{:12.4E}{:12.4E}  {}", 
                            sys_char, iono.params[4], iono.params[5], iono.params[6], iono.params[7], label)?;
                }
            }
        }

        // 写入时间系统参数
        for time_sys in &self.time_system_params {
            let sys_char = match time_sys.system {
                RinexSystem::GPS => "G",
                RinexSystem::Galileo => "E",
                RinexSystem::BeiDou => "C",
                RinexSystem::QZSS => "J",
                RinexSystem::GLONASS => "R",
                RinexSystem::SBAS => "S",
                RinexSystem::IRNSS => "I",
                _ => continue,
            };
            
            let label = match time_sys.system {
                RinexSystem::GPS => "TIME SYSTEM CORR",
                RinexSystem::Galileo => "TIME SYSTEM CORR",
                RinexSystem::BeiDou => "TIME SYSTEM CORR",
                RinexSystem::QZSS => "TIME SYSTEM CORR",
                RinexSystem::GLONASS => "TIME SYSTEM CORR",
                RinexSystem::SBAS => "TIME SYSTEM CORR",
                RinexSystem::IRNSS => "TIME SYSTEM CORR",
                _ => continue,
            };
            
            // 写入参数 (通常是A0, A1)
            if time_sys.params.len() >= 2 {
                writeln!(writer, "{}  {:12.4E}{:12.4E}{:>6}{:>6}  {}", 
                        sys_char, time_sys.params[0], time_sys.params[1], 
                        time_sys.ref_time.map_or("".to_string(), |t| t.format("%Y%m%d").to_string()),
                        time_sys.ref_system.as_deref().unwrap_or(""),
                        label)?;
            }
        }

        // 写入闰秒信息
        if let Some(leap_sec) = self.leap_seconds {
            writeln!(writer, "{:6}{:54}LEAP SECONDS", leap_sec, "")?;
        }

        // 写入第一次观测时间
        if let Some(first_time) = &self.first_obs_time {
            if self.version >= 3.0 {
                writeln!(writer, "  {:4} {:02} {:02} {:02} {:02} {:010.7}{:>23}TIME OF FIRST OBS", 
                        first_time.year(), first_time.month(), first_time.day(),
                        first_time.hour(), first_time.minute(), 
                        first_time.second() as f64 + first_time.nanosecond() as f64 / 1_000_000_000.0,
                        "GPS")?;
            } else {
                writeln!(writer, "  {:4} {:2} {:2} {:2} {:2} {:11.7}{:>5}{:>18}TIME OF FIRST OBS", 
                        first_time.year(), first_time.month(), first_time.day(),
                        first_time.hour(), first_time.minute(), 
                        first_time.second() as f64 + first_time.nanosecond() as f64 / 1_000_000_000.0,
                        "", "GPS")?;
            }
        }

        // 写入最后一次观测时间
        if let Some(last_time) = &self.last_obs_time {
            if self.version >= 3.0 {
                writeln!(writer, "  {:4} {:02} {:02} {:02} {:02} {:010.7}{:>23}TIME OF LAST OBS", 
                        last_time.year(), last_time.month(), last_time.day(),
                        last_time.hour(), last_time.minute(), 
                        last_time.second() as f64 + last_time.nanosecond() as f64 / 1_000_000_000.0,
                        "GPS")?;
            } else {
                writeln!(writer, "  {:4} {:2} {:2} {:2} {:2} {:11.7}{:>5}{:>18}TIME OF LAST OBS", 
                        last_time.year(), last_time.month(), last_time.day(),
                        last_time.hour(), last_time.minute(), 
                        last_time.second() as f64 + last_time.nanosecond() as f64 / 1_000_000_000.0,
                        "", "GPS")?;
            }
        }

        // 接收机时钟偏移标志
        if self.rcv_clock_offs_applied {
            writeln!(writer, "{:6}{:54}RCV CLOCK OFFS APPL", 1, "")?;
        }

        // 写入自定义字段
        for (key, value) in &self.custom_fields {
            writeln!(writer, "{:<60}{}", value, key)?;
        }

        // 写入头部结束标志
        writeln!(writer, "{:>60}", "END OF HEADER")?;

        Ok(())
    }
}