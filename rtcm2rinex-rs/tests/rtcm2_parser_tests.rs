use rtcm2rinex::rtcm::{Rtcm2Parser, Rtcm2Message, Rtcm2MessageType};

#[test]
fn test_rtcm2_parser_initialization() {
    // 只测试可以创建解析器
    let _parser = Rtcm2Parser::new();
}

#[test]
fn test_rtcm2_preamble_detection() {
    let mut parser = Rtcm2Parser::new();
    
    // 0x66是RTCM2前导码
    let result = parser.process_byte(0x66).unwrap();
    assert!(result.is_none());
    
    // 非前导码字节应该被忽略（对于处于FindPreamble状态的解析器）
    let result = parser.process_byte(0x00).unwrap();
    assert!(result.is_none());
}

// 这个测试需要真实的RTCM2数据才能完全验证
// 这里只是提供一个结构性测试
#[test]
fn test_rtcm2_type1_structure() {
    // 模拟一条RTCM2 Type1消息
    // 头部: 前导码(1字节) + 消息类型1(6位) + 站点ID 1(10位) + ...
    // 数据: 比例因子(1位) + 卫星数据...
    
    // 这只是一个基本结构测试，不包含完整的RTCM2消息
    // 真实测试需要实际的RTCM2数据流
    let mut parser = Rtcm2Parser::new();
    
    // 前导码
    parser.process_byte(0x66).unwrap();
    
    // 消息类型1和站点ID (模拟)
    parser.process_byte(0x44).unwrap(); // 第一个字节包含消息类型(1)和站点ID的高位
    parser.process_byte(0x00).unwrap(); // 站点ID的低位
    
    // 模拟更多字节 (不是真实的RTCM2数据)
    // 在实际测试中，应该提供完整有效的RTCM2消息
}

// 注意: 完整测试需要使用实际的RTCM2数据文件
// 这些测试只是验证基本结构和初始化 