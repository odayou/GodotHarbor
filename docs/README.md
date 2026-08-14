# GodotHarbor 文档体系

## 目录结构

```
docs/
├── README.md          — 本文件（文档体系规范）
├── wiki/              — 用户手册（同步到 GitHub Wiki，对外可见）
├── technical/         — 技术设计文档（内部）
├── ux/                — UX 分析报告（内部）
├── design/            — UI 风格指南 + 设计素材
├── screenshots/       — 截图素材
├── planning/          — 当前活跃的计划/路线图
├── archive/           — 被取代的旧文档（保留历史，不删除）
├── update-system.md   — 更新系统设计
└── Godot与GodotHarbor整合研究报告_2026-07-31.md

docs-private/          — 商业/运营敏感文档（.gitignore，不进仓库）
```

## 什么内容放哪里

| 内容类型 | 放在哪里 | 说明 |
|---|---|---|
| 用户手册 / 操作指南 | `docs/wiki/` | 通过 wiki-sync.yml 同步到 GitHub Wiki，对外可见 |
| 技术设计 / 架构分析 | `docs/technical/` | 内部技术文档 |
| UX 分析报告 | `docs/ux/` | 内部 UX 研究 |
| UI 风格 / 设计素材 | `docs/design/` | 设计资源 |
| 当前活跃的路线图 / 计划 | `docs/planning/` | 仅放当前活跃文档，被取代的移入 archive/ |
| 被取代的旧文档 | `docs/archive/` | 保留历史可追溯，不直接删除；每份头部有"📌 归档说明"标注接管文档 |
| 商业计划 / 推广策略 / 运营 | `docs-private/` | 敏感信息，.gitignore 排除，不进仓库 |

## 命名规范

- 计划类：`主题_v版本.md`（如 `产品方向锚定_v1.1.md`）或 `主题_日期.md`
- 报告类：`主题_版本.md`（如 `GodotHarbor整体UX分析报告_v1.0.md`）
- 归档文档保持原文件名，不重命名；头部加 `> 📌 归档说明：本文档已被 [接管文档名] 取代/承接` 标注

## 归档规则

- 文档被新版本取代时，移入 `docs/archive/`，不删除
- archive/ 中的文档不再维护，仅供历史查阅
- 每份归档文档头部必须加归档说明，指明接管文档，避免误读
- 如果同一主题有多个版本，全部保留在 archive/ 中

## 当前活跃文档（planning/）

### 路线图与状态类（信息类）

| 文档 | 角色 |
|---|---|
| `主路线图_v2.md` | 最新主路线图（2026-08-02），三线并行协调（协议/MCP/推广），取代 v1 路线图 |
| `项目当前状态_v1.md` | 整合完成度、技术栈、已修复问题、已实施架构决策（P0-P3 + P4 部分） |
| `已知问题与待办_v1.md` | P0-P3 分级的待修复 bug、技术债、未承接点子清单 |

### 方向与专题类（点子类）

| 文档 | 角色 |
|---|---|
| `产品方向锚定_v1.1.md` | 长期方向锚定 + 产品边界守则 + UID 风险矩阵 + .harbor.yml v2 规范 |
| `harbor-architecture-refactor-plan.md` | 架构改造 ADR 索引（P0-P4 决策记录与任务总览） |
| `MCP增强计划.md` | MCP 方向 ADR + Phase 2 待办 + 客户端集成文档待办 |
| `UX减法规划.md` | UX 精简专题 + 逆向流程设计原则附录 + 导入入口散落原诊断附录 |
| `Asset_Library迭代计划.md` | Asset Library 三阶段迭代 + MCP 对接说明附录 |
| `协议优化与软著计划.md` | 协议/软著/商标专题（待执行） |
| `update-strategy-plan.md` | 更新系统修复计划（Phase 1-7），与 docs/update-system.md 互补；待与 update-system.md 整合 |

## archive/ 文档清单（13 份，均带归档标注）

| 归档文档 | 接管文档 |
|---|---|
| 实施计划.md | 项目当前状态_v1 + 已知问题与待办_v1（性能优化方案） |
| 产品规划_v0.1.md | 产品方向锚定_v1.1（核心概念/错误提示三要素） |
| 迭代计划_v0.2.md | 已知问题与待办_v1（B2/E2 未承接项） |
| 项目状态.md | 项目当前状态_v1 |
| 实施路线图_v2.0.md | 产品方向锚定_v1.1（.harbor.yml v2/MCP 三层设计） |
| godot-harbor-评估报告.md | 产品方向锚定_v1.1（UID 风险场景矩阵） |
| 核心功能审视与下一步计划.md | 产品方向锚定_v1.1（边界守则/AssetPack 理念） |
| harbor-polish-plan.md | UX减法规划（附录B 导入入口散落诊断） |
| harbor-deep-polish-plan.md | 已知问题与待办_v1（F1/F2/T/E） + UX减法规划（附录A 逆向流程） |
| 后续工作计划_详细版.md | 主路线图_v2 + 项目当前状态_v1（完成度章节） |
| 三线并行执行路线图.md | 主路线图_v2（三线协调框架作为附录） |
| 软件打磨计划.md | 项目当前状态_v1（Phase 1+2 已完成） + 已知问题与待办_v1（S6/S7/S8/M/Phase 3） |
| 产品方向锚定_v1.0.md | 产品方向锚定_v1.1 |

## .trae/documents/ 说明

`.trae/documents/` 已清空，原 3 份文档已整合：harbor-polish-plan、harbor-deep-polish-plan 归档到 `docs/archive/`，update-strategy-plan 移到 `docs/planning/`。

## wiki 同步

`wiki-sync.yml` 只同步 `docs/wiki/` 到 GitHub Wiki，其他目录不同步。`docs/planning/` 在 `.gitignore` 中，不进仓库。
