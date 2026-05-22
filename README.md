# Global Data Tool - 全球城市时间搜索与汇率换算

轻量级 Windows 桌面应用，支持全球城市时间查询与实时汇率换算。

## 功能特性

### 城市时间搜索

- 支持中英文城市名、国家名搜索（FTS5 全文搜索 + LIKE 模糊匹配）
- 搜索国家名时自动展示该国 Top 8 城市（附大洲标签）
- 实时显示当地时间、日期、时区、UTC 偏移（每秒自动刷新）
- 首都城市标记（★），城市级别标注（省会、地级市、县级市、县）
- 点击城市弹出详情卡片（国家、时间、日期、时区、UTC偏移、人口、坐标）
- 最近搜索记录（本地持久化，最多 10 条，点击可重新搜索）
- 搜索输入 300ms 防抖

### 汇率换算

- 支持 158 种货币（含中英文名称）
- 双向换算：编辑任一侧金额自动计算对侧
- 一键交换上下货币
- 货币搜索筛选：按代码、英文名、中文名模糊匹配
- 智能精度：整数 2 位、≥1 取 4 位、<1 取 6 位小数
- 千分位格式化（大数字自动加逗号）
- 汇率显示智能翻转（始终以 ≥1 方向展示 "1 X = Y Z"）

### 通用

- 暗色主题 UI，左侧侧栏 Tab 导航
- 数据库 gzip 压缩嵌入 exe，首次运行自动解压
- 汇率数据本地缓存，离线可用

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri v2
- **数据库**: SQLite + FTS5 全文搜索（gzip 压缩嵌入）
- **数据源**: GeoNames 离线数据（繁简转换 via OpenCC）、open.er-api.com 实时汇率
- **CI/CD**: GitHub Actions（编译检查 + 自动构建发布）

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

## 数据源

| 数据 | 来源 | 说明 |
|------|------|------|
| 城市数据 | GeoNames `cities5000` | 人口 ≥ 5000 的城市，共 68,787 个 |
| 别名数据 | GeoNames `alternateNamesV2` | 多语言城市别名（英文 + 中英文变体） |
| 国家数据 | GeoNames `countryInfo` | 252 个国家，含中文名 |
| 汇率数据 | [open.er-api.com](https://open.er-api.com) | 免费 API，以 CNY 为基准，无需 API Key |

### 数据处理

- **繁简转换**: 中文别名通过 OpenCC（t2s）自动转为简体，共 18,043 个城市中文名
- **港澳台纠正**: HK/MO/TW 的 ISO 代码纠正为 CN，确保搜索"中国"时包含港澳台城市；纠正后移除首都标记（如台北不再标为中国首都）
- **城市过滤**: 仅保留 P 类（populated place）地点
- **城市级别**: PPLC=首都, PPLA=省会, PPLA2=地级市, PPLA3=县级市, PPLA4=县

## 汇率缓存机制

汇率数据存储在独立的 `exchange.db`（SQLite），与城市数据库分离。

1. 应用启动时读取本地缓存
2. 检查 API 返回的 `next_update` 时间，仅在过期后才后台静默刷新
3. 刷新成功后更新本地缓存，下次启动继续使用
4. 无网络时使用上次缓存数据，离线可用

## 数据库

项目仓库已包含预构建的 `src-tauri/data/data.db.gz`，正常使用和构建无需自行处理数据库。仅当需要修改数据过滤规则或更新数据源时才需重新构建。

### 重新构建数据库

方式一：使用下载脚本：

```bash
python scripts/download.py
```

方式二：手动下载以下文件到 `scripts/raw_data/` 目录：

1. **cities5000.zip** - https://download.geonames.org/export/dump/cities5000.zip
2. **alternateNamesV2.zip** - https://download.geonames.org/export/dump/alternateNamesV2.zip
3. **countryInfo.txt** - https://download.geonames.org/export/dump/countryInfo.txt

然后运行：

```bash
pip install opencc-python-reimplemented
python scripts/build_db.py
```

### 数据库版本

数据库版本号在 `src-tauri/src/main.rs` 的 `DB_VERSION` 常量中管理。更新数据库后需递增版本号，运行时会自动删除旧库并重新解压。

## 搜索实现

- **英文搜索**: FTS5 前缀匹配（`query*`），按首都优先、人口降序排序
- **中文搜索**: 自动识别 CJK 字符，走 FTS `name_cn` 列搜索
- **中文 fallback**: FTS 无结果时回退到 `city_names` 表的 LIKE 模糊匹配
- **国家搜索**: 匹配国家名/ISO 代码，展示该国主要城市

## CI/CD

| Workflow | 触发条件 | 说明 |
|----------|----------|------|
| `check.yml` | main 分支 push / PR | 前端 build + Rust cargo check 编译检查 |
| `build.yml` | tag push（v*） | 构建 MSI + exe，发布到 GitHub Release |

Release Body 使用项目根目录的 `RELEASE_NOTES.md`。

手动构建可在 Actions 页面选择分支触发 `workflow_dispatch`。

## 项目结构

```
global-data-tool/
├── src/                    # Vue 前端
│   ├── components/
│   │   ├── CityCard.vue        # 城市列表行（实时时间）
│   │   ├── CityDetail.vue      # 城市详情弹窗
│   │   ├── CityTimePage.vue    # 城市时间页面（搜索+列表+最近搜索）
│   │   ├── CurrencyPage.vue    # 汇率换算页面
│   │   ├── ResultList.vue      # 搜索结果（国家城市卡片+城市列表）
│   │   ├── SearchBox.vue       # 搜索输入框（300ms 防抖）
│   │   └── SideTab.vue         # 左侧侧栏 Tab 导航
│   ├── types/
│   │   └── index.ts            # TypeScript 类型定义
│   ├── utils/
│   │   ├── currencies.ts       # 158 种货币中英文名映射
│   │   └── time.ts             # 时区时间计算 + 实时 composable
│   └── styles/
│       └── main.css            # 全局样式
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs             # 入口：数据库版本管理、解压、ExchangeDb 初始化
│   │   ├── commands.rs         # Tauri 命令（搜索、最近记录、汇率 CRUD）
│   │   ├── db.rs               # 数据库层（FTS5 搜索 + 最近搜索 + ExchangeDb）
│   │   ├── models.rs           # 数据模型
│   │   └── timecalc.rs         # 时区时间计算（chrono + chrono_tz）
│   └── data/
│       └── data.db.gz          # 构建产物（gitignore）
├── scripts/                # 数据预处理脚本
│   ├── build_db.py             # 构建数据库
│   ├── download.py             # 下载 GeoNames 数据源
│   └── raw_data/               # 原始数据（gitignore）
├── .github/workflows/      # CI/CD
│   ├── build.yml               # 构建 + 发布 Release
│   └── check.yml               # 编译检查
├── RELEASE_NOTES.md            # Release 发布说明
└── package.json
```

## 运行时要求

- Windows 10+（需要 WebView2 运行时，Win10+ 自带）

## 许可证

MIT
