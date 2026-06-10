---
name: "ux-simplify"
description: "Reduces UI redundancy and improves flow cohesion. Invoke when user asks to simplify UX, remove duplicate entry points, merge menus, or restructure navigation flows."
---

# UX Simplify Skill

Reduces user cognitive load by eliminating redundant entry points, merging duplicate menus, and improving cross-section flow cohesion.

## Core Principles

1. **One action, one entry point** — If the same action appears in 3+ places, keep the most discoverable one, remove the rest
2. **Flow over feature** — Actions should follow the user's mental model (task flow), not the system's data model (CRUD)
3. **Context over navigation** — Related actions should be accessible within the current context, not require page switching
4. **Progressive disclosure** — Primary actions visible, secondary in menus, tertiary in command palette

## Simplification Patterns

### Pattern 1: Menu Deduplication
- Right-click menu and ellipsis menu serving the same item → **Keep ellipsis only** (visible, consistent), remove right-click
- Exception: If right-click provides significantly different actions than ellipsis, keep both but ensure zero overlap

### Pattern 2: Empty State Consolidation
- Empty state actions duplicating top toolbar → **Remove from empty state**, keep toolbar only
- Exception: If the empty state is the ONLY place the user looks (e.g., first-time experience), keep a single primary CTA

### Pattern 3: Flow Integration
- Standalone section for a cross-cutting concern → **Integrate into the primary flow**
- Example: Templates as standalone page → Integrate into "Create Project" dialog as a step

### Pattern 4: Action Hierarchy
- Same action at 3+ locations → Apply hierarchy:
  1. **Primary location** (most contextual, e.g., inline button on the item)
  2. **Secondary location** (menu/dropdown, e.g., ellipsis menu)
  3. **Tertiary location** (command palette only, e.g., Ctrl+K)
  - Remove from all locations below secondary

### Pattern 5: Cross-Section Bridging
- Section A needs action from Section B → **Add bridge entry in A's context**, not a full duplicate
- Example: Project list needs "add project" → Single "Add Project" button that opens a unified dialog (with template selection as a step)

## Analysis Checklist

When analyzing a view for simplification:

- [ ] List every action entry point (buttons, menus, right-click, empty state, command palette)
- [ ] Identify actions with 3+ entry points → apply Pattern 4
- [ ] Identify overlapping menus (right-click vs ellipsis) → apply Pattern 1
- [ ] Identify empty state duplicating toolbar → apply Pattern 2
- [ ] Identify standalone sections that should be flow steps → apply Pattern 3
- [ ] Identify missing cross-section bridges → apply Pattern 5
- [ ] Verify: after changes, every remaining action is still discoverable within 2 clicks

## Implementation Notes

- Always preserve keyboard shortcuts (Ctrl+K command palette) as tertiary entry — they serve power users
- Always preserve accessibility — removing a visual entry point must not remove the only accessible path
- When merging flows, ensure the merged flow is NOT longer than the sum of the separate flows
- Test: after simplification, a new user should complete the primary task with fewer decisions
