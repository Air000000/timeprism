<h1 align="center">TimePrism</h1>

<p align="center">
  <strong>不必时刻盯着计时器，也能看清电脑时间究竟花在了哪里。</strong>
</p>

<p align="center">
  Windows-first、local-first 的专注与活动记录工具：支持隐私感知的前台应用追踪、Idle 校正、提醒，以及银月桌宠。
</p>

<p align="center">
  <img alt="Release" src="https://img.shields.io/github/v/release/Air000000/timeprism?style=for-the-badge&sort=semver">
  <img alt="CI" src="https://github.com/Air000000/timeprism/actions/workflows/ci.yml/badge.svg?branch=main">
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white">
  <img alt="Vue" src="https://img.shields.io/badge/Vue-3-42B883?style=for-the-badge&logo=vuedotjs&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-backend-B7410E?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="License" src="https://img.shields.io/github/license/Air000000/timeprism?style=for-the-badge">
</p>

<p align="center">
  <strong>中文</strong> ← 当前 · <a href="docs/en/README.md">English</a>
</p>

<p align="center">
  <a href="https://github.com/Air000000/timeprism/releases/tag/v0.1.0"><strong>下载 v0.1.0</strong></a> ·
  <a href="#product">产品</a> ·
  <a href="#engineering-highlights">工程亮点</a> ·
  <a href="#architecture">架构</a> ·
  <a href="#privacy">隐私</a> ·
  <a href="#development">开发</a>
</p>

---

## 为什么做 TimePrism

传统计时器有一个天然问题：只有在你记得“开始计时”和“结束计时”时，它才有效。

TimePrism 采用了另一种思路：后台采样当前前台应用，在写入前先应用隐私规则，再通过本地规则完成活动分类，把结果整理成学习、休息、Idle 时段、提醒和应用使用情况。

核心流程全部在 Windows 本地运行，不需要账号，也不依赖云服务。

| 自动记录 | 本地优先 | 可校正 |
| --- | --- | --- |
| 后台采样前台应用，不要求手动开关计时器。 | 核心数据存储在本地 SQLite，正常持久化前先经过隐私处理。 | Idle 时段由用户确认，可归因到之前的应用，也可标记为学习 / 休息 / 离开，或稍后处理。 |

> **当前版本：** 正式版 <strong>v0.1.0</strong> 已提供 NSIS 安装程序和 MSI 安装包。TimePrism 目前尚未进行 Windows 代码签名，因此 SmartScreen 可能提示“未知发布者”。

<!--
SHOWCASE MEDIA SLOT
完成经过隐私检查的 v0.1.0 实机截图后，在这里插入 docs/media/timeprism-demo.gif，
并按照 docs/media/README.md 加入两行截图表格。
在媒体文件真正提交前，不要提前引用不存在的图片。
-->

<a id="product"></a>
## 产品能力

| 模块 | 功能 |
| --- | --- |
| **首页** | 展示当天学习/休息节奏、可编辑学习目标、提醒、待处理信号和最近活动。 |
| **数据看板** | 查看常用应用、最近记录、历史使用、热力图与每日时间组成。 |
| **专注守护** | 为未知应用分类、处理 Idle 时段、编辑规则并查看采样诊断。 |
| **提醒** | 支持一次性、每日和每周提醒，以及完成、恢复和稍后提醒。 |
| **隐私控制** | 配置浏览器标题处理方式、仅白名单记录等本地持久化策略。 |
| **银月桌宠** | 提供轻量提醒、Idle 决策、快捷操作、学习/休息统计和小型数据面板。 |

### 银月桌宠

银月不是嵌在主界面里的装饰，而是一个独立的 Tauri always-on-top 窗口。

它支持拖拽动画、左右贴边、紧凑提示、原生右键菜单和伴随小面板；窗口尺寸还会根据不同显示器的分辨率与 DPI 做自适应，避免在多屏环境中固定使用同一套物理像素尺寸。

## 工作流程

~~~text
前台窗口
   |
   v
隐私处理
   |
   v
本地应用规则分类
   |
   v
SQLite 持久化
   |
   +------> 首页 / 数据看板
   +------> 专注守护
   +------> 提醒
   `------> 银月桌宠 / 小面板
~~~

TimePrism 将“活动归因”和“活动分类”分开处理。一个经过确认的 Idle 时段可以重新归因到某个应用，但不会因此改写这个应用原有的 学习 / 休息 / 未分类 规则。

<a id="engineering-highlights"></a>
## 工程亮点

### 1. 入库前隐私处理

前台窗口数据可能包含敏感标题，因此 TimePrism 的隐私决策发生在正常持久化之前，而不是先保存原始数据、再在展示层做遮挡。

- 浏览器标题支持 <code>FULL</code>、<code>BLUR</code> 和 <code>NONE</code>。
- 仅白名单模式可以阻止未列入白名单的应用写入正常使用记录。
- 对私密 / 无痕浏览窗口有显式处理。
- 诊断路径不会在隐私边界之后继续保留原始窗口标题。

### 2. Idle 校正避免重复计时

进入 Idle 阈值之前采集到的前台样本只是暂定记录。

当用户处理 Idle 提示时，TimePrism 会事务性替换重叠的使用区间，而不是再追加一条新的 Idle 记录，从而避免同一段时间既算作前台活动、又算作 Idle / 休息。

确认后的区间还会保留 <code>FOREGROUND</code> / <code>IDLE_CONFIRMED</code> 等来源信息，便于区分原始采样和人工校正。

### 3. 跟踪状态持久化与启动顺序

首次启用追踪、Pause / Resume 状态和旧数据库迁移都持久化在 SQLite 中。

产品 WebView 只有在数据库初始化成功之后才会创建，避免前端已经开始查询，而数据库表结构还未完成初始化的启动竞态。

### 4. 多显示器桌面运行时

桌宠使用逻辑尺寸和显示器自适应缩放，而不是固定物理像素窗口。

测试覆盖了高 DPI 场景，也覆盖“较低物理分辨率显示器拥有相近逻辑桌面空间”的情况，降低跨屏移动时尺寸异常的风险。

### 5. Release 本身也是工程门槛

Windows Release 工作流会检查 Node / Cargo / Tauri 版本同步、前端类型、Rust 测试以及真实安装包生成。

涉及发布的 PR 可以先构建 NSIS / MSI 验证产物；版本 Tag 会创建 Draft GitHub Release，而最终发布仍要求真实 Windows 安装、启动和 smoke test 通过。

<a id="architecture"></a>
## 架构

~~~text
                     TimePrism 桌面端

       +----------------+  +---------------+  +----------------+
       |  Main WebView  |  |   银月桌宠     |  |   Pet Panel    |
       +--------+-------+  +-------+-------+  +--------+-------+
                |                  |                   |
                +------------------+-------------------+
                                   |
                                   v
                  Vue views / composables / src/lib
                                   |
                                   v
                        typed Tauri command layer
                                   |
                                   v
             +---------------- Rust backend ----------------+
             |                                               |
             |   services   ->   domain   ->   db/SQLite     |
             |      |                                        |
             |      +------ Windows / Tauri APIs             |
             +-----------------------------------------------+
~~~

前端把交互状态拆分到 Vue 组件与 composables 中，<code>src/api/commands/*</code> 负责提供类型化的 Tauri command 调用。

Rust 侧保持 command 注册层尽量薄，把行为下沉到 <code>services/*</code>，领域数据结构放在 <code>domain/*</code>，数据库生命周期与迁移集中在 <code>db/*</code>。

## 验证

主 CI 在 Windows 上针对 Pull Request 和 <code>main</code> 推送运行：

~~~text
pnpm run typecheck
pnpm run build:check
cargo check --all-targets
cargo test
~~~

测试覆盖数据库初始化与迁移、追踪状态持久化、隐私规则、Idle attribution / provenance、提醒周期、分析边界、桌宠窗口缩放以及首页目标编辑链路。

v0.1.0 的最终 Windows 安装包也在发布 Draft Release 之前进行过真实安装和 smoke test。

CI 和 smoke test 是这个项目的回归证据，但不等同于“已经达到生产级加固”的宣称。

<a id="privacy"></a>
## 隐私

TimePrism 按 local-first 原则设计：

- 核心使用记录保存在本地 SQLite；
- v0.1.0 不需要云同步和账号；
- 隐私处理发生在正常使用记录持久化之前；
- 浏览器标题处理策略可配置；
- 仅白名单模式可以限制哪些应用被记录；
- TimePrism 不采集屏幕截图，也不记录键盘输入内容。

由于前台窗口数据天然可能包含敏感信息，因此隐私在这里属于“采集与数据模型”的一部分，而不是只在 UI 层提供一个开关。

实现细节见 [Database & Privacy](docs/DATABASE_AND_PRIVACY.md)。

## 下载

当前 Windows 正式版本为 [TimePrism v0.1.0](https://github.com/Air000000/timeprism/releases/tag/v0.1.0)。

Release 包含：

- <code>TimePrism_0.1.0_x64-setup.exe</code> — NSIS 安装程序
- <code>TimePrism_0.1.0_x64_en-US.msi</code> — MSI 安装包

项目目前没有 Windows 代码签名，因此 SmartScreen 可能提示未知发布者。

<a id="development"></a>
## 本地开发

### 环境要求

- Node.js 24
- pnpm 9.15.9
- Rust stable toolchain
- Tauri v2 系统依赖
- 若要运行完整的前台捕获 / Windows runtime 链路，需要 Windows

### 启动

~~~bash
git clone https://github.com/Air000000/timeprism.git
cd timeprism
pnpm install
pnpm tauri dev
~~~

### 验证

~~~bash
pnpm run typecheck
pnpm run build:check
cd src-tauri
cargo check --all-targets
cargo test
~~~

### 构建安装包

~~~bash
pnpm tauri build
~~~

## 技术栈

| 层 | 技术 |
| --- | --- |
| 桌面壳 | Tauri 2 |
| 前端 | Vue 3、TypeScript、Vite |
| 后端 | Rust |
| 存储 | SQLite / rusqlite |
| 平台集成 | Windows + Tauri window APIs |
| 验证 | GitHub Actions、vue-tsc、Vite、Cargo |

## 仓库结构

~~~text
timeprism/
|- src/
|  |- components/          # 主界面与桌面 UI
|  |- composables/         # 功能状态与视图工作流
|  |- api/commands/        # 类型化 Tauri command 封装
|  |- lib/                 # 可复用 UI / domain helper
|  |- App.vue              # 主桌面壳
|  |- pet.ts               # 银月桌宠入口
|  `- pet-panel.ts         # 桌宠伴随面板
|- src-tauri/
|  `- src/
|     |- db/               # SQLite 连接、迁移、生命周期
|     |- domain/           # 后端数据模型与 command payload
|     |- services/         # tracking / privacy / idle / analytics / windows 等
|     `- lib.rs            # Tauri bootstrap / command 注册
|- docs/                   # 架构、隐私、测试、决策与发布证据
`- .github/workflows/      # CI 与 Windows Release pipeline
~~~

## 文档

从 [docs/README.md](docs/README.md) 开始。主要文档包括：

- [Current System Map](docs/CURRENT_SYSTEM_MAP.md)
- [API Contracts](docs/API_CONTRACTS.md)
- [Database & Privacy](docs/DATABASE_AND_PRIVACY.md)
- [Testing Strategy](docs/TESTING_STRATEGY.md)
- [Smoke Tests](docs/SMOKE_TESTS.md)
- [Release Checklist](docs/RELEASE_CHECKLIST.md)
- [Product Media Capture Guide](docs/media/README.md)
- [Architecture Decisions](docs/DECISIONS.md)

## 当前范围

v0.1.0 明确以 Windows-first 为目标。云同步 / 账号体系、完整的跨平台前台捕获，以及导出 / 备份 / 恢复流程不属于这一版本已经完成的范围。

## License

TimePrism 使用 MIT License。
