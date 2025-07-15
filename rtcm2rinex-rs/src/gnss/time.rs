/*!
 * GNSS时间处理模块
 * 
 * 该模块实现了GNSS时间的表示和操作。
 */

use std::fmt;
use std::ops::{Add, Sub};
use chrono::{DateTime, Utc, TimeZone, Datelike, Timelike, NaiveDate};

/// GPS时间开始的历元（1980-01-06 00:00:00 UTC）
const GPS_EPOCH: i64 = 315964800; // 1980-01-06 00:00:00 UTC

/// GLONASS时间和GPS时间的偏移 (3小时，GLONASS是UTC+3)
const GLONASS_GPS_DIFF: f64 = 10800.0; // 3小时 = 10800秒

/// GNSS时间表示
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct GnssTime {
    /// 秒数（自GPS起始时间）
    seconds: f64,
}

impl Default for GnssTime {
    fn default() -> Self {
        Self { seconds: 0.0 }
    }
}

impl GnssTime {
    /// 创建新的GNSS时间
    pub fn new(seconds: f64) -> Self {
        Self { seconds }
    }
    
    /// 从年、月、日、时、分、秒创建GNSS时间
    pub fn from_ymd_hms(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: f64) -> Option<Self> {
        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let sec_i = sec as i64;
        
        let time_since_epoch = date.and_hms_opt(hour, min, 0)?
            .and_utc().timestamp() - GPS_EPOCH + sec_i;
        
        let seconds = time_since_epoch as f64 + (sec - sec_i as f64);
        
        Some(Self { seconds })
    }
    
    /// 获取当前UTC时间对应的GNSS时间
    pub fn now() -> Self {
        let now = Utc::now().timestamp() as f64;
        let gps_seconds = now - GPS_EPOCH as f64;
        Self { seconds: gps_seconds }
    }
    
    /// 获取GPS周
    pub fn week(&self) -> i32 {
        (self.seconds / (7.0 * 86400.0)) as i32
    }
    
    /// 获取GPS周内秒
    pub fn tow(&self) -> f64 {
        self.seconds % (7.0 * 86400.0)
    }
    
    /// 转换为UTC DateTime
    pub fn to_utc_datetime(&self) -> DateTime<Utc> {
        let timestamp = (self.seconds + GPS_EPOCH as f64) as i64;
        let nsecs = (((self.seconds + GPS_EPOCH as f64) - timestamp as f64) * 1_000_000_000.0) as u32;
        
        Utc.timestamp_opt(timestamp, nsecs).unwrap()
    }
    
    /// 获取秒数
    pub fn seconds(&self) -> f64 {
        self.seconds
    }

    /// 从GPS周和周内秒创建GNSS时间
    pub fn new_gps(week: i32, tow: f64) -> Self {
        let seconds = week as f64 * 7.0 * 86400.0 + tow;
        Self { seconds }
    }
    
    /// 从GLONASS日期创建GNSS时间
    /// day: 周内天 (1-7，1=星期一)
    /// tod: 天内秒
    pub fn new_glonass(day: i32, tod: f64) -> Self {
        // GLONASS时间是UTC+3，需要转换为GPS时间
        // 目前简化处理，忽略闰秒等复杂问题
        let gps_seconds = (day - 1) as f64 * 86400.0 + tod - GLONASS_GPS_DIFF;
        Self { seconds: gps_seconds }
    }
    
    /// 调整GPS周 (处理周跨越)
    pub fn adjust_week(&mut self, reference_time: &Self) {
        let week_diff = (reference_time.seconds - self.seconds).abs() / (7.0 * 86400.0);
        if week_diff > 1000.0 {  // 如果差距太大，可能是周跨越
            let week_adj = ((reference_time.seconds - self.seconds) / (7.0 * 86400.0)).round() as i32;
            self.seconds += week_adj as f64 * 7.0 * 86400.0;
        }
    }

    /// 转换为DateTime<Utc>，与to_utc_datetime相同，保持API一致性
    pub fn to_datetime(&self) -> DateTime<Utc> {
        self.to_utc_datetime()
    }
}

impl fmt::Display for GnssTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dt = self.to_utc_datetime();
        write!(f, "{}-{:02}-{:02} {:02}:{:02}:{:010.7}",
            dt.year(), dt.month(), dt.day(),
            dt.hour(), dt.minute(), dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0
        )
    }
}

impl Add<f64> for GnssTime {
    type Output = Self;
    
    fn add(self, seconds: f64) -> Self {
        Self {
            seconds: self.seconds + seconds,
        }
    }
}

impl Sub<f64> for GnssTime {
    type Output = Self;
    
    fn sub(self, seconds: f64) -> Self {
        Self {
            seconds: self.seconds - seconds,
        }
    }
}

impl Sub<GnssTime> for GnssTime {
    type Output = f64;
    
    fn sub(self, other: GnssTime) -> f64 {
        self.seconds - other.seconds
    }
}
