/*!
 * C语言API实现
 * 
 * 提供C兼容的API函数。
 */

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};
use std::ptr;

use crate::{init_rtcm, simple_convert, RtcmContext};

/// 内部上下文句柄类型
type ContextHandle = *mut RtcmContext;

/// 创建新的RTCM上下文
/// 
/// # Returns
/// 
/// 非空指针表示成功，NULL表示失败
/// 
/// # Safety
/// 
/// 这是一个外部C API函数，调用者负责正确使用返回的句柄
#[no_mangle]
pub unsafe extern "C" fn rtcm2rinex_init() -> ContextHandle {
    match init_rtcm() {
        Ok(ctx) => Box::into_raw(Box::new(ctx)),
        Err(_) => ptr::null_mut(),
    }
}

/// 释放RTCM上下文
/// 
/// # Arguments
/// 
/// * `handle` - 由rtcm2rinex_init返回的上下文句柄
/// 
/// # Safety
/// 
/// 这是一个外部C API函数，调用者负责提供有效的句柄
#[no_mangle]
pub unsafe extern "C" fn rtcm2rinex_free(handle: ContextHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// 简单转换RTCM文件到RINEX文件
/// 
/// # Arguments
/// 
/// * `rtcm_file` - RTCM文件路径（以NULL结尾的C字符串）
/// * `rinex_file` - RINEX文件路径（以NULL结尾的C字符串）
/// * `rinex_version` - RINEX版本号（如3.04）
/// 
/// # Returns
/// 
/// 0表示成功，非0表示失败
/// 
/// # Safety
/// 
/// 这是一个外部C API函数，调用者负责提供有效的文件路径字符串
#[no_mangle]
pub unsafe extern "C" fn rtcm2rinex_convert(
    rtcm_file: *const c_char,
    rinex_file: *const c_char,
    rinex_version: c_double,
) -> c_int {
    // 检查空指针
    if rtcm_file.is_null() || rinex_file.is_null() {
        return -1;
    }
    
    // 转换C字符串到Rust字符串
    let rtcm_path = match CStr::from_ptr(rtcm_file).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let rinex_path = match CStr::from_ptr(rinex_file).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    // 执行转换
    match simple_convert(rtcm_path, rinex_path, rinex_version) {
        Ok(_) => 0,
        Err(_) => -3,
    }
} 