/*!
 * RTCM到RINEX转换命令行工具
 */

use clap::Parser;
use rtcm2rinex::simple_convert;
use std::path::PathBuf;
use std::process;

/// 命令行参数
#[derive(Parser, Debug)]
#[clap(author, version, about = "将RTCM格式转换为RINEX格式")]
struct Args {
    /// 输入RTCM文件
    #[clap(short, long, parse(from_os_str))]
    input: PathBuf,
    
    /// 输出RINEX文件
    #[clap(short, long, parse(from_os_str))]
    output: PathBuf,
    
    /// RINEX版本
    #[clap(short, long, default_value = "3.04")]
    version: f64,
}

fn main() {
    // 解析命令行参数
    let args = Args::parse();
    
    // 输入文件路径
    let input_path = args.input.to_string_lossy();
    
    // 输出文件路径
    let output_path = args.output.to_string_lossy();
    
    // 执行转换
    println!("正在将 {} 转换为 {} (RINEX v{})", input_path, output_path, args.version);
    
    match simple_convert(&input_path, &output_path, args.version) {
        Ok(_) => println!("转换成功"),
        Err(e) => {
            eprintln!("错误: {}", e);
            process::exit(1);
        }
    }
} 