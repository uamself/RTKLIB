use std::time::Instant;
use clap::{App, Arg};
use rtcm2rinex::{process_rtcm_file, init_rtcm, convert_to_rinex, ConversionOptions};

fn main() {
    let matches = App::new("rtcm2rinex")
        .version(env!("CARGO_PKG_VERSION"))
        .author("RTCM2RINEX Rust Team")
        .about("Convert RTCM messages to RINEX format")
        .arg(Arg::with_name("INPUT")
            .help("Input RTCM file")
            .required(true)
            .index(1))
        .arg(Arg::with_name("OUTPUT")
            .help("Output RINEX file (without extension)")
            .required(true)
            .index(2))
        .arg(Arg::with_name("version")
            .short('v')
            .long("version")
            .value_name("VERSION")
            .help("RINEX version (2.11, 3.04, etc.)")
            .takes_value(true)
            .default_value("3.04"))
        .arg(Arg::with_name("systems")
            .short('s')
            .long("systems")
            .value_name("SYSTEMS")
            .help("Satellite systems to include (G=GPS, R=GLONASS, E=Galileo, C=BeiDou, J=QZSS, S=SBAS, I=IRNSS)")
            .takes_value(true)
            .default_value("GRECJSI"))
        .arg(Arg::with_name("output-dir")
            .short('d')
            .long("output-dir")
            .value_name("DIR")
            .help("Output directory")
            .takes_value(true))
        .arg(Arg::with_name("prefix")
            .short('p')
            .long("prefix")
            .value_name("PREFIX")
            .help("Output file prefix")
            .takes_value(true))
        .arg(Arg::with_name("obs-only")
            .long("obs-only")
            .help("Output only observation data"))
        .arg(Arg::with_name("nav-only")
            .long("nav-only")
            .help("Output only navigation data"))
        .arg(Arg::with_name("compress")
            .short('c')
            .long("compress")
            .help("Compress output files"))
        .arg(Arg::with_name("debug")
            .long("debug")
            .help("Enable debug output"))
        .get_matches();

    let input_file = matches.value_of("INPUT").unwrap();
    let output_file = matches.value_of("OUTPUT").unwrap();
    let rinex_version = matches.value_of("version").unwrap().parse::<f64>().unwrap_or(3.04);
    
    // 解析卫星系统
    let systems_str = matches.value_of("systems").unwrap();
    let systems: Vec<char> = systems_str.chars().collect();
    
    // 解析其他选项
    let output_dir = matches.value_of("output-dir").map(|s| s.to_string());
    let prefix = matches.value_of("prefix").map(|s| s.to_string());
    let obs_only = matches.is_present("obs-only");
    let nav_only = matches.is_present("nav-only");
    let compress = matches.is_present("compress");
    let debug = matches.is_present("debug");
    
    // 创建转换选项
    let options = ConversionOptions {
        rinex_version,
        output_dir,
        output_prefix: prefix,
        systems,
        output_obs: !nav_only,
        output_nav: !obs_only,
        debug,
        compress,
    };
    
    // 记录开始时间
    let start_time = Instant::now();
    
    if debug {
        println!("Converting {} to RINEX format...", input_file);
    }
    
    // 创建RTCM上下文
    let mut ctx = match init_rtcm() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Failed to initialize RTCM context: {}", e);
            std::process::exit(1);
        }
    };
    
    // 处理RTCM文件
    if let Err(e) = process_rtcm_file(&mut ctx, input_file) {
        eprintln!("Failed to process RTCM file: {}", e);
        std::process::exit(1);
    }
    
    // 检查是否有观测数据
    let obs_count = ctx.get_observation_epochs_count();
    let gps_nav_count = ctx.get_gps_nav_count();
    let glo_nav_count = ctx.get_glonass_nav_count();
    
    if obs_count == 0 && gps_nav_count == 0 && glo_nav_count == 0 {
        eprintln!("No valid RTCM data found in the input file.");
        std::process::exit(1);
    }
    
    if debug {
        println!("Found {} observation epochs", obs_count);
        println!("Found {} GPS navigation messages", gps_nav_count);
        println!("Found {} GLONASS navigation messages", glo_nav_count);
    }
    
    // 转换为RINEX
    if let Err(e) = convert_to_rinex(&ctx, &options, output_file) {
        eprintln!("Failed to convert to RINEX: {}", e);
        std::process::exit(1);
    }
    
    // 计算处理时间
    let elapsed = start_time.elapsed();
    
    if debug {
        println!("Conversion completed in {:.2} seconds", elapsed.as_secs_f64());
    }
} 