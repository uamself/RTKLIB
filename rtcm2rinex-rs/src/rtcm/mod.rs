/*!
 * RTCM消息处理模块
 * 
 * 该模块实现了RTCM格式的解析和处理。
 */

// 导出子模块
pub mod rtcm2;
pub mod rtcm3; // 已实现的RTCM3解析模块
pub mod msm;   // MSM消息解析模块

// 从子模块重新导出关键类型
pub use self::rtcm2::{Rtcm2Parser, Rtcm2Message, Rtcm2MessageType, Rtcm2Error};
pub use self::rtcm3::{Rtcm3Parser, Rtcm3MessageType, Rtcm3Error};

use thiserror::Error;
use std::io;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::gnss::time::GnssTime;
use crate::rinex::obs::{RinexEpochData, RinexObservation};
use crate::rinex::nav::{NavData, GpsNavData, GlonassNavData};

/// RTCM处理上下文
#[derive(Debug)]
pub struct RtcmContext {
    // 内部状态
    buffer: Vec<u8>,
    message_length: usize,
    state: RtcmState,
    
    // RTCM2解析器
    rtcm2_parser: rtcm2::Rtcm2Parser,
    
    // RTCM3解析器
    rtcm3_parser: rtcm3::Rtcm3Parser,
    
    // 最后接收到的消息类型
    last_message_type: Option<RtcmMessageType>,
    
    // 观测数据记录 (时间 -> 历元数据)
    pub observations: HashMap<i64, RinexEpochData>,
    
    // GPS导航数据记录 (PRN -> 导航数据)
    pub gps_nav_data: HashMap<u8, Vec<GpsNavData>>,
    
    // GLONASS导航数据记录 (SLOT -> 导航数据)
    pub glo_nav_data: HashMap<u8, Vec<GlonassNavData>>,
    
    // 参考站信息
    pub station_id: u16,
    pub station_pos: Option<(f64, f64, f64)>,
    pub antenna_height: Option<f64>,
    pub antenna_descriptor: Option<String>,
    pub antenna_serial: Option<String>,
    pub receiver_type: Option<String>,
    pub firmware_version: Option<String>,
    pub receiver_serial: Option<String>,
}

/// RTCM处理状态
#[derive(Debug)]
enum RtcmState {
    /// 等待前导码
    WaitingForPreamble,
    
    /// 读取消息头
    ReadingHeader,
    
    /// 读取消息体
    ReadingBody,
}

/// RTCM消息类型
#[derive(Debug, Clone)]
pub enum RtcmMessageType {
    /// RTCM2消息
    Rtcm2(rtcm2::Rtcm2MessageType),
    
    /// RTCM3消息
    Rtcm3(rtcm3::Rtcm3MessageType),
    
    /// 未知类型
    Unknown(u16),
}

/// RTCM错误类型
#[derive(Debug, Error)]
pub enum RtcmError {
    #[error("Invalid message format")]
    InvalidFormat,
    
    #[error("Checksum error")]
    ChecksumError,
    
    #[error("Unsupported message type: {0}")]
    UnsupportedType(u16),
    
    #[error("RTCM2 error: {0}")]
    Rtcm2Error(#[from] rtcm2::Rtcm2Error),
    
    #[error("RTCM3 error: {0}")]
    Rtcm3Error(#[from] rtcm3::Rtcm3Error),
    
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl RtcmContext {
    /// 创建新的RTCM上下文
    pub fn new() -> Result<Self, RtcmError> {
        Ok(Self {
            buffer: Vec::with_capacity(1024),
            message_length: 0,
            state: RtcmState::WaitingForPreamble,
            rtcm2_parser: rtcm2::Rtcm2Parser::new(),
            rtcm3_parser: rtcm3::Rtcm3Parser::new(),
            last_message_type: None,
            observations: HashMap::new(),
            gps_nav_data: HashMap::new(),
            glo_nav_data: HashMap::new(),
            station_id: 0,
            station_pos: None,
            antenna_height: None,
            antenna_descriptor: None,
            antenna_serial: None,
            receiver_type: None,
            firmware_version: None,
            receiver_serial: None,
        })
    }
    
    /// 处理单个字节
    pub fn process_byte(&mut self, byte: u8) -> Result<Option<RtcmMessageType>, RtcmError> {
        // 首先尝试RTCM3解析器
        match self.rtcm3_parser.process_byte(byte)? {
            Some(msg) => {
                let msg_type = RtcmMessageType::Rtcm3(msg.clone());
                self.last_message_type = Some(msg_type.clone());
                
                // 处理接收到的消息并更新内部状态
                self.process_rtcm3_message(&msg)?;
                
                return Ok(Some(msg_type));
            },
            None => {
                // RTCM3解析器没有返回消息，继续尝试RTCM2
            }
        }
        
        // 如果RTCM3没有结果，尝试RTCM2解析器
        match self.rtcm2_parser.process_byte(byte)? {
            Some(msg) => {
                let msg_type = RtcmMessageType::Rtcm2(msg.header.message_type);
                self.last_message_type = Some(msg_type.clone());
                
                // 处理RTCM2消息 (未实现)
                
                return Ok(Some(msg_type));
            },
            None => {
                // 两个解析器都没有返回消息
            }
        }
        
        // 如果都没有返回消息，说明还在解析过程中
        Ok(None)
    }
    
    /// 处理RTCM3消息并更新内部状态
    fn process_rtcm3_message(&mut self, msg: &rtcm3::Rtcm3MessageType) -> Result<(), RtcmError> {
        match msg {
            // 处理MSM消息 (观测数据)
            rtcm3::Rtcm3MessageType::Msm4Gps { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm5Gps { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm6Gps { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm7Gps { station_id, epoch_time, satellites, signals } => {
                self.station_id = *station_id;
                self.process_msm_observation(epoch_time, satellites, signals, 'G')?;
            },
            rtcm3::Rtcm3MessageType::Msm4Glonass { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm5Glonass { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm6Glonass { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm7Glonass { station_id, epoch_time, satellites, signals } => {
                self.station_id = *station_id;
                self.process_msm_observation(epoch_time, satellites, signals, 'R')?;
            },
            rtcm3::Rtcm3MessageType::Msm4Galileo { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm5Galileo { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm6Galileo { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm7Galileo { station_id, epoch_time, satellites, signals } => {
                self.station_id = *station_id;
                self.process_msm_observation(epoch_time, satellites, signals, 'E')?;
            },
            rtcm3::Rtcm3MessageType::Msm4Beidou { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm5Beidou { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm6Beidou { station_id, epoch_time, satellites, signals } |
            rtcm3::Rtcm3MessageType::Msm7Beidou { station_id, epoch_time, satellites, signals } => {
                self.station_id = *station_id;
                self.process_msm_observation(epoch_time, satellites, signals, 'C')?;
            },
            
            // 处理参考站坐标
            rtcm3::Rtcm3MessageType::ReferenceStationCoordinates { station_id, x, y, z, antenna_height, .. } => {
                self.station_id = *station_id;
                self.station_pos = Some((*x, *y, *z));
                self.antenna_height = *antenna_height;
            },
            
            // 处理天线描述
            rtcm3::Rtcm3MessageType::AntennaDescription { station_id, description, serial_number, .. } => {
                self.station_id = *station_id;
                self.antenna_descriptor = Some(description.clone());
                self.antenna_serial = serial_number.clone();
            },
            
            // 处理接收机和天线描述
            rtcm3::Rtcm3MessageType::ReceiverAntennaDescription { 
                station_id, antenna_descriptor, antenna_serial, receiver_type, firmware_version, receiver_serial, .. 
            } => {
                self.station_id = *station_id;
                self.antenna_descriptor = Some(antenna_descriptor.clone());
                self.antenna_serial = Some(antenna_serial.clone());
                self.receiver_type = Some(receiver_type.clone());
                self.firmware_version = Some(firmware_version.clone());
                self.receiver_serial = Some(receiver_serial.clone());
            },
            
            // 处理GPS星历
            rtcm3::Rtcm3MessageType::GpsEphemeris { station_id, prn, week, sva, code_l2, idot, iode, toc, af2, af1, af0,
                iodc, crs, deln, m0, cuc, e, cus, sqrt_a, toe, cic, omega0, cis, i0, crc, omega, omegad, tgd, svh, flag, .. } => {
                self.station_id = *station_id;
                
                // 创建GPS星历数据
                let utc_time = GnssTime::new_gps(*week as i32, *toc).to_utc_datetime();
                let nav_data = GpsNavData {
                    prn: *prn,
                    toc: utc_time,
                    week: *week,
                    sva: *sva,
                    code_l2: *code_l2,
                    idot: *idot,
                    iode: *iode,
                    af2: *af2,
                    af1: *af1,
                    af0: *af0,
                    iodc: *iodc,
                    crs: *crs,
                    delta_n: *deln,
                    m0: *m0,
                    cuc: *cuc,
                    e: *e,
                    cus: *cus,
                    sqrt_a: *sqrt_a,
                    toe: *toe,
                    cic: *cic,
                    omega0: *omega0,
                    cis: *cis,
                    i0: *i0,
                    crc: *crc,
                    omega: *omega,
                    omega_dot: *omegad,
                    tgd: *tgd,
                    svh: *svh,
                    flag: *flag,
                };
                
                // 存储GPS星历
                self.gps_nav_data.entry(*prn).or_insert_with(Vec::new).push(nav_data);
            },
            
            // 处理GLONASS星历
            rtcm3::Rtcm3MessageType::GlonassEphemeris { 
                station_id, slot, freqo, svh, age, pos, vel, acc, tau_n, gamma, tk, week, tod, .. 
            } => {
                self.station_id = *station_id;
                
                // 创建GLONASS星历数据
                let glonass_time = GnssTime::new_gps(*week as i32, *tod);
                let utc_time = glonass_time.to_utc_datetime();
                
                let nav_data = GlonassNavData {
                    slot: *slot,
                    epoch: utc_time,
                    freqno: *freqo,
                    age: *age,
                    svh: *svh,
                    pos_x: pos[0],
                    pos_y: pos[1],
                    pos_z: pos[2],
                    vel_x: vel[0],
                    vel_y: vel[1],
                    vel_z: vel[2],
                    acc_x: acc[0],
                    acc_y: acc[1],
                    acc_z: acc[2],
                    gamma: *gamma,
                    tau_n: *tau_n,
                    tk: *tk,
                };
                
                // 存储GLONASS星历
                self.glo_nav_data.entry(*slot).or_insert_with(Vec::new).push(nav_data);
            },
            
            // 其他消息类型暂不处理
            _ => {}
        }
        
        Ok(())
    }
    
    /// 处理MSM观测数据
    fn process_msm_observation(&mut self, epoch_time: &GnssTime, satellites: &[msm::MsmSatelliteData], 
                              signals: &[msm::MsmSignalData], sys_id: char) -> Result<(), RtcmError> {
        // 获取当前历元时间的整数秒部分作为键
        let time_key = epoch_time.seconds() as i64;
        
        // 获取或创建当前历元的观测数据
        let epoch_data = self.observations.entry(time_key).or_insert_with(|| {
            RinexEpochData::new(epoch_time.clone())
        });
        
        // 处理每个信号的观测数据
        for signal in signals {
            // 获取相应卫星数据
            if signal.satellite_index as usize >= satellites.len() {
                continue; // 索引无效
            }
            
            let sat_data = &satellites[signal.satellite_index as usize];
            let sat_id = sat_data.satellite_id;
            
            // 构建卫星PRN标识
            let prn = format!("{}{:02}", sys_id, sat_id);
            
            // 确定信号类型 (简化实现，实际应根据信号索引确定具体类型)
            // 这里使用一个简单的映射规则
            let obs_type = match signal.signal_index {
                0 => format!("{}1C", sys_id),  // L1C 观测类型
                1 => format!("{}2X", sys_id),  // L2X 观测类型
                2 => format!("{}5X", sys_id),  // L5X 观测类型
                _ => format!("{}{}X", sys_id, signal.signal_index), // 其他信号类型
            };
            
            // 处理伪距观测
            if let Some(fine_pseudorange) = signal.fine_pseudorange {
                // 伪距观测 = 粗略伪距 + 精细伪距
                let pseudorange = sat_data.rough_range_ms * 299792.458 + fine_pseudorange;
                
                // 创建伪距观测数据
                let observation = RinexObservation::new(pseudorange);
                
                // 添加到观测数据中
                epoch_data.add_observation(&prn, &obs_type, observation);
            }
            
            // 处理载波相位观测
            if let Some(fine_phaserange) = signal.fine_phaserange {
                // 载波相位观测 = 粗略相位 + 精细相位
                let phaserange = sat_data.rough_range_ms * 299792.458 + fine_phaserange;
                
                // 创建相位观测数据
                let observation = RinexObservation::new(phaserange);
                
                // 添加到观测数据中
                epoch_data.add_observation(&prn, &format!("L{}", &obs_type[1..]), observation);
            }
            
            // 处理多普勒观测
            if let Some(phase_range_rate) = sat_data.phase_range_rate {
                // 创建多普勒观测数据
                let observation = RinexObservation::new(phase_range_rate as f64);
                
                // 添加到观测数据中
                epoch_data.add_observation(&prn, &format!("D{}", &obs_type[1..]), observation);
            }
            
            // 处理信噪比观测
            if let Some(cnr) = signal.cnr {
                // 创建信噪比观测数据
                let observation = RinexObservation::new(cnr as f64);
                
                // 添加到观测数据中
                epoch_data.add_observation(&prn, &format!("S{}", &obs_type[1..]), observation);
            }
        }
        
        Ok(())
    }
    
    /// 获取所有观测类型
    pub fn get_observation_types(&self) -> HashMap<char, Vec<String>> {
        let mut obs_types: HashMap<char, Vec<String>> = HashMap::new();
        
        // 遍历所有历元的观测数据
        for epoch_data in self.observations.values() {
            // 遍历每个卫星的观测数据
            for (prn, sat_obs) in &epoch_data.observations {
                if !prn.is_empty() {
                    // 获取卫星系统标识
                    let sys_id = prn.chars().next().unwrap();
                    
                    // 获取或创建系统的观测类型列表
                    let sys_types = obs_types.entry(sys_id).or_insert_with(Vec::new);
                    
                    // 添加所有观测类型
                    for obs_type in sat_obs.keys() {
                        if !sys_types.contains(obs_type) {
                            sys_types.push(obs_type.clone());
                        }
                    }
                }
            }
        }
        
        // 对每个系统的观测类型进行排序
        for types in obs_types.values_mut() {
            types.sort();
        }
        
        obs_types
    }
    
    /// 获取观测历元数量
    pub fn get_observation_epochs_count(&self) -> usize {
        self.observations.len()
    }
    
    /// 获取最早的观测时间
    pub fn get_first_observation_time(&self) -> Option<GnssTime> {
        self.observations.keys()
            .min()
            .map(|&key| {
                self.observations.get(&key).map(|epoch| epoch.time.clone()).unwrap_or_default()
            })
    }
    
    /// 获取最晚的观测时间
    pub fn get_last_observation_time(&self) -> Option<GnssTime> {
        self.observations.keys()
            .max()
            .map(|&key| {
                self.observations.get(&key).map(|epoch| epoch.time.clone()).unwrap_or_default()
            })
    }
    
    /// 获取GPS星历数量
    pub fn get_gps_nav_count(&self) -> usize {
        self.gps_nav_data.values().map(|v| v.len()).sum()
    }
    
    /// 获取GLONASS星历数量
    pub fn get_glonass_nav_count(&self) -> usize {
        self.glo_nav_data.values().map(|v| v.len()).sum()
    }
    
    /// 清除数据
    pub fn clear(&mut self) {
        self.observations.clear();
        self.gps_nav_data.clear();
        self.glo_nav_data.clear();
        self.buffer.clear();
        self.message_length = 0;
        self.state = RtcmState::WaitingForPreamble;
        self.last_message_type = None;
    }
} 