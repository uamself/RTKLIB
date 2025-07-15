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
        })
    }
    
    /// 处理单个字节
    pub fn process_byte(&mut self, byte: u8) -> Result<Option<RtcmMessageType>, RtcmError> {
        // 检查是否为RTCM3前导码 (0xD3)
        if byte == 0xD3 && matches!(self.state, RtcmState::WaitingForPreamble) {
            // 使用RTCM3解析器处理
            match self.rtcm3_parser.process_byte(byte)? {
                Some(msg) => {
                    let msg_type = RtcmMessageType::Rtcm3(msg);
                    self.last_message_type = Some(msg_type.clone());
                    return Ok(Some(msg_type));
                },
                None => return Ok(None),
            }
        }
        // 检查是否为RTCM2前导码 (0x66)
        else if byte == 0x66 && matches!(self.state, RtcmState::WaitingForPreamble) {
            // 使用RTCM2解析器处理
            match self.rtcm2_parser.process_byte(byte)? {
                Some(msg) => {
                    let msg_type = RtcmMessageType::Rtcm2(msg.header.message_type);
                    self.last_message_type = Some(msg_type.clone());
                    return Ok(Some(msg_type));
                },
                None => return Ok(None),
            }
        } 
        
        // 如果已经在解析过程中
        if !matches!(self.state, RtcmState::WaitingForPreamble) {
            // 尝试使用RTCM3解析器
            if let Some(rtcm3_result) = self.rtcm3_parser.process_byte(byte).ok() {
                if let Some(msg) = rtcm3_result {
                    let msg_type = RtcmMessageType::Rtcm3(msg);
                    self.last_message_type = Some(msg_type.clone());
                    self.state = RtcmState::WaitingForPreamble;
                    return Ok(Some(msg_type));
                }
            }
            
            // 尝试使用RTCM2解析器
            match self.rtcm2_parser.process_byte(byte)? {
                Some(msg) => {
                    let msg_type = RtcmMessageType::Rtcm2(msg.header.message_type);
                    self.last_message_type = Some(msg_type.clone());
                    self.state = RtcmState::WaitingForPreamble;
                    return Ok(Some(msg_type));
                },
                None => return Ok(None),
            }
        }
        
        // 如果都不匹配，保存字节并返回无消息
        self.buffer.push(byte);
        Ok(None)
    }
} 