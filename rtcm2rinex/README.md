# RTCM到RINEX转换工具

这个目录包含了RTKLIB中与RTCM到RINEX转换相关的所有源代码。主要用于将RTCM2/RTCM3格式的数据转换为RINEX格式的观测数据和导航数据。

## 功能特点

- 支持RTCM2和RTCM3格式
- 支持转换为RINEX 2.x和3.x格式
- 处理多种GNSS系统：GPS, GLONASS, Galileo, BeiDou, QZSS, SBAS, IRNSS
- 支持多种RTCM消息类型：
  - RTCM2: 类型1, 3, 9, 14, 16, 17, 18, 19, 22
  - RTCM3: 类型1002, 1004, 1005, 1006, 1010, 1012, 1019, 1020, 1071-1127(MSM)等

## 编译方法

```bash
cd rtcm2rinex
make
```

## 使用方法

### 命令行工具

```bash
./rtcm2rinex [options] rtcm_file
```

#### 常用选项

- `-ts y/m/d h:m:s` 开始时间 (GPST)
- `-te y/m/d h:m:s` 结束时间 (GPST)
- `-ti tint` 时间间隔 (秒)
- `-v ver` RINEX版本 (2.10,2.11,2.12,3.00,3.01,3.02,3.03,3.04)
- `-d dir` 输出目录
- `-o ofile` 输出RINEX观测数据文件
- `-n nfile` 输出RINEX导航数据文件
- `-g gfile` 输出RINEX GLONASS导航数据文件
- `-trace level` 输出跟踪级别

更多选项请参考帮助信息：

```bash
./rtcm2rinex -h
```

### 转换示例

从RTCM3文件转换为RINEX 3.04观测数据文件和导航数据文件：

```bash
./rtcm2rinex -v 3.04 -o obs.rnx -n nav.rnx input.rtcm3
```

## 程序接口集成

如果您需要将RTCM到RINEX转换功能集成到自己的程序中，可以参考以下关键函数：

1. 初始化RTCM解码器：
```c
rtcm_t rtcm={0};
init_rtcm(&rtcm);
```

2. 处理RTCM数据：
```c
int ret = input_rtcm3(&rtcm, data);
```

3. 转换为RINEX：
```c
convrnx(STRFMT_RTCM3, &opt, input_file, output_files);
```

主要的转换流程是：
1. 读取RTCM文件
2. 解析RTCM消息
3. 转换到内部观测数据和导航数据结构
4. 输出为RINEX格式

## 文件说明

- `convbin.c`: 主程序，命令行接口
- `rtcm.c`: RTCM通用函数
- `rtcm2.c`: RTCM2格式处理
- `rtcm3.c`: RTCM3格式处理
- `rtcm3e.c`: RTCM3扩展格式处理
- `convrnx.c`: RINEX转换核心函数
- `rinex.c`: RINEX格式处理
- `rtklib.h`: 头文件（所有数据结构和函数声明）

## 参考资料

- RTCM标准文档 10403.x (可从RTCM官网获取)
- RINEX格式说明文档
- RTKLIB文档
