/*!
 * 输入/输出工具模块
 * 
 * 该模块提供文件I/O相关的实用函数。
 */

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// 打开文件进行读取，支持自动检测压缩格式
pub fn open_file_read<P: AsRef<Path>>(path: P) -> io::Result<Box<dyn Read>> {
    let file = File::open(path)?;
    
    // 这里可以实现自动检测gz, zip等压缩格式并解压
    // 目前简单返回一个BufReader
    
    Ok(Box::new(BufReader::new(file)))
}

/// 打开文件进行写入，支持自动压缩
pub fn open_file_write<P: AsRef<Path>>(path: P) -> io::Result<Box<dyn Write>> {
    let file = File::create(path)?;
    
    // 这里可以根据文件扩展名自动选择压缩方式
    // 目前简单返回一个BufWriter
    
    Ok(Box::new(BufWriter::new(file)))
}

/// 内存映射文件
pub fn mmap_file<P: AsRef<Path>>(path: P) -> io::Result<memmap2::Mmap> {
    let file = File::open(path)?;
    unsafe { memmap2::MmapOptions::new().map(&file) }
}
