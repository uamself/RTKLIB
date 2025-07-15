/*!
 * MSM (Multiple Signal Message) 解析模块
 * 
 * 本模块实现了 RTCM3 MSM (Multiple Signal Message) 消息的解析功能。
 * MSM 消息用于传输各种卫星系统的观测数据，包括伪距、相位、多普勒和信号强度。
 */

use crate::gnss::time::GnssTime;
use crate::util::bits::BitReader;
use crate::rtcm::Rtcm3MessageType;
use crate::rtcm::Rtcm3Error;
use thiserror::Error;
use std::io;

/// MSM 解析错误
#[derive(Debug, Error)]
pub enum MsmError {
    /// 无效的消息类型
    #[error("无效的MSM消息类型: {0}")]
    InvalidMessageType(u16),
    
    /// 无效的卫星掩码
    #[error("无效的卫星掩码")]
    InvalidSatelliteMask,
    
    /// 无效的信号掩码
    #[error("无效的信号掩码")]
    InvalidSignalMask,
    
    /// 无效的单元掩码
    #[error("无效的单元掩码")]
    InvalidCellMask,
    
    /// 数据解析错误
    #[error("数据解析错误: {0}")]
    ParseError(String),
    
    /// I/O 错误
    #[error("I/O 错误: {0}")]
    IoError(#[from] io::Error),
}

/// MSM 解析结果
pub type Result<T> = std::result::Result<T, MsmError>;

/// MSM 消息类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MsmType {
    /// MSM1: 紧凑观测数据
    Compact = 1,
    /// MSM2: 紧凑相位数据
    CompactPhaseRangeRate = 2,
    /// MSM3: 紧凑相位和伪距数据
    CompactPhaseRange = 3,
    /// MSM4: 完整相位和伪距数据，伪距高精度
    FullPseudoranges = 4,
    /// MSM5: 完整相位和伪距、相位范围率数据，伪距高精度
    FullPseudorangesRates = 5,
    /// MSM6: 完整相位和伪距数据，相位高精度，伪距高精度
    FullPhaserangesPseudoranges = 6,
    /// MSM7: 完整相位、伪距和相位范围率数据，相位高精度，伪距高精度
    FullPhaserangesPseudorangesRates = 7,
}

/// 卫星系统类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GnssType {
    /// GPS
    Gps,
    /// GLONASS
    Glonass,
    /// Galileo
    Galileo,
    /// SBAS
    Sbas,
    /// QZSS
    Qzss,
    /// BeiDou
    Beidou,
    /// NavIC/IRNSS
    NavIc,
    /// 其他
    Other(u8),
}

/// MSM 卫星数据
#[derive(Debug, Clone, PartialEq)]
pub struct MsmSatelliteData {
    /// 卫星 ID
    pub satellite_id: u8,
    /// 整数毫秒范围伪距
    pub rough_range_ms: f64,
    /// 相位范围率
    pub phase_range_rate: Option<f32>,
}

/// MSM 信号数据
#[derive(Debug, Clone, PartialEq)]
pub struct MsmSignalData {
    /// 卫星索引
    pub satellite_index: u8,
    /// 信号索引
    pub signal_index: u8,
    /// 精细伪距
    pub fine_pseudorange: Option<f64>,
    /// 精细相位
    pub fine_phaserange: Option<f64>,
    /// 锁时间指示器
    pub lock_time_indicator: Option<u8>,
    /// 半周期歧义度指示器
    pub half_cycle_ambiguity: Option<bool>,
    /// 载噪比
    pub cnr: Option<f32>,
}

/// MSM 消息头
#[derive(Debug, Clone)]
pub struct MsmHeader {
    /// 参考站 ID
    pub station_id: u16,
    /// GNSS 系统类型
    pub gnss_type: GnssType,
    /// MSM 消息类型
    pub msm_type: MsmType,
    /// 历元时间
    pub epoch_time: GnssTime,
    /// 多消息标志
    pub multiple_message: bool,
    /// 问题卫星标志
    pub issue_of_data_station: u8,
    /// 平滑指示器
    pub smooth_indicator: u8,
    /// 平滑间隔
    pub smooth_interval: u8,
    /// 卫星掩码
    pub satellite_mask: u64,
    /// 信号掩码
    pub signal_mask: u32,
    /// 单元掩码
    pub cell_mask: Vec<u8>,
}

// 常量定义，对应RTKLIB中的常量
const RANGE_MS: f64 = 299792.458; // 光速(m/ms)
const P2_10: f64 = 0.0009765625; // 2^-10
const P2_24: f64 = 5.960464477539063E-8; // 2^-24
const P2_29: f64 = 1.862645149230957E-9; // 2^-29
const P2_31: f64 = 4.656612873077393E-10; // 2^-31

/// MSM主入口解析函数，对应RTKLIB中的decode_rtcm3中对MSM类型的分发
pub fn parse_msm(payload: &[u8], msg_type: u16) -> Result<Rtcm3MessageType> {
    // MSM类型分发，参考RTKLIB的decode_msm4/5/6/7
    let result = match msg_type {
        1074 | 1084 | 1094 | 1124 => parse_msm4(payload, msg_type),
        1075 | 1085 | 1095 | 1125 => parse_msm5(payload, msg_type),
        1076 | 1086 | 1096 | 1126 => parse_msm6(payload, msg_type),
        1077 | 1087 | 1097 | 1127 => parse_msm7(payload, msg_type),
        _ => Err(MsmError::InvalidMessageType(msg_type)),
    };

    // 不需要转换错误类型，因为我们内部使用 Result<T> = std::result::Result<T, MsmError>
    result
}

/// 解析MSM4消息，对应RTKLIB中的decode_msm4
fn parse_msm4(payload: &[u8], msg_type: u16) -> Result<Rtcm3MessageType> {
    // 创建位读取器
    let mut bit_reader = BitReader::new(payload);
    
    // 解析MSM头部
    let (header, satellites, signals) = parse_msm_common(&mut bit_reader, msg_type, MsmType::FullPseudoranges)?;
    
    // 根据不同的卫星系统创建相应的消息类型
    create_msm_message(msg_type, header, satellites, signals)
}

/// 解析MSM5消息，对应RTKLIB中的decode_msm5
fn parse_msm5(payload: &[u8], msg_type: u16) -> Result<Rtcm3MessageType> {
    // 创建位读取器
    let mut bit_reader = BitReader::new(payload);
    
    // 解析MSM头部
    let (header, satellites, signals) = parse_msm_common(&mut bit_reader, msg_type, MsmType::FullPseudorangesRates)?;
    
    // 根据不同的卫星系统创建相应的消息类型
    create_msm_message(msg_type, header, satellites, signals)
}

/// 解析MSM6消息，对应RTKLIB中的decode_msm6
fn parse_msm6(payload: &[u8], msg_type: u16) -> Result<Rtcm3MessageType> {
    // 创建位读取器
    let mut bit_reader = BitReader::new(payload);
    
    // 解析MSM头部
    let (header, satellites, signals) = parse_msm_common(&mut bit_reader, msg_type, MsmType::FullPhaserangesPseudoranges)?;
    
    // 根据不同的卫星系统创建相应的消息类型
    create_msm_message(msg_type, header, satellites, signals)
}

/// 解析MSM7消息，对应RTKLIB中的decode_msm7
fn parse_msm7(payload: &[u8], msg_type: u16) -> Result<Rtcm3MessageType> {
    // 创建位读取器
    let mut bit_reader = BitReader::new(payload);
    
    // 解析MSM头部
    let (header, satellites, signals) = parse_msm_common(&mut bit_reader, msg_type, MsmType::FullPhaserangesPseudorangesRates)?;
    
    // 根据不同的卫星系统创建相应的消息类型
    create_msm_message(msg_type, header, satellites, signals)
}

/// 根据消息类型获取卫星系统类型
fn get_gnss_type(msg_type: u16) -> GnssType {
    match msg_type {
        1074..=1077 => GnssType::Gps,
        1084..=1087 => GnssType::Glonass,
        1094..=1097 => GnssType::Galileo,
        1104..=1107 => GnssType::Sbas,
        1114..=1117 => GnssType::Qzss,
        1124..=1127 => GnssType::Beidou,
        1134..=1137 => GnssType::NavIc,
        _ => GnssType::Other(0),
    }
}

/// 创建MSM消息，根据不同的卫星系统
fn create_msm_message(msg_type: u16, header: MsmHeader, satellites: Vec<MsmSatelliteData>, signals: Vec<MsmSignalData>) -> Result<Rtcm3MessageType> {
    match header.msm_type {
        MsmType::FullPseudoranges => {
            // MSM4
            match header.gnss_type {
                GnssType::Gps => Ok(Rtcm3MessageType::Msm4Gps {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Glonass => Ok(Rtcm3MessageType::Msm4Glonass {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Galileo => Ok(Rtcm3MessageType::Msm4Galileo {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Beidou => Ok(Rtcm3MessageType::Msm4Beidou {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                _ => {
                    // 对于其他卫星系统，返回Raw类型
                    let mut raw_data = Vec::new();
                    raw_data.extend_from_slice(&[(msg_type >> 8) as u8, (msg_type & 0xFF) as u8]);
                    Ok(Rtcm3MessageType::Raw {
                        msg_type,
                        payload: raw_data,
                    })
                }
            }
        },
        MsmType::FullPseudorangesRates => {
            // MSM5
            match header.gnss_type {
                GnssType::Gps => Ok(Rtcm3MessageType::Msm5Gps {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Glonass => Ok(Rtcm3MessageType::Msm5Glonass {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Galileo => Ok(Rtcm3MessageType::Msm5Galileo {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Beidou => Ok(Rtcm3MessageType::Msm5Beidou {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                _ => {
                    // 对于其他卫星系统，返回Raw类型
                    let mut raw_data = Vec::new();
                    raw_data.extend_from_slice(&[(msg_type >> 8) as u8, (msg_type & 0xFF) as u8]);
                    Ok(Rtcm3MessageType::Raw {
                        msg_type,
                        payload: raw_data,
                    })
                }
            }
        },
        MsmType::FullPhaserangesPseudoranges => {
            // MSM6
            match header.gnss_type {
                GnssType::Gps => Ok(Rtcm3MessageType::Msm6Gps {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Glonass => Ok(Rtcm3MessageType::Msm6Glonass {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Galileo => Ok(Rtcm3MessageType::Msm6Galileo {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Beidou => Ok(Rtcm3MessageType::Msm6Beidou {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                _ => {
                    // 对于其他卫星系统，返回Raw类型
                    let mut raw_data = Vec::new();
                    raw_data.extend_from_slice(&[(msg_type >> 8) as u8, (msg_type & 0xFF) as u8]);
                    Ok(Rtcm3MessageType::Raw {
                        msg_type,
                        payload: raw_data,
                    })
                }
            }
        },
        MsmType::FullPhaserangesPseudorangesRates => {
            // MSM7
            match header.gnss_type {
                GnssType::Gps => Ok(Rtcm3MessageType::Msm7Gps {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Glonass => Ok(Rtcm3MessageType::Msm7Glonass {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Galileo => Ok(Rtcm3MessageType::Msm7Galileo {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                GnssType::Beidou => Ok(Rtcm3MessageType::Msm7Beidou {
                    station_id: header.station_id,
                    epoch_time: header.epoch_time,
                    satellites,
                    signals,
                }),
                _ => {
                    // 对于其他卫星系统，返回Raw类型
                    let mut raw_data = Vec::new();
                    raw_data.extend_from_slice(&[(msg_type >> 8) as u8, (msg_type & 0xFF) as u8]);
                    Ok(Rtcm3MessageType::Raw {
                        msg_type,
                        payload: raw_data,
                    })
                }
            }
        },
        _ => {
            // 不支持的MSM类型（MSM1/2/3）
            let mut raw_data = Vec::new();
            raw_data.extend_from_slice(&[(msg_type >> 8) as u8, (msg_type & 0xFF) as u8]);
            Ok(Rtcm3MessageType::Raw {
                msg_type,
                payload: raw_data,
            })
        }
    }
}

/// 解析MSM公共部分，包括头部和数据，对应RTKLIB的decode_msm_head
fn parse_msm_common(bit_reader: &mut BitReader, msg_type: u16, msm_type: MsmType) -> Result<(MsmHeader, Vec<MsmSatelliteData>, Vec<MsmSignalData>)> {
    // 跳过已经解析的消息类型（在调用方已处理）
    // bit_reader.skip_bits(12)?;

    // 解析站点ID (12位)
    let station_id = bit_reader.read_bits(12)? as u16;
    
    // 获取GNSS系统类型
    let gnss_type = get_gnss_type(msg_type);
    
    // 根据不同的卫星系统解析时间
    let epoch_time = match gnss_type {
        GnssType::Glonass => {
            // GLONASS时间：天(3位) + 毫秒(27位)
            let dow = bit_reader.read_bits(3)? as u8;
            let tod = bit_reader.read_bits(27)? as f64 * 0.001; // 转换为秒
            // 创建GNSS时间
            GnssTime::new_glonass(dow as i32, tod)
        },
        GnssType::Beidou => {
            // BeiDou时间：周内秒(30位)，需加14秒转为GPST
            let tow = bit_reader.read_bits(30)? as f64 * 0.001 + 14.0; // 转换为秒并加14秒
            // 创建GNSS时间
            GnssTime::new_gps(0, tow) // 周数在后续处理中调整
        },
        _ => {
            // GPS/Galileo/SBAS/QZSS时间：周内秒(30位)
            let tow = bit_reader.read_bits(30)? as f64 * 0.001; // 转换为秒
            // 创建GNSS时间
            GnssTime::new_gps(0, tow) // 周数在后续处理中调整
        }
    };
    
    // 多消息标志(1位)
    let multiple_message = bit_reader.read_bit()?;
    
    // 问题卫星标志(3位)
    let issue_of_data_station = bit_reader.read_bits(3)? as u8;
    
    // 平滑指示器(1位)和平滑间隔(3位)
    let smooth_indicator = bit_reader.read_bit()? as u8;
    let smooth_interval = bit_reader.read_bits(3)? as u8;
    
    // 卫星掩码(64位)
    let satellite_mask = bit_reader.read_bits(64)? as u64;
    
    // 信号掩码(32位)
    let signal_mask = bit_reader.read_bits(32)? as u32;
    
    // 计算卫星数和信号数
    let satellite_count = satellite_mask.count_ones() as usize;
    let signal_count = signal_mask.count_ones() as usize;
    
    // 单元掩码(satellite_count * signal_count位)
    let mut cell_mask = Vec::with_capacity(satellite_count * signal_count);
    for _ in 0..(satellite_count * signal_count) {
        cell_mask.push(bit_reader.read_bit()? as u8);
    }
    
    // 创建MSM头部
    let header = MsmHeader {
        station_id,
        gnss_type,
        msm_type,
        epoch_time,
        multiple_message,
        issue_of_data_station,
        smooth_indicator,
        smooth_interval,
        satellite_mask,
        signal_mask,
        cell_mask,
    };
    
    // 解析卫星数据
    let satellites = parse_satellite_data(bit_reader, &header, satellite_count)?;
    
    // 解析信号数据
    let signals = parse_signal_data(bit_reader, &header, satellite_count, signal_count)?;
    
    Ok((header, satellites, signals))
}

/// 解析卫星数据
fn parse_satellite_data(bit_reader: &mut BitReader, header: &MsmHeader, satellite_count: usize) -> Result<Vec<MsmSatelliteData>> {
    let mut satellites = Vec::with_capacity(satellite_count);
    
    // 提取卫星ID
    let satellite_ids = extract_satellite_ids(header.satellite_mask);
    
    // 整数毫秒范围 (伪距的整数毫秒部分)
    let mut rough_ranges = vec![0.0; satellite_count];
    for i in 0..satellite_count {
        let rng = bit_reader.read_bits(8)? as u16;
        if rng != 255 {
            rough_ranges[i] = rng as f64 * RANGE_MS;
        }
    }
    
    // 扩展信息（对于MSM4-7）
    let mut extended_info = vec![15; satellite_count];
    if matches!(header.msm_type, MsmType::FullPseudoranges | MsmType::FullPseudorangesRates | MsmType::FullPhaserangesPseudoranges | MsmType::FullPhaserangesPseudorangesRates) {
        for i in 0..satellite_count {
            extended_info[i] = bit_reader.read_bits(4)? as u8;
        }
    }
    
    // 毫秒范围的小数部分
    for i in 0..satellite_count {
        if rough_ranges[i] != 0.0 {
            let rng_m = bit_reader.read_bits(10)? as u16;
            rough_ranges[i] += rng_m as f64 * P2_10 * RANGE_MS;
        } else {
            bit_reader.skip_bits(10)?;
        }
    }
    
    // 相位范围率（对于MSM2、MSM5、MSM7）
    let mut phase_range_rates = if matches!(header.msm_type, MsmType::CompactPhaseRangeRate | MsmType::FullPseudorangesRates | MsmType::FullPhaserangesPseudorangesRates) {
        let mut rates = vec![None; satellite_count];
        for i in 0..satellite_count {
            let rate = bit_reader.read_bits_signed(14)? as i16;
            if rate != -8192 {
                rates[i] = Some(rate as f32);
            }
        }
        rates
    } else {
        vec![None; satellite_count]
    };
    
    // 组装卫星数据
    for i in 0..satellite_count {
        satellites.push(MsmSatelliteData {
            satellite_id: satellite_ids[i],
            rough_range_ms: rough_ranges[i],
            phase_range_rate: phase_range_rates[i],
        });
    }
    
    Ok(satellites)
}

/// 解析信号数据
fn parse_signal_data(bit_reader: &mut BitReader, header: &MsmHeader, satellite_count: usize, signal_count: usize) -> Result<Vec<MsmSignalData>> {
    // 从单元掩码确定有效的卫星-信号组合
    let cell_count = header.cell_mask.iter().filter(|&&b| b == 1).count();
    let mut signals = Vec::with_capacity(cell_count);
    
    // 为每个信号提取ID
    let signal_ids = extract_signal_ids(header.signal_mask);
    
    // 根据MSM类型确定各字段的位宽
    let (pseudorange_bits, pseudorange_resolution) = match header.msm_type {
        MsmType::FullPseudoranges | 
        MsmType::FullPseudorangesRates | 
        MsmType::FullPhaserangesPseudoranges | 
        MsmType::FullPhaserangesPseudorangesRates => (20, P2_29), // 高精度伪距
        _ => (15, P2_24), // 紧凑伪距
    };
    
    let (phaserange_bits, phaserange_resolution) = match header.msm_type {
        MsmType::FullPhaserangesPseudoranges | 
        MsmType::FullPhaserangesPseudorangesRates => (24, P2_31), // 高精度相位
        _ => (20, P2_29), // 紧凑相位
    };
    
    // 检查是否有各类型数据
    let has_pseudoranges = matches!(
        header.msm_type,
        MsmType::Compact | 
        MsmType::CompactPhaseRange | 
        MsmType::FullPseudoranges | 
        MsmType::FullPseudorangesRates | 
        MsmType::FullPhaserangesPseudoranges | 
        MsmType::FullPhaserangesPseudorangesRates
    );
    
    let has_phaseranges = matches!(
        header.msm_type,
        MsmType::CompactPhaseRange | 
        MsmType::FullPseudoranges | 
        MsmType::FullPseudorangesRates | 
        MsmType::FullPhaserangesPseudoranges | 
        MsmType::FullPhaserangesPseudorangesRates
    );
    
    // 解析数据，先读所有伪距，再读所有相位...
    // 构建cell_indices数组，记录每个有效cell的卫星和信号索引
    let mut cell_indices = Vec::with_capacity(cell_count);
    let mut idx = 0;
    for sat_idx in 0..satellite_count {
        for sig_idx in 0..signal_count {
            if idx < header.cell_mask.len() && header.cell_mask[idx] == 1 {
                cell_indices.push((sat_idx, sig_idx));
            }
            idx += 1;
        }
    }
    
    // 为所有cell分配空间
    let mut pseudoranges = vec![None; cell_count];
    let mut phaseranges = vec![None; cell_count];
    let mut lock_indicators = vec![None; cell_count];
    let mut half_cycles = vec![None; cell_count];
    let mut cnrs = vec![None; cell_count];
    
    // 读取伪距
    if has_pseudoranges {
        for i in 0..cell_count {
            let prv = bit_reader.read_bits_signed(pseudorange_bits)? as i32;
            let invalid_value = match pseudorange_bits {
                20 => -524288,
                15 => -16384,
                _ => -1
            };
            if prv != invalid_value {
                pseudoranges[i] = Some(prv as f64 * pseudorange_resolution * RANGE_MS);
            }
        }
    }
    
    // 读取相位
    if has_phaseranges {
        for i in 0..cell_count {
            let cpv = bit_reader.read_bits_signed(phaserange_bits)? as i32;
            let invalid_value = match phaserange_bits {
                24 => -8388608,
                20 => -524288,
                _ => -1
            };
            if cpv != invalid_value {
                phaseranges[i] = Some(cpv as f64 * phaserange_resolution * RANGE_MS);
            }
        }
        
        // 锁时间指示器
        let lock_bits = if matches!(header.msm_type, MsmType::FullPseudoranges | MsmType::FullPseudorangesRates | MsmType::FullPhaserangesPseudoranges | MsmType::FullPhaserangesPseudorangesRates) {
            10 // 高精度锁时间
        } else {
            4 // 紧凑锁时间
        };
        
        for i in 0..cell_count {
            lock_indicators[i] = Some(bit_reader.read_bits(lock_bits)? as u8);
        }
        
        // 半周期歧义度指示器
        for i in 0..cell_count {
            half_cycles[i] = Some(bit_reader.read_bit()?);
        }
    }
    
    // 载噪比
    let cnr_bits = if matches!(header.msm_type, MsmType::FullPseudoranges | MsmType::FullPseudorangesRates | MsmType::FullPhaserangesPseudoranges | MsmType::FullPhaserangesPseudorangesRates) {
        10 // 高精度CNR
    } else {
        6 // 紧凑CNR
    };
    
    for i in 0..cell_count {
        let cnr_val = bit_reader.read_bits(cnr_bits)? as u16;
        cnrs[i] = if cnr_bits == 10 {
            Some(cnr_val as f32 * 0.0625) // 1/16 dB-Hz
        } else {
            Some(cnr_val as f32) // dB-Hz
        };
    }
    
    // 组装信号数据
    for i in 0..cell_count {
        let (sat_idx, sig_idx) = cell_indices[i];
        signals.push(MsmSignalData {
            satellite_index: sat_idx as u8,
            signal_index: sig_idx as u8,
            fine_pseudorange: pseudoranges[i],
            fine_phaserange: phaseranges[i],
            lock_time_indicator: lock_indicators[i],
            half_cycle_ambiguity: half_cycles[i],
            cnr: cnrs[i],
        });
    }
    
    Ok(signals)
}

/// 从卫星掩码提取卫星ID
fn extract_satellite_ids(satellite_mask: u64) -> Vec<u8> {
    let mut ids = Vec::new();
    for i in 0..64 {
        if (satellite_mask & (1 << i)) != 0 {
            ids.push((i + 1) as u8);
        }
    }
    ids
}

/// 从信号掩码提取信号ID
fn extract_signal_ids(signal_mask: u32) -> Vec<u8> {
    let mut ids = Vec::new();
    for i in 0..32 {
        if (signal_mask & (1 << i)) != 0 {
            ids.push((i + 1) as u8);
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_satellite_ids() {
        // 直接调用 extract_satellite_ids 函数，不再使用 MsmParser
        let ids = extract_satellite_ids(0x0000_0000_0000_0003); // 第1和第2颗卫星
        assert_eq!(ids, vec![1, 2]);
    }
    
    #[test]
    fn test_extract_signal_ids() {
        // 直接调用 extract_signal_ids 函数，不再使用 MsmParser
        let ids = extract_signal_ids(0x0000_0005); // 第1和第3个信号
        assert_eq!(ids, vec![1, 3]);
    }
    
    #[test]
    fn test_msm_message_types() {
        // 创建一个通用的MSM头部
        let header = MsmHeader {
            station_id: 1024,
            gnss_type: GnssType::Gps,
            msm_type: MsmType::FullPseudoranges, // MSM4
            epoch_time: GnssTime::new_gps(2000, 345600.0),
            multiple_message: false,
            issue_of_data_station: 0,
            smooth_indicator: 0,
            smooth_interval: 0,
            satellite_mask: 0x0000000000000003,
            signal_mask: 0x00000005,
            cell_mask: vec![1, 0, 1, 0],
        };
        
        // 创建卫星和信号数据
        let satellites = vec![
            MsmSatelliteData {
                satellite_id: 1,
                rough_range_ms: 20000.0,
                phase_range_rate: None,
            },
            MsmSatelliteData {
                satellite_id: 2,
                rough_range_ms: 22000.0,
                phase_range_rate: None,
            },
        ];
        
        let signals = vec![
            MsmSignalData {
                satellite_index: 0,
                signal_index: 0,
                fine_pseudorange: Some(1.5),
                fine_phaserange: Some(2.5),
                lock_time_indicator: Some(3),
                half_cycle_ambiguity: Some(false),
                cnr: Some(45.0),
            },
            MsmSignalData {
                satellite_index: 1,
                signal_index: 2,
                fine_pseudorange: Some(1.6),
                fine_phaserange: Some(2.6),
                lock_time_indicator: Some(4),
                half_cycle_ambiguity: Some(true),
                cnr: Some(46.0),
            },
        ];
        
        // 测试MSM4
        let mut header_msm4 = header.clone();
        header_msm4.msm_type = MsmType::FullPseudoranges;
        let result_msm4 = create_msm_message(1074, header_msm4, satellites.clone(), signals.clone());
        assert!(result_msm4.is_ok());
        match result_msm4.unwrap() {
            Rtcm3MessageType::Msm4Gps { station_id, .. } => {
                assert_eq!(station_id, 1024);
            },
            _ => panic!("Expected MSM4 GPS message type"),
        }
        
        // 测试MSM5
        let mut header_msm5 = header.clone();
        header_msm5.msm_type = MsmType::FullPseudorangesRates;
        let result_msm5 = create_msm_message(1075, header_msm5, satellites.clone(), signals.clone());
        assert!(result_msm5.is_ok());
        match result_msm5.unwrap() {
            Rtcm3MessageType::Msm5Gps { station_id, .. } => {
                assert_eq!(station_id, 1024);
            },
            _ => panic!("Expected MSM5 GPS message type"),
        }
        
        // 测试MSM6
        let mut header_msm6 = header.clone();
        header_msm6.msm_type = MsmType::FullPhaserangesPseudoranges;
        let result_msm6 = create_msm_message(1076, header_msm6, satellites.clone(), signals.clone());
        assert!(result_msm6.is_ok());
        match result_msm6.unwrap() {
            Rtcm3MessageType::Msm6Gps { station_id, .. } => {
                assert_eq!(station_id, 1024);
            },
            _ => panic!("Expected MSM6 GPS message type"),
        }
        
        // 测试MSM7
        let mut header_msm7 = header.clone();
        header_msm7.msm_type = MsmType::FullPhaserangesPseudorangesRates;
        let result_msm7 = create_msm_message(1077, header_msm7, satellites.clone(), signals.clone());
        assert!(result_msm7.is_ok());
        match result_msm7.unwrap() {
            Rtcm3MessageType::Msm7Gps { station_id, .. } => {
                assert_eq!(station_id, 1024);
            },
            _ => panic!("Expected MSM7 GPS message type"),
        }
        
        // 测试GLONASS消息
        let mut header_glonass = header.clone();
        header_glonass.gnss_type = GnssType::Glonass;
        header_glonass.msm_type = MsmType::FullPseudoranges;
        let result_glonass = create_msm_message(1084, header_glonass, satellites.clone(), signals.clone());
        assert!(result_glonass.is_ok());
        match result_glonass.unwrap() {
            Rtcm3MessageType::Msm4Glonass { station_id, .. } => {
                assert_eq!(station_id, 1024);
            },
            _ => panic!("Expected MSM4 GLONASS message type"),
        }
        
        // 测试Galileo消息
        let mut header_galileo = header.clone();
        header_galileo.gnss_type = GnssType::Galileo;
        header_galileo.msm_type = MsmType::FullPseudoranges;
        let result_galileo = create_msm_message(1094, header_galileo, satellites.clone(), signals.clone());
        assert!(result_galileo.is_ok());
        match result_galileo.unwrap() {
            Rtcm3MessageType::Msm4Galileo { station_id, .. } => {
                assert_eq!(station_id, 1024);
            },
            _ => panic!("Expected MSM4 Galileo message type"),
        }
        
        // 测试BeiDou消息
        let mut header_beidou = header.clone();
        header_beidou.gnss_type = GnssType::Beidou;
        header_beidou.msm_type = MsmType::FullPseudoranges;
        let result_beidou = create_msm_message(1124, header_beidou, satellites.clone(), signals.clone());
        assert!(result_beidou.is_ok());
        match result_beidou.unwrap() {
            Rtcm3MessageType::Msm4Beidou { station_id, .. } => {
                assert_eq!(station_id, 1024);
            },
            _ => panic!("Expected MSM4 BeiDou message type"),
        }
    }
} 