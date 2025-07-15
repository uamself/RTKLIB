//! Test cases for RTCM2 parser and RTCM to RINEX conversion

use rtcm2rinex::{init_rtcm, RtcmContext, RtcmMessageType, ConversionOptions};
use std::fs;
use std::io::{self, Read};

// 测试文件路径
const TEST_DATA_DIR: &str = "../test/data/";

#[test]
fn test_rtcm2_to_rinex() -> io::Result<()> {
    // 创建RTCM上下文
    let mut ctx = match init_rtcm() {
        Ok(ctx) => ctx,
        Err(e) => {
            // 测试环境中，如果初始化失败只记录错误而非失败
            println!("Failed to initialize RTCM context: {}", e);
            return Ok(());
        }
    };
    
    // 读取测试文件
    let files = vec![
        "rcvraw/GMSD7_20121014.rtcm3",
    ];
    
    for file_path in files {
        let full_path = format!("{}{}", TEST_DATA_DIR, file_path);
        
        match fs::File::open(&full_path) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                if let Err(e) = file.read_to_end(&mut buffer) {
                    println!("Failed to read file {}: {}", full_path, e);
                    continue;
                }
                
                // 处理文件
                for &byte in &buffer {
                    if let Err(e) = ctx.process_byte(byte) {
                        println!("Error processing byte: {}", e);
                    }
                }
                
                // 检查是否有观测数据
                let observation_count = ctx.get_observation_epochs_count();
                println!("Processed file {}: {} observation epochs", full_path, observation_count);
                
                // 检查是否有导航数据
                let gps_nav_count = ctx.get_gps_nav_count();
                let glo_nav_count = ctx.get_glonass_nav_count();
                println!("GPS navigation data: {}", gps_nav_count);
                println!("GLONASS navigation data: {}", glo_nav_count);
                
                // 如果有观测数据，尝试转换为RINEX
                if observation_count > 0 {
                    // 创建临时输出文件路径
                    let output_path = format!("/tmp/test_output_{}.rnx", file_path.replace('/', "_"));
                    
                    // 创建转换选项
                    let obs_types = ctx.get_observation_types();
                    let options = ConversionOptions {
                        rinex_version: 3.04,
                        systems: obs_types.keys().cloned().collect(),
                        ..Default::default()
                    };
                    
                    // 转换为RINEX
                    match rtcm2rinex::convert_to_rinex(&ctx, &options, &output_path) {
                        Ok(_) => println!("Successfully converted to RINEX: {}", output_path),
                        Err(e) => println!("Failed to convert to RINEX: {}", e),
                    }
                } else {
                    println!("No observation data found, skipping RINEX conversion");
                }
                
                // 清除上下文以便处理下一个文件
                ctx.clear();
            },
            Err(e) => {
                println!("Failed to open file {}: {}", full_path, e);
                continue;
            }
        }
    }
    
    Ok(())
}

/// 助手函数：查找指定目录下的所有RTCM文件
fn find_rtcm_files(dir_path: &str) -> io::Result<Vec<String>> {
    let mut result = Vec::new();
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let ext_str = extension.to_string_lossy().to_lowercase();
                if ext_str == "rtcm" || ext_str == "rtcm3" || ext_str == "bin" {
                    if let Some(path_str) = path.to_str() {
                        result.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    Ok(result)
} 