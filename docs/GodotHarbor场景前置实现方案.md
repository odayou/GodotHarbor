# GodotHarbor 场景前置（冷启动破局）实现方案

> 目标：把触发点种在"痛苦发生的现场"，而非门上。解决"用户急用时才想起来、去装、去熟悉"的冷启动死局。
> 基于本地代码 `D:\rustCode\GodotHarbor` @ `089bf59` 实测接口，非设想。

---

## 0. 代码现状（决定可行性的硬事实）

| 能力 | 现状（已存在？） | 代码位置 | 对"前置"的意义 |
|---|---|---|---|
| `harbor.lock` 写入项目目录 | ✅ 已写 `project.path`，非 data_dir | `lockfile/mod.rs:209` `get_lock_path(project_path)`；`cli/mod.rs:1860` | 它**已是潜在提交物**，缺的是"约定提交 + 首次打开自动还原" |
| `harbor lock generate / verify` 命令 | ✅ 已存在 | `cli/mod.rs:1844 / 1879` | lockfile 可手动产出，但用户不知道该把它提交进仓库 |
| MCP server 与 `harbor_*` 工具集 | ✅ 已注册（`bind/apply/sync/install_plugin/switch_engine/check_drift/restore`） | `mcp/tools.rs:153-1080`；`bin/mcp_server.rs:2` | "程序化唤起 Harbor"通道已搭好一半，Gap 是"编辑器内怎么连" |
| 首次打开项目检测 / watcher | 🟡 有 `watcher/mod.rs`（监听 `project.godot` 变动） | `watcher/mod.rs:66-78` | 可挂"发现 harbor.lock / 提示 restore"的钩子，但当前未做 |
| 编辑器内 Harbor 插件（@tool addon） | ❌ 不存在 | — | 真实 Gap：用户在 Godot 内部无法触达 Harbor |
| lock 的 gitignore 约定 | ❌ 不存在 | — | 当前 `harbor.lock` 没被建议提交/忽略，用户无感 |
| clone 后自动还原钩子 | ❌ 不存在 | — | 用户 clone 别人项目后仍需手动想起 Harbor |

**结论**：前三条"基础设施"已就位，真正的工程 Gap 集中在两件事——①**编辑器内接入 MCP 的客户端**；②**把 lock 从"手动产物"变成"自动 onboarding 触发物"**。下面三条方案都基于这个事实，不要求重写后端。

> **关键补充（最上游分发层）**：以上方案（A/B/C）都有一个隐藏前提——"用户已经装了 Harbor"。但冷启动真正的第一道坎是：**用户从没听过你 → 为什么要装 → 什么契机让他装**。这条"从 0 到 1 的安装动机"链条，A/B/C 都不解决。它不是工程问题，是**分发与钩子**问题。见文末「方案 D：官方模板/资产钩子（最上游安装契机）」。

---

## 方案 D：官方模板 / 资产钩子（最上游的"从没听过你 → 愿意装"）

**问题本质**：方案 A 的 @tool 插件、方案 B 的 lock 提示、方案 C 的模板生成，**全部预设"用户已经装了 Harbor"**。但一个 1★ 项目的冷启动第一问是——用户凭什么第一次点"下载安装"？他连你存在都不知道。抢 OS 启动入口（Godots/Launcher 的地盘）是错的，但**借官方流量入口、用"免费好东西"当诱饵**是对的。

### D1 ｜ 维护几个优质项目模板，上架 Godot 官方 Asset Library（Templates）与 Asset Store（Full Project）

- **官方分发事实**（实测文档）：Godot 有两个官方分发位——旧 **Asset Library**（编辑器 Project Manager 内可见；**Templates/Demos/Projects 类目只在"新建项目"界面出现**，即用户点"新建"那一刻就看到）+ 新 **Asset Store**（store.godotengine.org，有 Publisher 品牌页、下载/访问 analytics、审核队列、Full Project / Addon 类目）。两者均手动审核（约几天），要求资产能跑、有 LICENSE、有图标、英文描述。
- **为什么这是正解**：模板是纯 Godot 项目，用户在"新建项目"界面**自然刷到**——**不需要先知道 Harbor 存在**。机制是"我提供一个好到他愿意用的模板 → 模板里藏着 Harbor 的钩子 → 他为了用模板/还原模板依赖顺手装上 Harbor"。把"独立 App 的认知负担"前置成"免费模板的获得感"。
- **钩子落点（模板内部怎么接 Harbor）**：模板仓库根目录放 `harbor.yaml`（声明此模板推荐的插件集合 + 版本）+ 一个**可选** `@tool` 引导插件（首次打开模板时非阻塞提示"用一键 Harbor 还原全部推荐插件"）。注意此处钩子是**软性的**：模板脱离 Harbor 也能跑（核心文件直接打进模板），Harbor 只负责"后续增删插件/团队同步"——这正是方案 A/B 的触发入口，D1 负责把人领进门，A/B 负责进门后留人。
- **模板选题（直接命中高频痛点，提高上架被用的概率）**：
  1. **2D 平台跳跃 starter**（含状态机、相机、TileMap 辅助、保存系统——拼一堆高频插件）；
  2. **UI/工具型 starter**（含信箱 UI、本地化、设置面板框架）；
  3. **多人/Netcode 实验场**（含网络同步插件脚手架）；
  4. **编辑器工具模板**（含若干生产力插件）。
  选题原则：**每个模板 = 一组自然要装在一起的插件**，这样"用 Harbor 管这些插件"的动机在模板语境里自洽。
- **工程量**：每个模板约 1–3 天（复用 Harbor 的插件仓库做素材）；上架为一次性后台流程；维护成本在跟随 Godot 大版本（每大版本要重提一次条目）。
- **可行性风险**：① 审核排队几天，无硬阻塞；② 官方要求资产"能跑 + 有 LICENSE 文件"，模板天然满足；③ 不能硬塞推广——钩子必须做成"不用 Harbor 也能跑、用了更省事"，否则违反 Asset Library 的"资产本身可用"原则、审核也会被拒。这点反而和 Harbor 的"插件复用"基因一致（非强制绑定）。

### D2 ｜ 把 Harbor 自身也作为一个 Addon 上架？——不建议优先

- 把 Harbor 桌面 App 当 Addon 上架**不成立**（它是 Tauri 桌面程序，不是编辑器插件）；若做方案 A 的 @tool 插件，倒可以**以 Addon 类目上架**，作为"已装桌面的用户在编辑器里连 Harbor"的官方可信渠道。优先级低于 D1：**先有模板把人领进门，再谈编辑器内插件上架**。

### D3 ｜ 钩子的转化漏斗（把 D1 接回 A/B/C）

```
官方"新建项目"界面刷到优质模板 (D1, 无需知道 Harbor)
        ↓ 下载模板、打开项目
模板自带 harbor.yaml + 软性@tool提示 (D1 落点)
        ↓ 用户想"一键装齐推荐插件/团队同步"
首次打开非阻塞提示 → 装 Harbor 桌面端 (B 的 onboarding)
        ↓ 装完后
编辑器内 @tool 插件接管日常 (A) / clone 他人项目自动还原 (B/C)
        ↓
养成习惯 + 每个他发出的项目都带 harbor.lock → 拉新飞轮 (B/C 网络效应)
```

**D 的核心价值**：它解决的是**所有 A/B/C 都不碰的"第一次"**——把"独立 App 下载决策"替换成"免费模板获得感"，用官方入口把"从没听过你"变成"已经在用你产出的东西"。这是 1★ 项目绕开"记忆冷启动"、借官方流量的正路，也是你直觉里"维护优质官方模板当钩子"的落地。

---

## 方案 A：编辑器内 Harbor 插件（@tool addon）— 把触发点种在痛苦现场

**痛点现场**：用户改项目时，在 Godot 编辑器内发现"插件没装/版本不对/引擎不对"。此刻他最想一键修好，而不是切出去开一个独立 App。

### 可行路径（两条，复杂度递增）

**A1（推荐，零后端改动）｜ Godot 插件通过本地 MCP 客户端连已运行的 `harbor-mcp-server`**
- 实现一个 GDScript `@tool` 插件（放 releases 里分发，或 Harbor 安装时自动注入到 `addons/harbor_bridge/`）。
- 插件启动后连 `harbor-mcp-server`（已存在，`bin/mcp_server.rs`）。Godot 4 支持 `WebSocketClient`，MCP 走 stdio 或 ws——若当前只有 stdio，需给 mcp_server 加一个 `--transport ws --port 18734` 模式（小改动，约 30 行）。
- 插件暴露编辑器面板：「检测本项目 Harbor 状态 → 一键 apply / restore / switch_engine」。直接调用已注册的 `tool_apply` / `tool_check_drift` / `tool_switch_engine`。
- **可行性**：后端 `tool_apply` 等已验证可用，新写代码集中在 GDScript 插件（约 200-400 行）+ mcp transport 一个小开关。**不碰 Rust 核心逻辑**。

**A2（更强，但重）｜ Godot 插件直接调 Harbor 的 CLI**
- 插件用 `OS.execute("harbor", ["apply", project_name])` 调本地 CLI（`cli/mod.rs` 的 `Apply` 命令已存在）。
- 优点：完全不依赖 MCP transport。缺点：需 Harbor CLI 在 PATH 且常驻，跨平台 PATH 处理繁琐；不如 A1 干净。

### 关键风险
- `@tool` 插件在编辑器启动期跑，若 Harbor 没装/没运行，插件必须**静默降级**（不弹错、不卡编辑器），否则会被用户当成 Godot 本身的 bug 卸载。
- GDScript 写 WebSocket 客户端 + JSON-RPC 有样板代码量，但社区有现成 `godot-mcp` 客户端可借鉴。

---

## 方案 B：把 `harbor.lock` 变成项目提交物 + 首次打开自动还原

**这是性价比最高的一条**，因为基础设施已 90% 就位。

### 具体改动（文件级）

1. **落地位置已是项目目录**（`lockfile/mod.rs:209` 写 `project.path`）——无需挪位置。
2. **在 `harbor lock generate` 时主动写/提示 `.gitignore` 与提交建议**（`cli/mod.rs:1844 cmd_lock_generate`）：
   - 生成后若项目根无 `.gitignore`，追加 `harbor.lock` 的**反向约定**——实际应建议**提交** `harbor.lock`（不是忽略），所以这里应输出一句：`✓ 已将 harbor.lock 写入项目；建议 `git add harbor.lock` 提交，使协作者克隆后自动还原环境`。
   - 或在 Harbor 文档/首次 bind 成功时提示"提交 harbor.lock"。
3. **首次打开项目的自动还原钩子**（`watcher/mod.rs` 已是监听 `project.godot` 的入口）：
   - 在 watcher 检测到某个项目被 Harbor 纳入管理且存在 `harbor.lock` 时，比对当前 `addons/` 与 lock——若不一致，在 GUI 弹一个**非阻塞提示**：「检测到 harbor.lock 与当前环境不一致，是否还原？（绑定插件 X 个）」。
   - 注意：**不要静默自动改用户的 `addons/`**，否则和上次我们担心的"破坏性操作"一样危险。用"提示 + 一键还原"而非"自动还原"。
4. **clone 后引导**：可提供一个 `harbor bootstrap <repo_url>` 命令，内部 `git clone` → 检测 `harbor.lock` → 提示 apply。或在 README/模板里写明"克隆后跑 `harbor apply`"。

### 为什么这一条决定性强
- 每个提交 `harbor.lock` 的仓库，都是一个**自带 onboarding 的广告牌**：协作者 clone 后第一件事就是被引导装 Harbor 并还原——**用产物前置代替记忆前置**，拉新成本趋近于零。
- 不需要抢 OS 启动入口，不稀释插件基因，且和 Godots/Launcher 零竞争。

### 风险
- lock 里若存了**绝对路径**或**机器相关字段**，跨机器还原会失败。需确认 `HarborLock` 结构用相对路径/插件名+版本（从 `lockfile/mod.rs:203` 看 `godot_version` 是相对语义，但需实测确认无绝对路径泄漏）。
- "提示还原"若太频繁会烦，需做"已忽略本项目"的本地记忆。

---

## 方案 C（谨慎）：项目创建入口的顺手带入

**不做独立启动器**，只在"建项目"这一步借 Godot/模板的力：

- Harbor 已有 `template.rs`（从模板建项目，`commands/template.rs:720` 写 `project.godot`）。可在"用模板建项目"成功时，顺带 `harbor lock generate` 并把常用插件 bind 进去——新项目**自带 harbor.lock**，天然进入 B 的飞轮。
- 不在 OS 层做 launcher，避免与 Godots/Launcher 正面竞争，也避免"用户得先想起你"。

---

## 方案 E：剥离内置模板（与 D 协调——D 加钩子渠道，E 解耦内容）

**代码事实（已核实 `089bf59`）**：`src-tauri/templates/` 下硬编码 **6 个内置模板**（`builtin-2d-platformer` / `builtin-2d-rpg` / `builtin-3d-starter` / `builtin-blank-recommended` / `builtin-multiplayer` + framework，共 98 文件，打包进安装包）；`ensure_builtin_templates`（`commands/template.rs:1174`）把每个模板的**具体内容写死在 Rust 里**——名字、描述、分类、**写死的插件集**（Phantom Camera `v0.9.4.2`、GdUnit4 `6.1.3`、Dialogic 等，含 git URL + 版本）、目录结构、input mappings、layer names、autoloads。用户**必须装 Harbor 才能用这些模板**。

**判断：该剥离，但要剥对层次**

| 剥离对象 | 是否剥 | 理由 |
|---|---|---|
| ① 模板**内容**硬编码进 Rust | ✅ 剥 | 内容（插件版本、framework 代码、按键映射）变化快却锁在 app 发版；Phantom Camera 一升级或有漏洞，Harbor 不发版就带病。改为外部 manifest（JSON / 远程 registry），`ensure_builtin_templates` 改读 manifest，内容可热更、不随 app 发版 |
| ② 模板**分发**依赖内置 | ✅ 剥（转 D1） | 见方案 D：把同一套模板上架 Godot 官方 Store 作独立钩子渠道。这是"新增渠道"不是"删功能" |
| ③ 套模板 **+ 插件集绑定**能力 | ❌ 不剥 | Harbor 建项目一键拉"推荐插件集"并生成结构，是它与 Godot 官方模板馆、Godots/Launcher 的**差异化点**，必须保留并强化 |

**具体实现路径**
- **内容外置**：把 6 个模板定义从 `template.rs` 抽成 `templates/manifest.json`（或远程 registry），`ensure_builtin_templates` 改为读取 manifest 注册；首次启动仍注入一个**精简 baseline**（保证离线 / 墙内可用），远程 manifest 作扩展。改动量 ~200–300 行 + manifest schema 设计。
- **分发双轨**：内置 baseline 保留兜底；同一套模板上架官方 Store（资产本身能跑 + LICENSE + 图标 + 审核），模板内 `harbor.yaml` 埋软引导（不装 Harbor 也能跑）。
- **能力强化**：保留并突出"模板 + 插件集一键绑定"——这是 Harbor 独有层；剥离的是"馆"（交给官方 / 社区），留下的是"插件集绑定引擎"。

**基因收益**：内置"模板馆"本质在抢 Godot 官方 PM 已占的位（Harbor 真基因位是插件复用，不该抢启动器 / 模板馆）。剥离"馆"、保留"绑定层"，定位更锐——既回应 D 的钩子需求，又消除与官方模板馆的正面撞车。

**可行性风险**：① manifest 需约定 schema 且兼容旧版内置模板迁移；② 远程 manifest 在墙内可能拉不到，故必须保留本地 baseline 兜底；③ 若全删内置、只靠远程，离线用户无模板可用——故"内置 baseline + 远程扩展"而非清空。

---

## 落地优先级与工作量估算

| 方案 | 改动量 | 关键文件 | 是否需后端 | ROI | 优先级 |
|---|---|---|---|---|---|
| **B（lock 提交 + 首次提示还原）** | 小（CLI+watcher 提示，~150 行） | `cli/mod.rs:1844`、`watcher/mod.rs`、`lockfile/mod.rs` | 否（仅接线） | 极高（自带拉新飞轮） | **P0** |
| **A1（编辑器内 MCP 插件）** | 中（GDScript 插件 ~300 行 + MCP ws transport ~30 行） | 新 `addons/harbor_bridge/`、`mcp/server.rs` | 小（transport 开关） | 高（现场触发） | **P0/P1** |
| **E（剥离内置模板内容 → 外置 manifest + 官方 Store 钩子）** | 中（~200–300 行 + manifest 设计） | `commands/template.rs:1174`、`src-tauri/templates/` | 否（仅内容治理解耦） | 高（治理解耦 + 接 D1 钩子） | **P1** |
| **C（模板建项目带入 lock）** | 小（~30 行） | `commands/template.rs:720` | 否 | 中（喂 B 的飞轮） | P1 |
| A2（CLI 直调） | 中 | GDScript + PATH 处理 | 否 | 中低（PATH 脆弱） | 备选 |

## 一句话原则
**用"官方模板钩子（D/E）+ 编辑器内接入（A）+ 项目产物自动 onboarding（B/C）"前置，不靠"抢 OS 启动入口"前置。** Harbor 的真正基因位是**插件复用**：剥离"模板馆"内容（E）、把钩子交给官方流量入口（D）、在痛苦现场（编辑器 / clone 后）自动接住用户（A/B/C）——这是 1★ 项目绕过记忆冷启动、借官方流量与他人仓库自动拉新的正路。
