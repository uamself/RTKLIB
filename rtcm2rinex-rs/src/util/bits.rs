/*!
 * 位操作工具
 * 
 * 该模块提供位操作相关的工具函数和类型。
 */

use std::io::{self, Error, ErrorKind};

/// 位读取器
/// 用于从字节数组中按位读取数据
#[derive(Debug)]
pub struct BitReader<'a> {
    /// 输入数据
    data: &'a [u8],
    
    /// 当前位置（字节索引）
    byte_pos: usize,
    
    /// 当前位置（位索引，0-7）
    bit_pos: usize,
}

impl<'a> BitReader<'a> {
    /// 创建新的位读取器
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }
    
    /// 检查是否还有更多数据
    pub fn has_more(&self) -> bool {
        self.byte_pos < self.data.len()
    }
    
    /// 读取单个位
    pub fn read_bit(&mut self) -> Result<bool, io::Error> {
        if self.byte_pos >= self.data.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "End of data"));
        }
        
        let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 0x01;
        
        // 更新位置
        self.bit_pos += 1;
        if self.bit_pos >= 8 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
        
        Ok(bit != 0)
    }
    
    /// 读取指定位数的无符号整数
    pub fn read_bits(&mut self, bit_count: usize) -> Result<u32, io::Error> {
        if bit_count > 32 {
            return Err(Error::new(ErrorKind::InvalidInput, "Cannot read more than 32 bits into u32"));
        }
        
        let mut result = 0u32;
        
        for _ in 0..bit_count {
            result = (result << 1) | (if self.read_bit()? { 1 } else { 0 });
        }
        
        Ok(result)
    }
    
    /// 读取指定位数的有符号整数
    pub fn read_bits_signed(&mut self, bit_count: usize) -> Result<i32, io::Error> {
        if bit_count > 32 {
            return Err(Error::new(ErrorKind::InvalidInput, "Cannot read more than 32 bits into i32"));
        }
        
        if bit_count == 0 {
            return Ok(0);
        }
        
        let unsigned = self.read_bits(bit_count)?;
        
        // 检查符号位
        let sign_bit = 1u32 << (bit_count - 1);
        if (unsigned & sign_bit) != 0 {
            // 负数，扩展符号位
            let mask = !((1u32 << bit_count) - 1);
            Ok((unsigned | mask) as i32)
        } else {
            // 正数
            Ok(unsigned as i32)
        }
    }
    
    /// 读取指定位数的无符号整数 (RTKLIB 风格的 getbitu 实现)
    pub fn read_bits_u32(&mut self, bit_count: usize) -> Result<u32, io::Error> {
        if bit_count > 32 {
            return Err(Error::new(ErrorKind::InvalidInput, "Cannot read more than 32 bits into u32"));
        }
        
        let mut result = 0u32;
        let mut bits_read = 0;
        
        while bits_read < bit_count && self.byte_pos < self.data.len() {
            let available_bits = 8 - self.bit_pos;
            let bits_to_read = std::cmp::min(bit_count - bits_read, available_bits);
            
            // 创建掩码并从当前字节读取位
            let mask = ((1u16 << bits_to_read) - 1) as u8;
            let shifted = (self.data[self.byte_pos] >> (available_bits - bits_to_read)) & mask;
            
            // 添加到结果
            result = (result << bits_to_read) | (shifted as u32);
            
            // 更新位置
            self.bit_pos += bits_to_read;
            if self.bit_pos >= 8 {
                self.bit_pos = 0;
                self.byte_pos += 1;
            }
            
            bits_read += bits_to_read;
        }
        
        if bits_read < bit_count {
            Err(Error::new(ErrorKind::UnexpectedEof, "Not enough data available"))
        } else {
            Ok(result)
        }
    }
    
    /// 读取指定位数的有符号整数 (RTKLIB 风格的 getbits 实现)
    pub fn read_bits_i32(&mut self, bit_count: usize) -> Result<i32, io::Error> {
        if bit_count > 32 {
            return Err(Error::new(ErrorKind::InvalidInput, "Cannot read more than 32 bits into i32"));
        }
        
        let unsigned = self.read_bits_u32(bit_count)?;
        
        // 检查符号位
        if bit_count == 0 {
            return Ok(0);
        }
        
        let sign_bit = 1u32 << (bit_count - 1);
        if (unsigned & sign_bit) != 0 {
            // 负数，扩展符号位
            let mask = !((1u32 << bit_count) - 1);
            Ok((unsigned | mask) as i32)
        } else {
            // 正数
            Ok(unsigned as i32)
        }
    }
    
    /// 跳过指定位数
    pub fn skip_bits(&mut self, bit_count: usize) -> Result<(), io::Error> {
        let total_bits = self.data.len() * 8;
        let current_pos = self.byte_pos * 8 + self.bit_pos;
        let new_pos = current_pos + bit_count;
        
        if new_pos > total_bits {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Cannot skip beyond end of data"));
        }
        
        self.byte_pos = new_pos / 8;
        self.bit_pos = new_pos % 8;
        
        Ok(())
    }
    
    /// 将位置对齐到字节边界
    pub fn align_to_byte(&mut self) {
        if self.bit_pos > 0 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
    }
    
    /// 获取当前位置
    pub fn position(&self) -> (usize, usize) {
        (self.byte_pos, self.bit_pos)
    }
    
    /// 设置位置
    pub fn set_position(&mut self, byte_pos: usize, bit_pos: usize) -> Result<(), io::Error> {
        if byte_pos >= self.data.len() || (byte_pos == self.data.len() - 1 && bit_pos >= 8) {
            return Err(Error::new(ErrorKind::InvalidInput, "Position outside data bounds"));
        }
        
        if bit_pos >= 8 {
            return Err(Error::new(ErrorKind::InvalidInput, "Bit position must be 0-7"));
        }
        
        self.byte_pos = byte_pos;
        self.bit_pos = bit_pos;
        
        Ok(())
    }
}
