/*!
 * RTCM to RINEX conversion library
 * 
 * This library provides functionality to convert RTCM (Radio Technical Commission for Maritime Services)
 * format GNSS data to RINEX (Receiver Independent Exchange Format).
 * 
 * Based on the RTKLIB C library, reimplemented in Rust.
 */

// 导出各模块
pub mod rtcm;
pub mod rinex;
pub mod gnss;
pub mod util;

// 仅当使用FFI特性时导出FFI模块
#[cfg(feature = "ffi")]
pub mod ffi;

// 重新导出主要类型供用户直接使用
pub use crate::rtcm::{RtcmContext, RtcmError, RtcmMessageType};
pub use crate::rinex::{RinexOptions, RinexError};
pub use crate::gnss::time::GnssTime;

/// 错误类型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// RTCM处理错误
    #[error("RTCM error: {0}")]
    Rtcm(#[from] rtcm::RtcmError),
    
    /// RINEX处理错误
    #[error("RINEX error: {0}")]
    Rinex(#[from] rinex::RinexError),
    
    /// 输入/输出错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// 其他错误
    #[error("{0}")]
    Other(String),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// 公共API函数

/// 初始化RTCM处理上下文
///
/// # Returns
///
/// 返回一个新的RTCM上下文实例
pub fn init_rtcm() -> Result<RtcmContext> {
    RtcmContext::new().map_err(Error::Rtcm)
}

/// 处理单个RTCM数据字节
///
/// # Arguments
///
/// * `ctx` - RTCM处理上下文
/// * `data` - 输入的RTCM数据字节
///
/// # Returns
///
/// 处理结果，指示是否成功解析出消息
pub fn process_rtcm_data(ctx: &mut RtcmContext, data: u8) -> Result<Option<RtcmMessageType>> {
    ctx.process_byte(data).map_err(Error::Rtcm)
}

/// 处理RTCM文件
///
/// # Arguments
///
/// * `ctx` - RTCM处理上下文
/// * `file_path` - RTCM文件路径
///
/// # Returns
///
/// 处理结果
pub fn process_rtcm_file(ctx: &mut RtcmContext, file_path: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    use util::io;
    
    // 打开RTCM文件
    let mut file = io::open_file_read(file_path)?;
    
    // 逐字节处理文件内容
    let mut buffer = [0u8; 4096];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // 文件结束
        }
        
        // 处理每个字节
        for i in 0..bytes_read {
            let _ = ctx.process_byte(buffer[i])?;
        }
    }
    
    Ok(())
}

/// 转换RTCM数据到RINEX格式并写入文件
///
/// # Arguments
///
/// * `ctx` - RTCM处理上下文
/// * `options` - RINEX选项
/// * `output_path` - 输出RINEX文件路径
///
/// # Returns
///
/// 处理结果
pub fn convert_to_rinex(ctx: &RtcmContext, options: &RinexOptions, output_path: &str) -> Result<()> {
    // 这里是转换的实现
    // 目前是一个存根实现
    Err(Error::Other("Not implemented yet".to_string()))
}

/// 简单的文件转换函数
///
/// 使用默认选项将RTCM文件转换为RINEX文件
///
/// # Arguments
///
/// * `rtcm_file` - 输入RTCM文件路径
/// * `rinex_file` - 输出RINEX文件路径
/// * `rinex_version` - RINEX版本号（例如3.04）
///
/// # Returns
///
/// 处理结果
pub fn simple_convert(rtcm_file: &str, rinex_file: &str, rinex_version: f64) -> Result<()> {
    let mut ctx = init_rtcm()?;
    process_rtcm_file(&mut ctx, rtcm_file)?;
    
    let options = RinexOptions::new(rinex_version);
    convert_to_rinex(&ctx, &options, rinex_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // 基本测试框架
        assert!(true);
    }
}
