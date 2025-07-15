/*!
 * RTCM2.x格式解析模块
 * 
 * 实现RTCM2.x消息格式的解析功能。
 * RTCM2.x的消息格式包括:
 * - 消息头: 固定6字节
 * - 消息体: 可变长度，最长为1023字节
 * - 奇偶校验: 固定3字节
 */

use crate::util::bits::BitReader;
use thiserror::Error;
use std::io;

/// RTCM2.x错误类型
#[derive(Debug, Error)]
pub enum Rtcm2Error {
    #[error("同步错误: 未找到前导码")]
    SyncError,

    #[error("CRC校验失败")]
    CrcError,

    #[error("消息长度无效: {0}")]
    InvalidLength(usize),

    #[error("消息类型无效: {0}")]
    InvalidType(u16),

    #[error("I/O错误: {0}")]
    IoError(#[from] io::Error),

    #[error("数据解析错误: {0}")]
    ParseError(String),
}

/// RTCM2.x消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rtcm2MessageType {
    /// 类型1: 差分GPS修正
    Type1,
    
    /// 类型2: 差分GPS修正增量
    Type2,
    
    /// 类型3: GPS参考站参数
    Type3,
    
    /// 类型9: GPS部分修正集
    Type9,
    
    /// 类型18: RTK非差分载波相位修正
    Type18,
    
    /// 类型19: RTK非差分伪距修正
    Type19,
    
    /// 其他类型
    Other(u16),
}

impl From<u16> for Rtcm2MessageType {
    fn from(type_num: u16) -> Self {
        match type_num {
            1 => Rtcm2MessageType::Type1,
            2 => Rtcm2MessageType::Type2,
            3 => Rtcm2MessageType::Type3,
            9 => Rtcm2MessageType::Type9,
            18 => Rtcm2MessageType::Type18,
            19 => Rtcm2MessageType::Type19,
            _ => Rtcm2MessageType::Other(type_num),
        }
    }
}

/// RTCM2.x消息头
#[derive(Debug, Clone)]
pub struct Rtcm2Header {
    /// 消息类型
    pub message_type: Rtcm2MessageType,
    
    /// 参考站ID
    pub station_id: u16,
    
    /// Z计数（GPS时间参考）
    pub z_count: u16,
    
    /// 消息序列号
    pub sequence_number: u8,
    
    /// 消息长度（字）
    pub length: u16,
    
    /// 站点健康状态
    pub station_health: u8,
}

/// RTCM2.x帧解析器
#[derive(Debug)]
pub struct Rtcm2Parser {
    /// 同步状态
    sync_state: SyncState,
    
    /// 当前缓冲区
    buffer: Vec<u8>,
    
    /// 消息长度（字，1字=30位，包括24位数据和6位奇偶校验）
    message_length: usize,
}

/// 同步状态
#[derive(Debug)]
enum SyncState {
    /// 查找前导码
    FindPreamble,
    
    /// 读取消息
    ReadingMessage,
}

impl Rtcm2Parser {
    /// 创建新的RTCM2解析器
    pub fn new() -> Self {
        Self {
            sync_state: SyncState::FindPreamble,
            buffer: Vec::with_capacity(1024),
            message_length: 0,
        }
    }
    
    /// 处理单个字节
    pub fn process_byte(&mut self, byte: u8) -> Result<Option<Rtcm2Message>, Rtcm2Error> {
        match self.sync_state {
            SyncState::FindPreamble => {
                // RTCM2.x前导码为0x66
                if byte == 0x66 {
                    self.buffer.clear();
                    self.buffer.push(byte);
                    self.sync_state = SyncState::ReadingMessage;
                }
                Ok(None)
            },
            SyncState::ReadingMessage => {
                self.buffer.push(byte);
                
                // 至少需要3字节才能读取消息长度
                if self.buffer.len() == 3 {
                    // 消息长度在第2、3字节的部分位中
                    let mut bit_reader = BitReader::new(&self.buffer[1..3]);
                    bit_reader.skip_bits(6)?; // 跳过前6位
                    let length_bits = bit_reader.read_bits_u32(10)?;
                    self.message_length = length_bits as usize;
                }
                
                // 检查是否接收到完整消息
                // RTCM2.x格式：1字节前导码 + 2字节控制信息 + N字节数据 + 3字节奇偶校验
                let total_bytes = 3 + self.message_length * 3 + 3;
                if self.buffer.len() == total_bytes {
                    // 验证CRC
                    if !self.verify_crc() {
                        self.sync_state = SyncState::FindPreamble;
                        return Err(Rtcm2Error::CrcError);
                    }
                    
                    // 解析消息
                    let message = self.parse_message()?;
                    
                    // 重置状态
                    self.sync_state = SyncState::FindPreamble;
                    
                    Ok(Some(message))
                } else {
                    Ok(None)
                }
            }
        }
    }
    
    /// 验证CRC
    fn verify_crc(&self) -> bool {
        // RTCM2.x使用24位校验和，最后3字节包含校验和
        if self.buffer.len() < 6 {  // 至少需要头部和校验和
            return false;
        }
        
        let data_len = self.buffer.len() - 3;  // 排除最后3字节的校验和
        let received_crc = (self.buffer[data_len] as u32) << 16 |
                           (self.buffer[data_len+1] as u32) << 8 |
                           (self.buffer[data_len+2] as u32);
        
        let calculated_crc = self.calculate_crc24q(&self.buffer[0..data_len]);
        
        received_crc == calculated_crc
    }
    
    /// 计算CRC-24Q校验和
    /// RTCM2.x和RTCM3.x都使用CRC-24Q，多项式为0x1864CFB
    fn calculate_crc24q(&self, data: &[u8]) -> u32 {
        // CRC-24Q多项式
        const CRC24Q: u32 = 0x1864CFB;
        
        let mut crc = 0;
        
        for &byte in data {
            crc ^= (byte as u32) << 16;
            
            for _ in 0..8 {
                crc <<= 1;
                if (crc & 0x1000000) != 0 {
                    crc ^= CRC24Q;
                }
            }
        }
        
        crc & 0xFFFFFF  // 保留低24位
    }
    
    /// 解析消息
    fn parse_message(&self) -> Result<Rtcm2Message, Rtcm2Error> {
        // 解析消息头
        let header = self.parse_header()?;
        
        // 根据消息类型解析消息体
        let body = match header.message_type {
            Rtcm2MessageType::Type1 => {
                Rtcm2MessageBody::Type1(self.parse_type1()?)
            },
            Rtcm2MessageType::Type2 => {
                Rtcm2MessageBody::Type2(self.parse_type2()?)
            },
            Rtcm2MessageType::Type3 => {
                Rtcm2MessageBody::Type3(self.parse_type3()?)
            },
            // 为简单起见，其他类型暂不实现
            _ => Rtcm2MessageBody::Unknown(header.message_type)
        };
        
        Ok(Rtcm2Message { header, body })
    }
    
    /// 解析消息头
    fn parse_header(&self) -> Result<Rtcm2Header, Rtcm2Error> {
        let mut bit_reader = BitReader::new(&self.buffer[1..3]);
        
        // 消息类型（6位）
        let type_num = bit_reader.read_bits_u32(6)? as u16;
        let message_type = Rtcm2MessageType::from(type_num);
        
        // 站点ID（10位）
        let station_id = bit_reader.read_bits_u32(10)? as u16;
        
        // 修改Z计数（13位）
        let z_count = {
            let mut r = BitReader::new(&self.buffer[3..]);
            r.read_bits_u32(13)? as u16
        };
        
        // 序列号（3位）
        let sequence_number = {
            let mut r = BitReader::new(&self.buffer[4..]);
            r.skip_bits(3)?;
            r.read_bits_u32(3)? as u8
        };
        
        // 消息长度（5位）
        let length = {
            let mut r = BitReader::new(&self.buffer[5..]);
            r.read_bits_u32(5)? as u16
        };
        
        // 站点健康（3位）
        let station_health = {
            let mut r = BitReader::new(&self.buffer[5..]);
            r.skip_bits(5)?;
            r.read_bits_u32(3)? as u8
        };
        
        Ok(Rtcm2Header {
            message_type,
            station_id,
            z_count,
            sequence_number,
            length,
            station_health,
        })
    }
    
    /// 解析类型1消息
    fn parse_type1(&self) -> Result<Rtcm2Type1, Rtcm2Error> {
        // 位置：开始于消息头(6字节)之后
        let data_start = 6;
        
        if self.buffer.len() <= data_start {
            return Err(Rtcm2Error::ParseError("Message too short for Type 1".to_string()));
        }
        
        // 创建BitReader从数据部分开始
        let mut reader = BitReader::new(&self.buffer[data_start..]);
        
        // 读取比例因子(1位)
        let scale_factor = match reader.read_bits_u32(1) {
            Ok(factor) => factor as u8,
            Err(e) => return Err(Rtcm2Error::ParseError(format!("Failed to read scale factor: {}", e))),
        };
        
        // 读取UDRE系数和卫星数量
        let mut udre = Vec::new();
        let mut satellite_id = Vec::new();
        let mut prc = Vec::new();
        let mut rrc = Vec::new();
        
        // RTCM2 Type1最多可以包含31颗卫星，但通常会分成多个消息发送
        // 这里需要计算剩余数据可以包含多少颗卫星
        // 每颗卫星数据占用：2位UDRE + 5位卫星ID + 16位PRC + 8位RRC = 31位
        // 每字节8位，消息结束前有24位CRC
        
        // 计算可能包含的最大卫星数量（考虑已读取的比例因子1位）
        let remaining_bits = (self.buffer.len() - data_start) * 8 - 1 - 24;  // 减去CRC
        let max_satellites = remaining_bits / 31;  // 31位/卫星
        
        for _ in 0..max_satellites {
            // 检查是否还有足够的数据
            if !reader.has_more() {
                break;
            }
            
            // 读取UDRE (2位)
            let udre_val = match reader.read_bits_u32(2) {
                Ok(val) => val as u8,
                Err(_) => break, // 数据不足，退出循环
            };
            
            // 读取卫星ID (5位)
            let sat_id = match reader.read_bits_u32(5) {
                Ok(val) => val as u8,
                Err(_) => break,
            };
            
            // 如果卫星ID为0，表示数据结束
            if sat_id == 0 {
                break;
            }
            
            // 读取伪距修正值 (16位)
            let prc_val = match reader.read_bits_i32(16) {
                Ok(val) => {
                    // 根据比例因子转换为物理单位
                    if scale_factor == 0 {
                        val as f64 * 0.02  // 比例因子0: 0.02米
                    } else {
                        val as f64 * 0.32  // 比例因子1: 0.32米
                    }
                },
                Err(_) => break,
            };
            
            // 读取伪距修正变化率 (8位)
            let rrc_val = match reader.read_bits_i32(8) {
                Ok(val) => {
                    // 转换为物理单位：米/秒
                    if scale_factor == 0 {
                        val as f64 * 0.002  // 比例因子0: 0.002米/秒
                    } else {
                        val as f64 * 0.032  // 比例因子1: 0.032米/秒
                    }
                },
                Err(_) => break,
            };
            
            // 存储数据
            udre.push(udre_val);
            satellite_id.push(sat_id);
            prc.push(prc_val);
            rrc.push(rrc_val);
        }
        
        Ok(Rtcm2Type1 {
            scale_factor,
            udre,
            satellite_id,
            prc,
            rrc,
        })
    }
    
    /// 解析类型2消息
    fn parse_type2(&self) -> Result<Rtcm2Type2, Rtcm2Error> {
        // 位置：开始于消息头(6字节)之后
        let data_start = 6;
        
        if self.buffer.len() <= data_start {
            return Err(Rtcm2Error::ParseError("Message too short for Type 2".to_string()));
        }
        
        // 创建BitReader从数据部分开始
        let mut reader = BitReader::new(&self.buffer[data_start..]);
        
        // 读取比例因子(1位)
        let scale_factor = match reader.read_bits_u32(1) {
            Ok(factor) => factor as u8,
            Err(e) => return Err(Rtcm2Error::ParseError(format!("Failed to read scale factor: {}", e))),
        };
        
        // 读取UDRE系数和卫星数量
        let mut udre = Vec::new();
        let mut satellite_id = Vec::new();
        let mut delta_prc = Vec::new();
        let mut delta_rrc = Vec::new();
        
        // 类型2消息结构与类型1类似，但数据字段表示增量
        // 每颗卫星数据占用：2位UDRE + 5位卫星ID + 16位deltaPRC + 8位deltaRRC = 31位
        
        // 计算可能包含的最大卫星数量（考虑已读取的比例因子1位）
        let remaining_bits = (self.buffer.len() - data_start) * 8 - 1 - 24;  // 减去CRC
        let max_satellites = remaining_bits / 31;  // 31位/卫星
        
        for _ in 0..max_satellites {
            // 检查是否还有足够的数据
            if !reader.has_more() {
                break;
            }
            
            // 读取UDRE (2位)
            let udre_val = match reader.read_bits_u32(2) {
                Ok(val) => val as u8,
                Err(_) => break, // 数据不足，退出循环
            };
            
            // 读取卫星ID (5位)
            let sat_id = match reader.read_bits_u32(5) {
                Ok(val) => val as u8,
                Err(_) => break,
            };
            
            // 如果卫星ID为0，表示数据结束
            if sat_id == 0 {
                break;
            }
            
            // 读取伪距修正增量 (16位)
            let delta_prc_val = match reader.read_bits_i32(16) {
                Ok(val) => {
                    // 根据比例因子转换为物理单位
                    if scale_factor == 0 {
                        val as f64 * 0.02  // 比例因子0: 0.02米
                    } else {
                        val as f64 * 0.32  // 比例因子1: 0.32米
                    }
                },
                Err(_) => break,
            };
            
            // 读取伪距修正变化率增量 (8位)
            let delta_rrc_val = match reader.read_bits_i32(8) {
                Ok(val) => {
                    // 转换为物理单位：米/秒
                    if scale_factor == 0 {
                        val as f64 * 0.002  // 比例因子0: 0.002米/秒
                    } else {
                        val as f64 * 0.032  // 比例因子1: 0.032米/秒
                    }
                },
                Err(_) => break,
            };
            
            // 存储数据
            udre.push(udre_val);
            satellite_id.push(sat_id);
            delta_prc.push(delta_prc_val);
            delta_rrc.push(delta_rrc_val);
        }
        
        Ok(Rtcm2Type2 {
            scale_factor,
            udre,
            satellite_id,
            delta_prc,
            delta_rrc,
        })
    }
    
    /// 解析类型3消息
    fn parse_type3(&self) -> Result<Rtcm2Type3, Rtcm2Error> {
        // 位置：开始于消息头(6字节)之后
        let data_start = 6;
        
        if self.buffer.len() <= data_start + 12 { // 至少需要头部+X/Y/Z坐标(每个坐标24位)
            return Err(Rtcm2Error::ParseError("Message too short for Type 3".to_string()));
        }
        
        // 创建BitReader从数据部分开始
        let mut reader = BitReader::new(&self.buffer[data_start..]);
        
        // X坐标 (24位)
        let x = match reader.read_bits_i32(24) {
            Ok(val) => val as f64 * 0.01,  // 单位：0.01米
            Err(e) => return Err(Rtcm2Error::ParseError(format!("Failed to read X coordinate: {}", e))),
        };
        
        // Y坐标 (24位)
        let y = match reader.read_bits_i32(24) {
            Ok(val) => val as f64 * 0.01,  // 单位：0.01米
            Err(e) => return Err(Rtcm2Error::ParseError(format!("Failed to read Y coordinate: {}", e))),
        };
        
        // Z坐标 (24位)
        let z = match reader.read_bits_i32(24) {
            Ok(val) => val as f64 * 0.01,  // 单位：0.01米
            Err(e) => return Err(Rtcm2Error::ParseError(format!("Failed to read Z coordinate: {}", e))),
        };
        
        Ok(Rtcm2Type3 {
            x,
            y,
            z,
        })
    }
}

/// RTCM2消息
#[derive(Debug, Clone)]
pub struct Rtcm2Message {
    /// 消息头
    pub header: Rtcm2Header,
    
    /// 消息体
    pub body: Rtcm2MessageBody,
}

/// RTCM2消息体
#[derive(Debug, Clone)]
pub enum Rtcm2MessageBody {
    /// 类型1: 差分GPS修正
    Type1(Rtcm2Type1),
    
    /// 类型2: 差分GPS修正增量
    Type2(Rtcm2Type2),
    
    /// 类型3: GPS参考站参数
    Type3(Rtcm2Type3),
    
    /// 未实现的类型
    Unknown(Rtcm2MessageType),
}

/// 类型1消息: 差分GPS修正
#[derive(Debug, Clone)]
pub struct Rtcm2Type1 {
    /// 比例因子
    pub scale_factor: u8,
    
    /// UDRE (User Differential Range Error)
    pub udre: Vec<u8>,
    
    /// 卫星ID
    pub satellite_id: Vec<u8>,
    
    /// 伪距修正值 (Pseudorange Correction)
    pub prc: Vec<f64>,
    
    /// 伪距修正变化率 (Range Rate Correction)
    pub rrc: Vec<f64>,
}

/// 类型2消息: 差分GPS修正增量
#[derive(Debug, Clone)]
pub struct Rtcm2Type2 {
    /// 比例因子
    pub scale_factor: u8,
    
    /// UDRE
    pub udre: Vec<u8>,
    
    /// 卫星ID
    pub satellite_id: Vec<u8>,
    
    /// 伪距修正增量
    pub delta_prc: Vec<f64>,
    
    /// 伪距修正变化率增量
    pub delta_rrc: Vec<f64>,
}

/// 类型3消息: GPS参考站参数
#[derive(Debug, Clone)]
pub struct Rtcm2Type3 {
    /// X坐标 (ECEF, 米)
    pub x: f64,
    
    /// Y坐标 (ECEF, 米)
    pub y: f64,
    
    /// Z坐标 (ECEF, 米)
    pub z: f64,
} 