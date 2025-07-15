/*!
 * RINEX 导航数据模块
 *
 * 该模块处理RINEX导航数据的生成和解析。
 */

use std::io::{self, Write};
use chrono::{DateTime, Utc, NaiveDateTime};

use super::RinexError;

/// 导航系统类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavSystem {
    /// GPS
    GPS,
    /// GLONASS
    GLONASS,
    /// Galileo
    Galileo,
    /// 北斗
    BeiDou,
    /// QZSS
    QZSS,
    /// SBAS
    SBAS,
    /// IRNSS
    IRNSS,
}

/// GPS导航数据
#[derive(Debug, Clone)]
pub struct GpsNavData {
    /// 卫星PRN号
    pub prn: u8,
    /// 星历参考时间
    pub toc: DateTime<Utc>,
    /// 参考周数
    pub week: u16,
    /// 轨道精度
    pub sva: u8,
    /// L2码标志
    pub code_l2: u8,
    /// 轨道倾角变化率
    pub idot: f64,
    /// 星历数据块号
    pub iode: u8,
    /// 时钟二阶项
    pub af2: f64,
    /// 时钟一阶项
    pub af1: f64,
    /// 时钟零阶项
    pub af0: f64,
    /// 星历数据块号(检验值)
    pub iodc: u16,
    /// 升交点角距修正项
    pub crs: f64,
    /// 平均角速度修正项
    pub delta_n: f64,
    /// 平近点角
    pub m0: f64,
    /// 纬度幅角的余弦调和振幅
    pub cuc: f64,
    /// 轨道偏心率
    pub e: f64,
    /// 纬度幅角的正弦调和振幅
    pub cus: f64,
    /// 轨道长半轴的平方根
    pub sqrt_a: f64,
    /// 星历参考时间
    pub toe: f64,
    /// 轨道倾角的余弦调和振幅
    pub cic: f64,
    /// 升交点赤经
    pub omega0: f64,
    /// 轨道倾角的正弦调和振幅
    pub cis: f64,
    /// 轨道倾角
    pub i0: f64,
    /// 地心距离的余弦调和振幅
    pub crc: f64,
    /// 近地点角距
    pub omega: f64,
    /// 升交点赤经变化率
    pub omega_dot: f64,
    /// 群延迟
    pub tgd: f64,
    /// 卫星健康状态
    pub svh: u8,
    /// 信号精度标识
    pub flag: u8,
}

/// GLONASS导航数据
#[derive(Debug, Clone)]
pub struct GlonassNavData {
    /// 卫星PRN号
    pub slot: u8,
    /// 历元时间
    pub epoch: DateTime<Utc>,
    /// 频率号
    pub freqno: i8,
    /// 数据龄期
    pub age: u8,
    /// 健康状态
    pub svh: u8,
    /// 位置X
    pub pos_x: f64,
    /// 位置Y
    pub pos_y: f64,
    /// 位置Z
    pub pos_z: f64,
    /// 速度X
    pub vel_x: f64,
    /// 速度Y
    pub vel_y: f64,
    /// 速度Z
    pub vel_z: f64,
    /// 加速度X
    pub acc_x: f64,
    /// 加速度Y
    pub acc_y: f64,
    /// 加速度Z
    pub acc_z: f64,
    /// 相对频率偏差
    pub gamma: f64,
    /// 接收机时钟偏差
    pub tau_n: f64,
    /// 消息帧时间
    pub tk: f64,
}

/// Galileo导航数据
#[derive(Debug, Clone)]
pub struct GalileoNavData {
    /// 卫星PRN号
    pub prn: u8,
    /// 星历参考时间
    pub toc: DateTime<Utc>,
    /// 参考周数
    pub week: u16,
    /// 信号精度指示符
    pub sisa: u8,
    /// 数据源
    pub data_source: u8, // 0: I/NAV E1-B, 1: F/NAV E5a-I, 2: I/NAV E5b-I
    /// 时钟二阶项
    pub af2: f64,
    /// 时钟一阶项
    pub af1: f64,
    /// 时钟零阶项
    pub af0: f64,
    /// 星历数据块号
    pub iode: u16,
    /// 升交点角距修正项
    pub crs: f64,
    /// 平均角速度修正项
    pub delta_n: f64,
    /// 平近点角
    pub m0: f64,
    /// 纬度幅角的余弦调和振幅
    pub cuc: f64,
    /// 轨道偏心率
    pub e: f64,
    /// 纬度幅角的正弦调和振幅
    pub cus: f64,
    /// 轨道长半轴的平方根
    pub sqrt_a: f64,
    /// 星历参考时间
    pub toe: f64,
    /// 轨道倾角的余弦调和振幅
    pub cic: f64,
    /// 升交点赤经
    pub omega0: f64,
    /// 轨道倾角的正弦调和振幅
    pub cis: f64,
    /// 轨道倾角
    pub i0: f64,
    /// 地心距离的余弦调和振幅
    pub crc: f64,
    /// 近地点角距
    pub omega: f64,
    /// 升交点赤经变化率
    pub omega_dot: f64,
    /// 轨道倾角变化率
    pub idot: f64,
    /// BGD E5a/E1
    pub bgd_e5a_e1: f64,
    /// BGD E5b/E1
    pub bgd_e5b_e1: f64,
    /// 卫星健康状态
    pub svh: u8,
}

/// BeiDou导航数据
#[derive(Debug, Clone)]
pub struct BeidouNavData {
    /// 卫星PRN号
    pub prn: u8,
    /// 星历参考时间
    pub toc: DateTime<Utc>,
    /// 参考周数
    pub week: u16,
    /// 轨道精度
    pub sva: u8,
    /// 时钟二阶项
    pub af2: f64,
    /// 时钟一阶项
    pub af1: f64,
    /// 时钟零阶项
    pub af0: f64,
    /// 星历数据块号
    pub iode: u8,
    /// 升交点角距修正项
    pub crs: f64,
    /// 平均角速度修正项
    pub delta_n: f64,
    /// 平近点角
    pub m0: f64,
    /// 纬度幅角的余弦调和振幅
    pub cuc: f64,
    /// 轨道偏心率
    pub e: f64,
    /// 纬度幅角的正弦调和振幅
    pub cus: f64,
    /// 轨道长半轴的平方根
    pub sqrt_a: f64,
    /// 星历参考时间
    pub toe: f64,
    /// 轨道倾角的余弦调和振幅
    pub cic: f64,
    /// 升交点赤经
    pub omega0: f64,
    /// 轨道倾角的正弦调和振幅
    pub cis: f64,
    /// 轨道倾角
    pub i0: f64,
    /// 地心距离的余弦调和振幅
    pub crc: f64,
    /// 近地点角距
    pub omega: f64,
    /// 升交点赤经变化率
    pub omega_dot: f64,
    /// 轨道倾角变化率
    pub idot: f64,
    /// TGD1 B1/B3
    pub tgd1: f64,
    /// TGD2 B2/B3
    pub tgd2: f64,
    /// 卫星健康状态
    pub svh: u8,
}

/// QZSS导航数据
#[derive(Debug, Clone)]
pub struct QzssNavData {
    /// 卫星PRN号
    pub prn: u8,
    /// 星历参考时间
    pub toc: DateTime<Utc>,
    /// 参考周数
    pub week: u16,
    /// 轨道精度
    pub sva: u8,
    /// L2码标志
    pub code_l2: u8,
    /// 轨道倾角变化率
    pub idot: f64,
    /// 星历数据块号
    pub iode: u8,
    /// 时钟二阶项
    pub af2: f64,
    /// 时钟一阶项
    pub af1: f64,
    /// 时钟零阶项
    pub af0: f64,
    /// 星历数据块号(检验值)
    pub iodc: u16,
    /// 升交点角距修正项
    pub crs: f64,
    /// 平均角速度修正项
    pub delta_n: f64,
    /// 平近点角
    pub m0: f64,
    /// 纬度幅角的余弦调和振幅
    pub cuc: f64,
    /// 轨道偏心率
    pub e: f64,
    /// 纬度幅角的正弦调和振幅
    pub cus: f64,
    /// 轨道长半轴的平方根
    pub sqrt_a: f64,
    /// 星历参考时间
    pub toe: f64,
    /// 轨道倾角的余弦调和振幅
    pub cic: f64,
    /// 升交点赤经
    pub omega0: f64,
    /// 轨道倾角的正弦调和振幅
    pub cis: f64,
    /// 轨道倾角
    pub i0: f64,
    /// 地心距离的余弦调和振幅
    pub crc: f64,
    /// 近地点角距
    pub omega: f64,
    /// 升交点赤经变化率
    pub omega_dot: f64,
    /// 群延迟
    pub tgd: f64,
    /// 卫星健康状态
    pub svh: u8,
    /// 信号精度标识
    pub flag: u8,
}

/// 通用导航数据
#[derive(Debug, Clone)]
pub enum NavData {
    /// GPS导航数据
    GPS(GpsNavData),
    /// GLONASS导航数据
    GLONASS(GlonassNavData),
    /// Galileo导航数据
    Galileo(GalileoNavData),
    /// BeiDou导航数据
    BeiDou(BeidouNavData),
    /// QZSS导航数据
    QZSS(QzssNavData),
}

impl NavData {
    /// 写入导航数据
    pub fn write<W: Write>(&self, writer: &mut W, version: f64) -> Result<(), RinexError> {
        match self {
            NavData::GPS(data) => write_gps_nav(writer, data, version),
            NavData::GLONASS(data) => write_glonass_nav(writer, data, version),
            NavData::Galileo(data) => write_galileo_nav(writer, data, version),
            NavData::BeiDou(data) => write_beidou_nav(writer, data, version),
            NavData::QZSS(data) => write_qzss_nav(writer, data, version),
        }
    }
    
    /// 获取卫星PRN
    pub fn get_prn(&self) -> u8 {
        match self {
            NavData::GPS(data) => data.prn,
            NavData::GLONASS(data) => data.slot,
            NavData::Galileo(data) => data.prn,
            NavData::BeiDou(data) => data.prn,
            NavData::QZSS(data) => data.prn,
        }
    }
    
    /// 获取导航数据时间
    pub fn get_time(&self) -> DateTime<Utc> {
        match self {
            NavData::GPS(data) => data.toc,
            NavData::GLONASS(data) => data.epoch,
            NavData::Galileo(data) => data.toc,
            NavData::BeiDou(data) => data.toc,
            NavData::QZSS(data) => data.toc,
        }
    }
    
    /// 获取导航系统类型
    pub fn get_system(&self) -> NavSystem {
        match self {
            NavData::GPS(_) => NavSystem::GPS,
            NavData::GLONASS(_) => NavSystem::GLONASS,
            NavData::Galileo(_) => NavSystem::Galileo,
            NavData::BeiDou(_) => NavSystem::BeiDou,
            NavData::QZSS(_) => NavSystem::QZSS,
        }
    }
}

/// 写入GPS导航数据
fn write_gps_nav<W: Write>(writer: &mut W, data: &GpsNavData, version: f64) -> Result<(), RinexError> {
    // 星历行1
    if version >= 3.0 {
        write!(writer, "G{:02} {:04} {:02} {:02} {:02} {:02} {:02}",
               data.prn,
               data.toc.year(),
               data.toc.month(),
               data.toc.day(),
               data.toc.hour(),
               data.toc.minute(),
               data.toc.second())?;
    } else {
        write!(writer, "{:2}{:>3}{:>3}{:>3}{:>3}{:>3}{:>3}",
               data.prn,
               data.toc.year() % 100,
               data.toc.month(),
               data.toc.day(),
               data.toc.hour(),
               data.toc.minute(),
               data.toc.second())?;
    }
    
    writeln!(writer, "{:19.12E}{:19.12E}{:19.12E}",
            data.af0, data.af1, data.af2)?;
    
    // 星历行2
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.iode as f64, data.crs, data.delta_n, data.m0)?;
    
    // 星历行3
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.cuc, data.e, data.cus, data.sqrt_a)?;
    
    // 星历行4
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.toe, data.cic, data.omega0, data.cis)?;
    
    // 星历行5
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.i0, data.crc, data.omega, data.omega_dot)?;
    
    // 星历行6
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.idot, data.code_l2 as f64, data.week as f64, data.flag as f64)?;
    
    // 星历行7
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.sva as f64, data.svh as f64, data.tgd, data.iodc as f64)?;
    
    // 星历行8
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            0.0, 0.0, 0.0, 0.0)?;
    
    Ok(())
}

/// 写入GLONASS导航数据
fn write_glonass_nav<W: Write>(writer: &mut W, data: &GlonassNavData, version: f64) -> Result<(), RinexError> {
    // 星历行1
    if version >= 3.0 {
        write!(writer, "R{:02} {:04} {:02} {:02} {:02} {:02} {:02}",
               data.slot,
               data.epoch.year(),
               data.epoch.month(),
               data.epoch.day(),
               data.epoch.hour(),
               data.epoch.minute(),
               data.epoch.second())?;
    } else {
        write!(writer, "{:2}{:>3}{:>3}{:>3}{:>3}{:>3}{:>3}",
               data.slot,
               data.epoch.year() % 100,
               data.epoch.month(),
               data.epoch.day(),
               data.epoch.hour(),
               data.epoch.minute(),
               data.epoch.second())?;
    }
    
    writeln!(writer, "{:19.12E}{:19.12E}{:19.12E}",
            data.tau_n, data.gamma, data.tk)?;
    
    // 星历行2
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.pos_x, data.vel_x, data.acc_x, data.svh as f64)?;
    
    // 星历行3
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.pos_y, data.vel_y, data.acc_y, data.freqno as f64)?;
    
    // 星历行4
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.pos_z, data.vel_z, data.acc_z, data.age as f64)?;
    
    Ok(())
}

/// 写入Galileo导航数据
fn write_galileo_nav<W: Write>(writer: &mut W, data: &GalileoNavData, version: f64) -> Result<(), RinexError> {
    // 星历行1
    if version >= 3.0 {
        write!(writer, "E{:02} {:04} {:02} {:02} {:02} {:02} {:02}",
               data.prn,
               data.toc.year(),
               data.toc.month(),
               data.toc.day(),
               data.toc.hour(),
               data.toc.minute(),
               data.toc.second())?;
    } else {
        // RINEX 2.x不支持Galileo，但为了兼容性仍然提供
        write!(writer, "{:2}{:>3}{:>3}{:>3}{:>3}{:>3}{:>3}",
               data.prn,
               data.toc.year() % 100,
               data.toc.month(),
               data.toc.day(),
               data.toc.hour(),
               data.toc.minute(),
               data.toc.second())?;
    }
    
    writeln!(writer, "{:19.12E}{:19.12E}{:19.12E}",
            data.af0, data.af1, data.af2)?;
    
    // 星历行2
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.iode as f64, data.crs, data.delta_n, data.m0)?;
    
    // 星历行3
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.cuc, data.e, data.cus, data.sqrt_a)?;
    
    // 星历行4
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.toe, data.cic, data.omega0, data.cis)?;
    
    // 星历行5
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.i0, data.crc, data.omega, data.omega_dot)?;
    
    // 星历行6
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.idot, data.data_source as f64, data.week as f64, data.sisa as f64)?;
    
    // 星历行7
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.svh as f64, data.bgd_e5a_e1, data.bgd_e5b_e1, 0.0)?;
    
    // 星历行8
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            0.0, 0.0, 0.0, 0.0)?;
    
    Ok(())
}

/// 写入BeiDou导航数据
fn write_beidou_nav<W: Write>(writer: &mut W, data: &BeidouNavData, version: f64) -> Result<(), RinexError> {
    // 星历行1
    if version >= 3.0 {
        write!(writer, "C{:02} {:04} {:02} {:02} {:02} {:02} {:02}",
               data.prn,
               data.toc.year(),
               data.toc.month(),
               data.toc.day(),
               data.toc.hour(),
               data.toc.minute(),
               data.toc.second())?;
    } else {
        // RINEX 2.x不支持BeiDou，但为了兼容性仍然提供
        write!(writer, "{:2}{:>3}{:>3}{:>3}{:>3}{:>3}{:>3}",
               data.prn,
               data.toc.year() % 100,
               data.toc.month(),
               data.toc.day(),
               data.toc.hour(),
               data.toc.minute(),
               data.toc.second())?;
    }
    
    writeln!(writer, "{:19.12E}{:19.12E}{:19.12E}",
            data.af0, data.af1, data.af2)?;
    
    // 星历行2
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.iode as f64, data.crs, data.delta_n, data.m0)?;
    
    // 星历行3
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.cuc, data.e, data.cus, data.sqrt_a)?;
    
    // 星历行4
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.toe, data.cic, data.omega0, data.cis)?;
    
    // 星历行5
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.i0, data.crc, data.omega, data.omega_dot)?;
    
    // 星历行6
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.idot, 0.0, data.week as f64, 0.0)?;
    
    // 星历行7
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.sva as f64, data.svh as f64, data.tgd1, data.tgd2)?;
    
    // 星历行8
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            0.0, 0.0, 0.0, 0.0)?;
    
    Ok(())
}

/// 写入QZSS导航数据
fn write_qzss_nav<W: Write>(writer: &mut W, data: &QzssNavData, version: f64) -> Result<(), RinexError> {
    // 星历行1
    if version >= 3.0 {
        write!(writer, "J{:02} {:04} {:02} {:02} {:02} {:02} {:02}",
               data.prn,
               data.toc.year(),
               data.toc.month(),
               data.toc.day(),
               data.toc.hour(),
               data.toc.minute(),
               data.toc.second())?;
    } else {
        // RINEX 2.x不支持QZSS，但为了兼容性仍然提供
        write!(writer, "{:2}{:>3}{:>3}{:>3}{:>3}{:>3}{:>3}",
               data.prn,
               data.toc.year() % 100,
               data.toc.month(),
               data.toc.day(),
               data.toc.hour(),
               data.toc.minute(),
               data.toc.second())?;
    }
    
    writeln!(writer, "{:19.12E}{:19.12E}{:19.12E}",
            data.af0, data.af1, data.af2)?;
    
    // 星历行2
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.iode as f64, data.crs, data.delta_n, data.m0)?;
    
    // 星历行3
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.cuc, data.e, data.cus, data.sqrt_a)?;
    
    // 星历行4
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.toe, data.cic, data.omega0, data.cis)?;
    
    // 星历行5
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.i0, data.crc, data.omega, data.omega_dot)?;
    
    // 星历行6
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.idot, data.code_l2 as f64, data.week as f64, data.flag as f64)?;
    
    // 星历行7
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            data.sva as f64, data.svh as f64, data.tgd, data.iodc as f64)?;
    
    // 星历行8
    writeln!(writer, "    {:19.12E}{:19.12E}{:19.12E}{:19.12E}",
            0.0, 0.0, 0.0, 0.0)?;
    
    Ok(())
}