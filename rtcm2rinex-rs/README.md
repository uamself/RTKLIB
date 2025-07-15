# RTCM2RINEX-RS

RTCM到RINEX格式转换工具的Rust实现。

## 功能

- 支持RTCM2.x和RTCM3.x格式解析
- 支持生成RINEX 2.x和3.x格式文件
- 支持多种卫星系统：GPS、GLONASS、Galileo、BeiDou、QZSS、SBAS、IRNSS
- 支持MSM4-MSM7观测数据处理
- 支持GPS和GLONASS星历数据处理
- 提供命令行工具和库API

## 使用方法

### 命令行工具

```bash
rtcm2rinex [OPTIONS] <INPUT> <OUTPUT>
```

参数:
- `<INPUT>`: 输入RTCM文件路径
- `<OUTPUT>`: 输出RINEX文件路径（不含扩展名）

选项:
- `-v, --version <VERSION>`: RINEX版本 (默认: 3.04)
- `-s, --systems <SYSTEMS>`: 卫星系统选择 (G=GPS, R=GLONASS, E=Galileo, C=BeiDou, J=QZSS, S=SBAS, I=IRNSS) (默认: GRECJSI)
- `-d, --output-dir <DIR>`: 输出目录
- `-p, --prefix <PREFIX>`: 输出文件前缀
- `--obs-only`: 仅输出观测数据
- `--nav-only`: 仅输出导航数据
- `-c, --compress`: 压缩输出文件
- `--debug`: 启用调试输出
- `-h, --help`: 显示帮助信息

### 库API

```rust
use rtcm2rinex::{init_rtcm, process_rtcm_file, convert_to_rinex, ConversionOptions};

// 创建RTCM上下文
let mut ctx = init_rtcm().unwrap();

// 处理RTCM文件
process_rtcm_file(&mut ctx, "input.rtcm").unwrap();

// 创建转换选项
let options = ConversionOptions {
    rinex_version: 3.04,
    systems: vec!['G', 'R', 'E', 'C', 'J', 'S', 'I'],
    output_obs: true,
    output_nav: true,
    ..Default::default()
};

// 转换为RINEX
convert_to_rinex(&ctx, &options, "output").unwrap();
```

## 安装

### 从源码安装

```bash
git clone https://github.com/yourusername/rtcm2rinex-rs.git
cd rtcm2rinex-rs
cargo build --release
```

安装到系统:

```bash
cargo install --path .
```

## 支持的RTCM消息类型

- RTCM 3.x MSM4-MSM7观测数据
- RTCM 3.x GPS星历数据
- RTCM 3.x GLONASS星历数据
- RTCM 3.x 参考站坐标
- RTCM 3.x 天线描述
- RTCM 2.x 观测数据
- RTCM 2.x 导航数据

## 支持的RINEX格式

- RINEX 2.11
- RINEX 3.00
- RINEX 3.01
- RINEX 3.02
- RINEX 3.03
- RINEX 3.04

## 构建选项

可以通过特性(features)启用额外功能:

```bash
# 启用压缩功能
cargo build --release --features compression
```

## 示例

```bash
# 转换为RINEX 3.04格式
rtcm2rinex input.rtcm output

# 转换为RINEX 2.11格式，仅包含GPS和GLONASS系统
rtcm2rinex -v 2.11 -s GR input.rtcm output

# 仅输出观测数据
rtcm2rinex --obs-only input.rtcm output

# 压缩输出文件
rtcm2rinex -c input.rtcm output
```

## 许可证

MIT

## 参考

- [RTCM标准](https://rtcm.org/)
- [RINEX标准](https://igs.org/formats-and-standards/)
- [RTKLIB](https://github.com/tomojitakasu/RTKLIB)