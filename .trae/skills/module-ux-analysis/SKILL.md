---
name: "module-ux-analysis"
description: "Analyzes a module's UX completeness across 6 dimensions (simplification, convenience, feature completeness, flow completeness, conflict tolerance, edge cases). Invoke when user asks to review/analyze a module's UX or find improvement opportunities."
---

# Module UX Analysis Skill

Systematically analyze a software module's user experience across 6 dimensions to identify improvement opportunities and prioritize fixes.

## When to Invoke

- User asks to review/analyze a module's UX quality
- User wants to find improvement opportunities in a feature area
- User asks for a comprehensive audit of a functional module
- User mentions "分析", "梳理", "体验", "流程" in context of a module

## Analysis Procedure

### Step 1: Code Survey (代码梳理)

Read all relevant source files for the target module:

1. **View files** (`src/views/`): Main page components, all dialogs, user interactions
2. **Store files** (`src/stores/`): State management, data flow, API calls
3. **API layer** (`src/api/`): Backend interface surface, available commands
4. **Type definitions** (`src/types/`): Data models, enums, interfaces
5. **i18n files** (`src/locales/`): Feature text, hints, error messages — reveals intended UX
6. **Router** (`src/router/`): Navigation structure, page relationships

Output: A complete flow diagram showing all user-facing operations and their connections.

### Step 2: 6-Dimension Analysis (六维度分析)

Analyze each dimension below. For every issue found, assign severity:
- 🔴 High: Blocks core workflow or causes data loss/confusion
- 🟡 Medium: Degrades experience significantly
- 🟢 Low: Nice-to-have improvement

#### Dimension 1: Operation Simplification (操作简化)

Check:
- [ ] Are there redundant steps that can be merged?
- [ ] Are there too many buttons/entries for the same goal?
- [ ] Can multi-step operations be combined into one-click?
- [ ] Is drag-and-drop supported where applicable?
- [ ] Are there unnecessary confirmation dialogs?

#### Dimension 2: Convenience Maximization (便利性最大化)

Check:
- [ ] Can users perform the most common action from the current context without navigation?
- [ ] Is there quick access from item cards (not just detail pages)?
- [ ] Are recent/frequent items surfaced?
- [ ] Do search/filter states persist across sessions?
- [ ] Are context menus (right-click) available?
- [ ] Are keyboard shortcuts provided?
- [ ] Can users customize default behaviors?

#### Dimension 3: Feature Completeness (功能实现完成度)

Check:
- [ ] Are all backend APIs exposed in the UI?
- [ ] Are there features implied by data models but not implemented in UI?
- [ ] Are there "view-only" features that should be actionable?
- [ ] Are there missing features that competitors/reference apps have?
- [ ] Are batch operations supported where applicable?
- [ ] Is there a recommendation/discovery mechanism?

#### Dimension 4: Flow Completeness (流程完备性)

Check:
- [ ] Is there a guided onboarding for first-time users?
- [ ] Is the end-to-end workflow connected or fragmented?
- [ ] Is there verification/feedback after critical operations?
- [ ] Is undo/rollback supported?
- [ ] Are downstream impacts shown before destructive operations?
- [ ] Are change logs/release notes displayed where relevant?

#### Dimension 5: Conflict Tolerance (冲突容错)

Check:
- [ ] Are health checks performed on existing state (e.g., broken symlinks)?
- [ ] Are path/resource conflicts detected before operations?
- [ ] Are duplicate checks performed during import?
- [ ] Is there recovery from failed operations (resume/retry)?
- [ ] Are compatibility constraints enforced (e.g., version mismatch warnings)?
- [ ] Are dependency requirements validated before operations?

#### Dimension 6: Edge Cases (场景边界情况)

Check:
- [ ] What happens when storage paths change?
- [ ] Is offline mode supported or gracefully degraded?
- [ ] Are orphaned resources cleaned up when parent is deleted?
- [ ] Is concurrent access handled?
- [ ] Are there performance concerns at scale (virtual scrolling, pagination)?
- [ ] What happens when external resources become unavailable?

### Step 3: Priority Ranking (优先级排序)

Select the Top N improvements using this framework:

| Priority | Criteria |
|----------|----------|
| **P0** | Blocks core workflow, causes data loss, or makes the app's value proposition questionable |
| **P1** | Significantly degrades experience, causes confusion, or creates risk of errors |
| **P2** | Improves efficiency, reduces friction, or enhances discoverability |
| **P3** | Polish, nice-to-have, or optimization for edge cases |

### Step 4: Output Format (输出格式)

Structure the analysis as:

```
## [模块名] UX分析报告

### 一、当前功能流程全景图
(ASCII flow diagram)

### 二、各环节详细梳理
(Table: 环节 | 已实现功能 | 涉及文件)

### 三、六维度问题分析
(6 tables, one per dimension, with columns: 问题 | 严重度 | 说明)

### 四、优先级排序（Top 10 改进项）
(Table: 优先级 | 改进项 | 预期收益)
```

## Key Principles

1. **Evidence-based**: Every issue must reference specific code (file path + line number)
2. **User-centric**: Evaluate from the user's perspective, not developer's
3. **Actionable**: Every issue should have a clear remediation direction
4. **Prioritized**: Not all issues are equal; focus on what matters most
5. **Holistic**: Consider the full lifecycle, not just individual features
6. **Comparative**: Reference similar tools (e.g., for Godot Harbor: gd-plug, GodotEnv, godam) when evaluating completeness

##  create optimization plan and execute it

    do it step by step , and check the result after each step 
to ensure the optimization is effective and efficient