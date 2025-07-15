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

use chrono::Utc;
use std::collections::HashMap;
use std::io::BufWriter;
use std::path::Path;
use crate::rinex::header::{RinexHeader, RinexSystem, RinexFileType};
use crate::rinex::nav::NavData;

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

/// 转换选项
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// RINEX版本
    pub rinex_version: f64,
    
    /// 输出目录
    pub output_dir: Option<String>,
    
    /// 输出文件名前缀
    pub output_prefix: Option<String>,
    
    /// 卫星系统选择 (G=GPS, R=GLONASS, E=Galileo, C=BeiDou, J=QZSS, S=SBAS, I=IRNSS)
    pub systems: Vec<char>,
    
    /// 是否输出观测数据
    pub output_obs: bool,
    
    /// 是否输出导航数据
    pub output_nav: bool,
    
    /// 是否输出调试信息
    pub debug: bool,
    
    /// 是否压缩输出文件
    pub compress: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            rinex_version: 3.04,
            output_dir: None,
            output_prefix: None,
            systems: vec!['G', 'R', 'E', 'C', 'J', 'S', 'I'],
            output_obs: true,
            output_nav: true,
            debug: false,
            compress: false,
        }
    }
}

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
    let mut total_bytes = 0;
    let mut messages_processed = 0;
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // 文件结束
        }
        
        total_bytes += bytes_read;
        
        // 处理每个字节
        for i in 0..bytes_read {
            if let Some(_) = ctx.process_byte(buffer[i])? {
                messages_processed += 1;
            }
        }
    }
    
    if ctx.observations.is_empty() && ctx.gps_nav_data.is_empty() && ctx.glo_nav_data.is_empty() {
        return Err(Error::Other(format!(
            "No valid RTCM messages found in file. Processed {} bytes, found {} messages.",
            total_bytes, messages_processed
        )));
    }
    
    Ok(())
}

/// 转换RTCM数据到RINEX格式并写入文件
///
/// # Arguments
///
/// * `ctx` - RTCM处理上下文
/// * `options` - 转换选项
/// * `output_path` - 输出RINEX文件路径
///
/// # Returns
///
/// 处理结果
pub fn convert_to_rinex(ctx: &RtcmContext, options: &ConversionOptions, output_path: &str) -> Result<()> {
    // 检查是否有观测数据
    if ctx.observations.is_empty() && options.output_obs {
        return Err(Error::Other("No observation data available".to_string()));
    }
    
    // 检查是否有导航数据
    let has_nav_data = !ctx.gps_nav_data.is_empty() || 
                       !ctx.glo_nav_data.is_empty();
    
    if !has_nav_data && options.output_nav {
        println!("Warning: No navigation data available");
    }
    
    // 获取输出路径
    let path = Path::new(output_path);
    let file_stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    
    let output_dir = match &options.output_dir {
        Some(dir) => dir.clone(),
        None => path.parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".").to_string(),
    };
    
    let prefix = options.output_prefix.as_deref().unwrap_or(file_stem);
    
    // 生成观测数据文件
    if options.output_obs && !ctx.observations.is_empty() {
        let obs_output_path = format!("{}/{}.obs", output_dir, prefix);
        generate_observation_file(ctx, options, &obs_output_path)?;
    }
    
    // 如果有GPS导航数据，生成GPS导航数据文件
    if options.output_nav && !ctx.gps_nav_data.is_empty() && options.systems.contains(&'G') {
        let nav_output_path = format!("{}/{}.nav", output_dir, prefix);
        generate_navigation_file(ctx, options, &nav_output_path, RinexSystem::GPS)?;
    }
    
    // 如果有GLONASS导航数据，生成GLONASS导航数据文件
    if options.output_nav && !ctx.glo_nav_data.is_empty() && options.systems.contains(&'R') {
        let gnav_output_path = format!("{}/{}.glo", output_dir, prefix);
        generate_navigation_file(ctx, options, &gnav_output_path, RinexSystem::GLONASS)?;
    }
    
    // 如果需要压缩输出文件
    if options.compress {
        // 压缩观测数据文件
        if options.output_obs && !ctx.observations.is_empty() {
            let obs_path = format!("{}/{}.obs", output_dir, prefix);
            compress_file(&obs_path)?;
        }
        
        // 压缩GPS导航数据文件
        if options.output_nav && !ctx.gps_nav_data.is_empty() && options.systems.contains(&'G') {
            let nav_path = format!("{}/{}.nav", output_dir, prefix);
            compress_file(&nav_path)?;
        }
        
        // 压缩GLONASS导航数据文件
        if options.output_nav && !ctx.glo_nav_data.is_empty() && options.systems.contains(&'R') {
            let gnav_path = format!("{}/{}.glo", output_dir, prefix);
            compress_file(&gnav_path)?;
        }
    }
    
    Ok(())
}

/// 压缩文件
fn compress_file(file_path: &str) -> Result<()> {
    #[cfg(feature = "compression")]
    {
        use std::fs::File;
        use std::io::{Read, Write};
        use flate2::write::GzEncoder;
        use flate2::Compression;
        
        // 读取原始文件
        let mut input = File::open(file_path)?;
        let mut contents = Vec::new();
        input.read_to_end(&mut contents)?;
        
        // 创建压缩文件
        let gz_path = format!("{}.gz", file_path);
        let output = File::create(&gz_path)?;
        let mut encoder = GzEncoder::new(output, Compression::default());
        encoder.write_all(&contents)?;
        encoder.finish()?;
        
        // 删除原始文件
        std::fs::remove_file(file_path)?;
    }
    
    #[cfg(not(feature = "compression"))]
    {
        println!("Warning: Compression feature not enabled. File not compressed: {}", file_path);
    }
    
    Ok(())
}

/// 生成观测数据文件
fn generate_observation_file(ctx: &RtcmContext, options: &ConversionOptions, output_path: &str) -> Result<()> {
    use std::fs::File;
    
    if options.debug {
        println!("Generating observation file: {}", output_path);
    }
    
    // 创建输出文件
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    
    // 准备RINEX头部
    let mut header = RinexHeader::new(
        options.rinex_version, 
        RinexFileType::Observation, 
        match options.systems.len() {
            0 => RinexSystem::Mixed,
            1 => match options.systems[0] {
                'G' => RinexSystem::GPS,
                'R' => RinexSystem::GLONASS,
                'E' => RinexSystem::Galileo,
                'C' => RinexSystem::BeiDou,
                'J' => RinexSystem::QZSS,
                'S' => RinexSystem::SBAS,
                'I' => RinexSystem::IRNSS,
                _ => RinexSystem::Mixed,
            },
            _ => RinexSystem::Mixed,
        }
    );
    
    // 设置程序信息
    header.program = "rtcm2rinex".to_string();
    header.run_by = "RUST".to_string();
    header.date_created = Utc::now();
    
    // 设置站点信息
    if let Some(pos) = ctx.station_pos {
        header.set_position(pos.0, pos.1, pos.2);
    }
    
    // 设置天线信息
    if let Some(ref antenna_descriptor) = ctx.antenna_descriptor {
        header.antenna_type = antenna_descriptor.clone();
    }
    
    if let Some(ref antenna_serial) = ctx.antenna_serial {
        header.antenna_number = antenna_serial.clone();
    }
    
    // 设置接收机信息
    if let Some(ref receiver_type) = ctx.receiver_type {
        header.receiver_type = receiver_type.clone();
    }
    
    if let Some(ref receiver_serial) = ctx.receiver_serial {
        header.receiver_number = receiver_serial.clone();
    }
    
    if let Some(ref firmware_version) = ctx.firmware_version {
        header.receiver_version = firmware_version.clone();
    }
    
    // 设置天线高度
    if let Some(antenna_height) = ctx.antenna_height {
        header.antenna_height = antenna_height;
    }
    
    // 设置观测类型
    let mut obs_types = ctx.get_observation_types();
    
    // 根据选项过滤卫星系统
    obs_types.retain(|sys, _| options.systems.contains(sys));
    
    for (sys, types) in &obs_types {
        for obs_type in types {
            header.add_obs_type(&obs_type);
        }
    }
    
    // 设置观测时间范围
    if let Some(first_time) = ctx.get_first_observation_time() {
        header.first_obs_time = Some(first_time.to_datetime());
    }
    
    if let Some(last_time) = ctx.get_last_observation_time() {
        header.last_obs_time = Some(last_time.to_datetime());
    }
    
    // 写入头部
    header.write(&mut writer).map_err(Error::Rinex)?;
    
    // 写入观测数据
    let rinex_version = options.rinex_version;
    
    // 排序历元，确保按时间顺序写入
    let mut epochs: Vec<&i64> = ctx.observations.keys().collect();
    epochs.sort();
    
    let mut epoch_count = 0;
    
    for &time_key in &epochs {
        if let Some(epoch_data) = ctx.observations.get(time_key) {
            // 根据RINEX版本选择不同的写入方式
            if rinex_version >= 3.0 {
                // RINEX 3.x格式
                // 转换为系统分组的观测类型
                let mut obs_types_by_sys: HashMap<char, Vec<String>> = HashMap::new();
                for (sys, types) in &obs_types {
                    if options.systems.contains(sys) {
                        obs_types_by_sys.insert(*sys, types.clone());
                    }
                }
                epoch_data.write_v3(&mut writer, &obs_types_by_sys).map_err(Error::Rinex)?;
            } else {
                // RINEX 2.x格式
                // 获取所有观测类型的平铺列表
                let mut all_types = Vec::new();
                for (_sys, types) in &obs_types {
                    if options.systems.contains(_sys) {
                        for obs_type in types {
                            // 移除系统前缀
                            let type_without_sys = obs_type.chars().skip(1).collect::<String>();
                            if !all_types.contains(&type_without_sys) {
                                all_types.push(type_without_sys);
                            }
                        }
                    }
                }
                epoch_data.write_v2(&mut writer, &all_types).map_err(Error::Rinex)?;
            }
            
            epoch_count += 1;
        }
    }
    
    if options.debug {
        println!("Wrote {} observation epochs to {}", epoch_count, output_path);
    }
    
    Ok(())
}

/// 生成导航数据文件
fn generate_navigation_file(ctx: &RtcmContext, options: &ConversionOptions, output_path: &str, system: RinexSystem) -> Result<()> {
    use std::fs::File;
    
    if options.debug {
        println!("Generating navigation file: {}", output_path);
    }
    
    // 创建输出文件
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    
    // 准备RINEX头部
    let mut header = RinexHeader::new(
        options.rinex_version, 
        RinexFileType::Navigation, 
        system
    );
    
    // 设置程序信息
    header.program = "rtcm2rinex".to_string();
    header.run_by = "RUST".to_string();
    header.date_created = Utc::now();
    
    // 写入头部
    header.write(&mut writer).map_err(Error::Rinex)?;
    
    // 写入导航数据
    let rinex_version = options.rinex_version;
    let mut nav_count = 0;
    
    match system {
        RinexSystem::GPS => {
            // 写入GPS导航数据
            for nav_list in ctx.gps_nav_data.values() {
                for nav_data in nav_list {
                    NavData::GPS(nav_data.clone()).write(&mut writer, rinex_version).map_err(Error::Rinex)?;
                    nav_count += 1;
                }
            }
        },
        RinexSystem::GLONASS => {
            // 写入GLONASS导航数据
            for nav_list in ctx.glo_nav_data.values() {
                for nav_data in nav_list {
                    NavData::GLONASS(nav_data.clone()).write(&mut writer, rinex_version).map_err(Error::Rinex)?;
                    nav_count += 1;
                }
            }
        },
        _ => {
            // 其他系统暂不支持
            return Err(Error::Other(format!("Navigation data for system {:?} not supported", system)));
        }
    }
    
    if options.debug {
        println!("Wrote {} navigation messages to {}", nav_count, output_path);
    }
    
    Ok(())
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
    // 创建RTCM上下文
    let mut ctx = init_rtcm()?;
    
    // 处理RTCM文件
    process_rtcm_file(&mut ctx, rtcm_file)?;
    
    // 如果没有观测数据，返回错误
    if ctx.observations.is_empty() {
        return Err(Error::Other(format!("No observation data found in file: {}", rtcm_file)));
    }
    
    println!("Found {} observation epochs", ctx.get_observation_epochs_count());
    println!("Found {} GPS navigation messages", ctx.get_gps_nav_count());
    println!("Found {} GLONASS navigation messages", ctx.get_glonass_nav_count());
    
    // 创建转换选项
    let options = ConversionOptions {
        rinex_version,
        ..Default::default()
    };
    
    // 转换为RINEX
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
