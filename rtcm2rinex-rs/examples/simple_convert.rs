/*!
 * 简单的RTCM到RINEX转换示例
 */

use rtcm2rinex::simple_convert;
use std::env;
use std::process;

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("用法: {} <输入RTCM文件> <输出RINEX文件> [RINEX版本]", args[0]);
        process::exit(1);
    }
    
    let input_file = &args[1];
    let output_file = &args[2];
    
    // 默认RINEX版本为3.04
    let rinex_version = if args.len() > 3 {
        match args[3].parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("错误: RINEX版本格式无效");
                process::exit(1);
            }
        }
    } else {
        3.04
    };
    
    println!("正在将 {} 转换为 {} (RINEX v{})", input_file, output_file, rinex_version);
    
    match simple_convert(input_file, output_file, rinex_version) {
        Ok(_) => println!("转换成功完成"),
        Err(e) => {
            eprintln!("转换失败: {}", e);
            process::exit(1);
        }
    }
} 