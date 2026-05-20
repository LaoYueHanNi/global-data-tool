# Global Data Tool - 全球城市时间搜索与汇率换算

轻量级 Windows 桌面应用，支持全球城市时间查询与实时汇率换算。

## 功能特性

- 支持中英文城市名搜索
- 支持国家搜索，同时展示所有匹配国家的主要城市
- 实时显示当地时间、时区、UTC偏移
- 首都城市标记
- 城市级别标注（省会、地级市、县等）
- 最近搜索记录（本地持久化）
- 点击城市显示详细信息
- 数据库 gzip 压缩嵌入 exe，首次运行自动解压
- 实时汇率换算，支持 158 种货币

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri v2
- **数据库**: SQLite + FTS5 全文搜索（gzip 压缩嵌入）
- **数据源**: GeoNames 离线数据（繁简转换 via OpenCC）、open.er-api.com 实时汇率

## 快速开始

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri dev
```

### 构建 exe

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/` 目录。

## 数据库

数据库通过 `scripts/build_db.py` 从 GeoNames 原始数据构建，包含：
- 68,787 个城市
- 252 个国家
- 18,043 个城市中文名（繁体自动转简体）

构建后生成 `src-tauri/data/data.db.gz`（约 5MB），编译时嵌入 exe。

### 重新构建数据库

方式一：使用下载脚本（需配置代理）：

```bash
python scripts/download.py
```

方式二：手动下载以下文件到 `scripts/raw_data/` 目录：

1. **cities5000.zip** - https://download.geonames.org/export/dump/cities5000.zip
2. **alternateNamesV2.zip** - https://download.geonames.org/export/dump/alternateNamesV2.zip
3. **countryInfo.txt** - https://download.geonames.org/export/dump/countryInfo.txt

然后运行：

```bash
python scripts/build_db.py
```

### 数据库版本

数据库版本号在 `src-tauri/src/main.rs` 的 `DB_VERSION` 常量中管理。更新数据库后需递增版本号，运行时会自动删除旧库并重新解压。

## 项目结构

```
global-data-tool/
├── src/                    # Vue 前端
│   ├── components/         # 组件（CityCard, CurrencyPage...）
│   ├── types/              # TypeScript 类型
│   ├── utils/              # 工具函数（时区计算）
│   └── styles/             # 样式
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs         # 入口：数据库版本管理、解压
│   │   ├── commands.rs     # Tauri 命令
│   │   ├── db.rs           # 数据库层（FTS5 搜索）
│   │   ├── models.rs       # 数据模型
│   │   └── timecalc.rs     # 时区时间计算
│   └── data/               # 构建产物（gitignore）
├── scripts/                # 数据预处理脚本
│   ├── build_db.py         # 构建数据库
│   ├── download.py         # 下载数据源
│   └── raw_data/           # 原始数据（gitignore）
└── package.json
```

## 运行时要求

- Windows 10+（需要 WebView2 运行时，Win10+ 自带）

## 许可证

MIT
