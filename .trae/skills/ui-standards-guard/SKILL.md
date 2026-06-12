---
name: "ui-standards-guard"
description: "Enforces GodotHarbor UI design system rules (JetBrains Islands style). Invoke when modifying any .vue/.css file's visual styles, adding new UI components, or reviewing UI changes."
---

# UI Standards Guard - GodotHarbor Design System

This skill enforces the GodotHarbor UI design system (based on JetBrains Islands/New UI). It MUST be followed whenever modifying visual styles in any `.vue` or `.css` file.

## Mandatory Pre-Flight Checklist

Before making ANY visual style change, you MUST:

1. **Read the current style definitions** in `src/style.css` (@layer components section) to know all available global component classes
2. **Audit the same element type across ALL views** — never fix just one file in isolation
3. **Use global component classes** — never write inline styles when a global class exists
4. **Verify consistency** after changes by searching for the old pattern across the entire `src/` directory

## Global Component Classes (MUST USE)

These classes are defined in `src/style.css` and MUST be used instead of inline styles:

### Buttons
| Class | Use For | Contains |
|-------|---------|----------|
| `btn-primary` | Primary action buttons | bg-primary-500, text-white, rounded-btn, hover/active states, shadow-xs |
| `btn-secondary` | Secondary action buttons | border, bg-white/dark:bg-surface-layer, rounded-btn, hover/active states |
| `btn-danger` | Destructive action buttons | bg-red-600, text-white, rounded-btn, hover/active states |

### Form Controls
| Class | Use For | Contains |
|-------|---------|----------|
| `input-field` | Text input fields | border, rounded-btn, focus ring, bg, text color, placeholder color |
| `select-field` | Dropdown selects | Same as input-field |
| `checkbox-field` | Checkboxes | rounded-[3px], border, focus ring, bg, checked state |
| `toggle-switch` | Toggle switches | 38x20px track, 16px thumb, ON/OFF states, focus ring |

### Filter & Navigation
| Class | Use For | Contains |
|-------|---------|----------|
| `filter-btn` | Inactive filter/tag buttons | rounded-btn, bg-gray-100, text-sm, font-medium |
| `filter-btn-active` | Active filter/tag buttons | rounded-btn, bg-primary-100, text-primary-700 |
| `tab-item` | Inactive tab | border-b-2 underline style, text-gray-500 |
| `tab-item-active` | Active tab | border-b-2 border-primary-500, text-primary-600 |

### Layout & Display
| Class | Use For | Contains |
|-------|---------|----------|
| `card` | Content cards | rounded-island, border, shadow-xs, bg-surface-card |
| `stat-card` | Clickable stat cards | Same as card + hover effects |
| `list-row` | List items | rounded-btn, hover bg, cursor-pointer |
| `toolbar` | Toolbar containers | border-b, flex, gap, padding |
| `badge` / `badge-success/warning/error/neutral` | Status badges | rounded-[4px], text-xs, font-medium |
| `info-label` | Small labels | text-[11px], uppercase, tracking-wider |
| `info-value` | Label values | text-sm, font-medium |

## Border Radius Rules (STRICT)

NEVER use bare `rounded` (4px). Always use the correct radius:

| Element Type | Radius | Tailwind Class |
|-------------|--------|---------------|
| Buttons, inputs, selects | 6px | `rounded-btn` |
| Cards, panels, islands | 10px | `rounded-island` |
| Dialogs, modals | 12px | `rounded-[12px]` |
| Badges, tags, kbd, small elements | 4px | `rounded-[4px]` |
| Sidebar nav items | 4px | `rounded-[4px]` |
| List items, icon buttons | 4px | `rounded-[4px]` |
| Alert/info containers | 6px | `rounded-[6px]` |
| Dropdown menus | 6px | `rounded-[6px]` |
| Floating panels | 10px | `rounded-island` |

## Color Rules (STRICT)

### Border Opacity
- Separator borders: `border-gray-200/60 dark:border-surface-border/40`
- Input/select borders: `border-gray-300 dark:border-surface-border` (NO opacity)
- Card borders: `border-gray-200/60 dark:border-surface-border/40`

### Background Colors
- Input/select bg: `bg-white dark:bg-surface-input` (via `input-field`/`select-field`)
- Card bg: `bg-white dark:bg-surface-card` (via `card`)
- Sidebar/panel bg: `bg-surface-light-layer dark:bg-surface-layer`
- Hover bg: `hover:bg-gray-50 dark:hover:bg-surface-hover` or `hover:bg-black/[0.04] dark:hover:bg-white/[0.06]`

### Brand Color
- Primary action: `bg-primary-500` (#3574F0) — NOT `bg-primary-600`
- Active text: `text-primary-600 dark:text-brand-primary`
- Active nav bg: `bg-primary-50/60 dark:bg-primary-500/10`

## Icon Rules (STRICT)

All SVG icons MUST use:
- `stroke-linecap="butt"` (NOT `round`)
- `stroke-linejoin="miter"` (NOT `round`)
- `stroke-width="1.5"` for 24px viewBox
- Prefer 45°/90° angles, flat geometric style

## Spacing Rules (8px Grid)

All spacing values MUST be multiples of 4px:
- `4px` (p-1, gap-1) — icon-text gap
- `8px` (p-2, gap-2) — same-group spacing
- `12px` (p-3, gap-3) — card padding
- `16px` (p-4, gap-4) — section spacing
- `24px` (p-6, gap-6) — large section spacing
- `32px` (p-8, gap-8) — page margin

NEVER use non-standard values like `p-1.5` (6px), `py-0.5` (2px), `gap-1.5` (6px).

## Animation Rules

| Type | Duration | Easing | Use For |
|------|----------|--------|---------|
| Instant | 100ms | ease | Button clicks, color changes |
| Fast | 150ms | ease | Hover, focus |
| Standard | 200ms | ease-in-out | Expand/collapse, tab switch |
| Slow | 300ms | ease-in-out | Page transitions, modals |

## Mandatory Audit Process

When modifying UI styles, follow this EXACT process:

### Step 1: Identify Scope
Search for ALL instances of the same element type across the entire `src/` directory before making any change.

### Step 2: Use Global Classes
Replace inline styles with the appropriate global component class from `src/style.css`.

### Step 3: If No Global Class Exists
If no suitable global class exists, FIRST add one to `src/style.css` @layer components, THEN use it everywhere.

### Step 4: Verify Completeness
After changes, search for the OLD pattern to confirm zero remaining violations:
```
grep -r "old_pattern" src/ --include="*.vue"
```

### Step 5: Build Check
Run `npx vue-tsc --noEmit && npx vite build` to verify no build errors.

## Common Violations to Avoid

1. **Inline button styles** — Use `btn-primary`/`btn-secondary`/`btn-danger` instead
2. **Inline input styles** — Use `input-field` instead
3. **Inline select styles** — Use `select-field` instead
4. **Bare `rounded`** — Always specify the correct radius class
5. **`bg-primary-600` for buttons** — Use `btn-primary` (which uses `bg-primary-500`)
6. **Missing border opacity** — Separators need `/60` and `/40`
7. **`stroke-linecap="round"`** — Must be `"butt"`
8. **`stroke-linejoin="round"`** — Must be `"miter"`
9. **Non-standard spacing** — Must follow 4px grid
10. **Inconsistent checkbox styles** — Use `checkbox-field`
11. **shadow-xs on non-card elements** — Only cards and primary buttons get shadow-xs
12. **Mixed active states** — Sidebar nav uses `bg-primary-50/60`, not `border-l-2` or `shadow-xs`

## Reference Files

- Design spec: `docs/design/UI风格指南.md`
- Global component classes: `src/style.css` (@layer components)
- Tailwind config: `tailwind.config.js`
- Theme composable: `src/composables/useTheme.ts`
