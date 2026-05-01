---
name: "code-quality-analysis"
description: "Analyzes code quality across 7 dimensions (complexity, duplication, componentization, modularity, performance, concurrency, blocking). Invoke when user asks to optimize/refactor code or improve code maintainability."
---

# Code Quality Analysis Skill

Systematically analyze a code module's quality across 7 dimensions to identify improvement opportunities and implement optimizations.

## When to Invoke

- User asks to optimize or refactor a module's code
- User wants to improve code maintainability, performance, or architecture
- User asks for a comprehensive code quality audit
- User mentions "优化代码", "重构", "性能", "复杂度", "重复代码" in context of a module

## Analysis Procedure

### Step 1: Code Survey (代码梳理)

Read all relevant source files for the target module:

1. **View files** (`src/views/`): Component size, template complexity, script logic
2. **Composables/Hooks** (`src/composables/`, `src/hooks/`): Reusable logic extraction
3. **Store files** (`src/stores/`): State management, data flow patterns
4. **API layer** (`src/api/`): Backend interface surface, request patterns
5. **Backend commands** (`src-tauri/src/commands/`): Command handlers, business logic placement
6. **Backend modules** (`src-tauri/src/*/mod.rs`): Core logic, data structures, algorithms
7. **Type definitions** (`src/types/`, `src-tauri/src/models/`): Data model design
8. **Utility files** (`src/utils/`, `src-tauri/src/*/utils.rs`): Shared helpers

Output: A cross-module dependency map showing:
- Which modules call which other modules (call graph)
- Which modules share state or events
- Which modules depend on the same types/interfaces
- Which backend commands are called by which frontend components
- Potential cross-module impact zones for any change

### Step 2: 7-Dimension Analysis (七维度分析)

Analyze each dimension below. For every issue found, assign severity:
- 🔴 High: Critical performance bottleneck, severe maintainability issue, or correctness risk
- 🟡 Medium: Significant degradation of code quality or performance
- 🟢 Low: Improvement opportunity, best practice alignment

#### Dimension 1: Code Complexity (代码复杂度)

Check:
- [ ] Are there functions/methods exceeding 50 lines that should be decomposed?
- [ ] Are there deeply nested control flows (if/for/match > 3 levels)?
- [ ] Are there cognitive complexity hotspots (multiple conditions, early returns scattered)?
- [ ] Are there "god functions" that handle too many responsibilities?
- [ ] Are there large match/switch statements that could use strategy patterns?
- [ ] Are there complex boolean expressions that should be named/extracted?
- [ ] Is the cyclomatic complexity of any function > 10?

#### Dimension 2: Code Duplication (代码重复度)

Check:
- [ ] Are there copy-pasted code blocks across files (>5 identical lines)?
- [ ] Are there similar patterns that differ only in variable names or types?
- [ ] Are there repeated API call patterns that could be abstracted?
- [ ] Are there duplicated type definitions between frontend and backend?
- [ ] Are there repeated error handling patterns that could be centralized?
- [ ] Are there similar template structures in Vue components that could be extracted?
- [ ] Are there repeated validation logic that could be shared?

#### Dimension 3: Componentization (组件化)

Check:
- [ ] Are there monolithic Vue components > 500 lines that should be split?
- [ ] Are there inline template sections that repeat across components?
- [ ] Are there UI patterns (cards, lists, dialogs) that could be reusable components?
- [ ] Are there props drilling > 3 levels deep (should use provide/inject or store)?
- [ ] Are there computed properties that should be extracted into composables?
- [ ] Are there event handler patterns that could be generalized?
- [ ] Are there mixed concerns in components (business logic + UI + data fetching)?

#### Dimension 4: Modularity (模块化)

Check:
- [ ] Are there circular dependencies between modules?
- [ ] Are there modules with unclear boundaries or mixed responsibilities?
- [ ] Are there backend command handlers containing business logic that should be in service modules?
- [ ] Are there frontend API calls scattered in components instead of centralized API layer?
- [ ] Are there shared state accessed directly instead of through well-defined interfaces?
- [ ] Are there utility functions that belong in a domain-specific module?
- [ ] Is the module's public API surface minimal and well-defined?

#### Dimension 5: Performance (性能)

Check:
- [ ] Are there N+1 query patterns (sequential API calls in loops)?
- [ ] Are there unnecessary re-renders in Vue (missing v-memo, key issues, deep watchers)?
- [ ] Are there large lists without virtual scrolling?
- [ ] Are there expensive computations that should be memoized?
- [ ] Are there unnecessary data fetching (over-fetching, polling instead of events)?
- [ ] Are there synchronous operations that should be async (blocking I/O)?
- [ ] Are there memory leaks (event listeners not cleaned up, intervals not cleared)?
- [ ] Are there bundle size concerns (large imports, tree-shaking issues)?

#### Dimension 6: Concurrency (并发度)

Check:
- [ ] Are there sequential operations that could run in parallel (Promise.all, join handles)?
- [ ] Are there shared mutable state accessed without synchronization?
- [ ] Are there race conditions in async operations (stale closures, outdated state)?
- [ ] Are there lock contention issues (Mutex held too long, fine-grained locking possible)?
- [ ] Are there opportunities for background processing (spawn tasks, web workers)?
- [ ] Are there cascading waits that could be eliminated?
- [ ] Are there dead-lock risks (multiple locks acquired in inconsistent order)?

#### Dimension 7: Blocking Analysis (阻塞性分析)

Check:
- [ ] Are there synchronous file I/O operations on the main thread (should be async/offloaded)?
- [ ] Are there long-running computations blocking the UI thread?
- [ ] Are there network requests without timeouts?
- [ ] Are there UI operations waiting for backend responses that could be optimistic?
- [ ] Are there startup operations that delay initial render?
- [ ] Are there blocking dialogs that prevent background work?
- [ ] Are there sequential awaits that could be parallelized with Promise.all?
- [ ] Are there heavy DOM operations that could be batched or deferred?

### Step 3: Priority Ranking (优先级排序)

Rank ALL discovered improvements using this framework:

| Priority | Criteria |
|----------|----------|
| **P0** | Correctness risk, data corruption, or crash-causing issue |
| **P1** | Significant performance degradation, severe maintainability debt, or high bug risk |
| **P2** | Noticeable performance improvement, meaningful code quality improvement |
| **P3** | Minor optimization, best practice alignment, future-proofing |

**IMPORTANT**: ALL discovered issues must be ranked and addressed, not just the top N. Every issue found across all 7 dimensions must have a corresponding fix in the implementation plan.

### Step 4: Output Format (输出格式)

Structure the analysis as:

```
## [模块名] 代码质量分析报告

### 一、模块依赖关系图
(ASCII dependency diagram showing module relationships)

### 二、关键指标概览
(Table: 指标 | 当前值 | 评估)

### 三、七维度问题分析
(7 tables, one per dimension, with columns: 问题 | 严重度 | 位置 | 说明)

### 四、优先级排序（全部改进项）
(Table: 优先级 | 改进项 | 预期收益)
```

## Key Principles

1. **Evidence-based**: Every issue must reference specific code (file path + line number)
2. **Measurable**: Prefer quantifiable metrics (line count, call depth, response time) over subjective assessments
3. **Actionable**: Every issue should have a clear refactoring direction
4. **Conservative**: Don't over-engineer; prefer simple, incremental improvements over architectural rewrites
5. **Context-aware**: Consider the project's scale and stage — not every code needs to be enterprise-grade
6. **Side-effect-aware**: Document potential risks of each refactoring (what could break)

## Global Impact Analysis (全局影响分析)

**This is the most critical principle and must be applied BEFORE implementing any fix.**

When analyzing a module, a local optimization may appear beneficial in isolation but can break functionality in dependent modules. Every proposed change must be evaluated across the entire codebase, not just the target module.

### Mandatory Pre-Change Checklist

Before implementing ANY optimization, answer these questions:

1. **Callers**: Who calls this function/method/component? Search the entire codebase for all call sites. A change to a function signature, return type, or behavior will affect ALL callers.
2. **Callees**: What does this function depend on? Changing a dependency's behavior may cascade upward.
3. **State consumers**: If the change affects shared state (store, global ref, cache, static variable), who reads this state? A state shape change will break all consumers.
4. **Event listeners**: If the change affects emitted events or event payloads, who listens to these events?
5. **Type contracts**: If the change modifies a type/interface, who imports and uses this type? Type changes propagate through the dependency graph.
6. **API contracts**: If the change modifies a Tauri command signature or return value, both the frontend API layer and all calling components must be updated simultaneously.
7. **Data persistence**: If the change affects data format (JSON schema, cache structure, config format), existing persisted data must remain compatible or be migrated.
8. **Ordering assumptions**: If the change modifies execution order (e.g., making async what was sync, parallelizing sequential operations), verify no downstream code depends on the original order.

### Impact Assessment Process

For each proposed fix:

1. **Trace all references**: Use search tools to find every file that imports, calls, or references the code being changed
2. **Map the blast radius**: Categorize affected code into:
   - 🔴 **Direct impact**: Code that will definitely break without changes (must fix simultaneously)
   - 🟡 **Indirect impact**: Code that may behave differently after the change (must verify)
   - 🟢 **No impact**: Code that is unaffected (safe to ignore)
3. **Batch related changes**: If a change requires updates in multiple files, make ALL updates in the same step — never leave the codebase in a broken intermediate state
4. **Verify after each change**: After implementing a fix, check that all identified impact points still compile and function correctly

### Anti-Patterns to Avoid

- ❌ Optimizing a function's return type without updating all callers
- ❌ Changing a shared state structure without migrating existing data
- ❌ Making a synchronous function async without updating all call sites (forgotten await)
- ❌ Removing a "redundant" parameter that a downstream consumer relies on
- ❌ Parallelizing operations that have hidden ordering dependencies
- ❌ Extracting shared logic into a utility without preserving edge-case behavior
- ❌ Changing error handling in a way that swallows errors a caller was expecting to catch

## Create Optimization Plan and Execute It

- Fix ALL discovered issues, not just the top N
- Do it step by step, and check the result after each step to ensure the optimization is effective and efficient
- Every issue from the 7-dimension analysis must have a corresponding fix in the implementation plan
- P0 and P1 issues must be fixed; P2 and P3 issues should be fixed unless technically infeasible

### Execution Rules

1. **Auto-fix without asking**: After completing the analysis and priority ranking, immediately create a todo list and start implementing fixes. Do NOT ask the user "是否按计划执行" or wait for confirmation — proceed directly.
2. **Auto-commit on success**: After all fixes are implemented, run build verification (frontend typecheck + backend cargo check). If both pass, automatically commit to git with a descriptive commit message. Do NOT ask the user for permission to commit.
3. **Build failure handling**: If the build fails, fix the errors and retry. Only ask the user if the error cannot be resolved after 2 attempts.
4. **Regression check**: After each fix, verify that the original functionality is preserved (no behavioral changes unless explicitly improving correctness).
