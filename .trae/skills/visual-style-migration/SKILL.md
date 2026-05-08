---
name: "visual-style-migration"
description: "Migrates UI visual style across an entire codebase: design token mapping, Acrylic/glass effects, dark mode adaptation, batch token replacement. Invoke when user wants to change or unify visual style, adopt a design reference, or fix inconsistent dark mode tokens."
---

# Visual Style Migration

Systematic methodology for migrating a project's visual style across all frontend files. Covers design token mapping, Acrylic/glass effects, dark mode adaptation, and batch token replacement.

## When to Use

- User provides reference screenshots/style descriptions and asks to adopt that style
- User reports inconsistent dark mode colors across the app
- User wants to unify design tokens (e.g., replace all `dark:bg-gray-800` with semantic tokens)
- User asks to add Acrylic/glass effects to sidebar, header, or other surfaces
- User wants to migrate from hardcoded colors to a design system

## Step 1: Audit Current Style System

Before any changes, collect the full picture:

1. **Read Tailwind config** (`tailwind.config.js/ts`): colors, shadows, backgroundImage, animation tokens
2. **Read CSS variables** (`style.css` or global CSS): `:root` and `.dark` variables
3. **Read theme composable** (`useTheme.ts` or similar): theme types, switching mechanism
4. **Audit all views and components**: grep for hardcoded dark mode tokens

### Audit Grep Patterns

```bash
# Find all hardcoded dark mode tokens
grep -rn "dark:bg-gray-\|dark:text-gray-\|dark:border-gray-\|dark:hover:bg-gray-" --include="*.vue" --include="*.tsx"
```

Record the count per file to estimate scope.

## Step 2: Map Reference Style to Project

Create a mapping table between the reference style's visual language and the project's domain:

| Reference Element | Project Equivalent | Implementation |
|---|---|---|
| Device context panel | Project context panel | Sidebar top section |
| Device render image | Project icon + version badge | Icon in card header |
| Info labels (capacity/battery) | Status labels | `info-label` / `info-value` classes |
| Storage bar | Coverage/progress bar | `storage-bar` / `storage-bar-fill` |
| Top navigation | Sidebar navigation | Already exists |
| Bottom status bar | StatusBar | Already exists |

### Key Visual Decisions

- **Primary color**: Keep existing or adopt reference? Usually keep — it's the brand.
- **Surface system**: Define light + dark surface hierarchy (base → layer → card → border → hover)
- **Glass/Acrylic**: Which surfaces get it? Typically: sidebar, header, modal backdrops. NOT: content cards (needs readability).
- **Border radius**: Unify — `rounded-xl` for cards, `rounded-lg` for buttons/inputs, `rounded-full` for avatars/badges
- **Shadow system**: Define 3-4 levels (sm, md, lg, xl) with consistent opacity

## Step 3: Define Design Tokens

### Tailwind Config Additions

```js
// tailwind.config.js
theme: {
  extend: {
    colors: {
      surface: {
        base: '#050508',        // Darkest background (dark mode)
        layer: '#0f1018',       // Page background (dark mode)
        card: '#1a1b28',        // Card/sidebar bg (dark mode)
        border: '#2d2d42',      // Borders (dark mode)
        hover: '#222236',       // Hover states (dark mode)
        'light-base': '#f0f4ff',  // Light mode equivalents
        'light-layer': '#f8faff',
        'light-card': '#ffffff',
        'light-border': '#dce4f0',
        'light-hover': '#e8f0fe',
      },
      content: {
        primary: '#F7FAFC',     // Main text (dark mode)
        secondary: '#CBD5E0',   // Secondary text (dark mode)
        muted: '#718096',       // Muted/placeholder text
      },
      status: {
        healthy: '#22c55e',
        warning: '#f59e0b',
        error: '#ef4444',
        info: '#3b82f6',
      },
    },
    backgroundImage: {
      'sidebar-acrylic': 'linear-gradient(180deg, rgba(255,255,255,0.82) 0%, rgba(240,244,255,0.78) 100%)',
      'header-acrylic': 'linear-gradient(90deg, rgba(255,255,255,0.85) 0%, rgba(248,250,255,0.80) 100%)',
    },
    boxShadow: {
      'acrylic': '0 8px 32px rgba(0, 0, 0, 0.06), inset 0 0 0 1px rgba(255, 255, 255, 0.15)',
      'acrylic-dark': '0 8px 32px rgba(0, 0, 0, 0.3), inset 0 0 0 1px rgba(255, 255, 255, 0.05)',
      'stat-card': '0 2px 8px rgba(37, 99, 235, 0.08), 0 1px 2px rgba(0, 0, 0, 0.04)',
      'stat-card-hover': '0 8px 24px rgba(37, 99, 235, 0.12), 0 2px 8px rgba(0, 0, 0, 0.06)',
    },
  },
},
```

### CSS Component Classes

```css
@layer components {
  .sidebar-acrylic {
    background: linear-gradient(180deg, rgba(255,255,255,0.82) 0%, rgba(240,244,255,0.78) 100%);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
  }

  .dark .sidebar-acrylic {
    background: linear-gradient(180deg, rgba(15,16,24,0.88) 0%, rgba(26,27,40,0.85) 100%);
    backdrop-filter: blur(20px) saturate(150%);
    -webkit-backdrop-filter: blur(20px) saturate(150%);
  }

  .header-acrylic {
    background: linear-gradient(90deg, rgba(255,255,255,0.85) 0%, rgba(248,250,255,0.80) 100%);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
  }

  .dark .header-acrylic {
    background: linear-gradient(90deg, rgba(15,16,24,0.90) 0%, rgba(26,27,40,0.85) 100%);
    backdrop-filter: blur(20px) saturate(150%);
    -webkit-backdrop-filter: blur(20px) saturate(150%);
  }

  .stat-card {
    @apply bg-white dark:bg-surface-card rounded-xl p-5 border border-gray-100 dark:border-surface-border cursor-pointer transition-all duration-200;
    box-shadow: 0 2px 8px rgba(37, 99, 235, 0.08), 0 1px 2px rgba(0, 0, 0, 0.04);
  }

  .stat-card:hover {
    box-shadow: 0 8px 24px rgba(37, 99, 235, 0.12), 0 2px 8px rgba(0, 0, 0, 0.06);
    transform: translateY(-1px);
  }

  .info-label {
    @apply text-xs text-gray-500 dark:text-content-muted uppercase tracking-wider font-medium;
  }

  .info-value {
    @apply text-sm text-gray-900 dark:text-content-primary font-medium;
  }

  .storage-bar {
    @apply w-full h-2 rounded-full overflow-hidden bg-gray-100 dark:bg-surface-border;
  }

  .storage-bar-fill {
    @apply h-full rounded-full transition-all duration-500;
  }
}
```

## Step 4: Batch Token Replacement

This is the core migration step. Use `sed` for bulk replacements across all Vue/TSX files.

### Phase 1: Dark Mode Background Tokens

```bash
for f in $(find . -name "*.vue" -not -path "./already-done.vue"); do
  sed -i 's/dark:bg-gray-800/dark:bg-surface-card/g' "$f"
  sed -i 's/dark:border-gray-700/dark:border-surface-border/g' "$f"
  sed -i 's/dark:bg-gray-700/dark:bg-surface-hover/g' "$f"
  sed -i 's/dark:bg-gray-600/dark:bg-surface-layer/g' "$f"
done
```

### Phase 2: Dark Mode Text Tokens

```bash
for f in $(find . -name "*.vue" -not -path "./already-done.vue"); do
  sed -i 's/dark:text-gray-100/dark:text-content-primary/g' "$f"
  sed -i 's/dark:text-gray-200/dark:text-content-primary/g' "$f"
  sed -i 's/dark:text-gray-300/dark:text-content-secondary/g' "$f"
  sed -i 's/dark:text-gray-400/dark:text-content-muted/g' "$f"
  sed -i 's/dark:text-gray-500/dark:text-content-muted/g' "$f"
  sed -i 's/dark:text-gray-600/dark:text-content-muted/g' "$f"
done
```

### Phase 3: Dark Mode Interactive Tokens

```bash
for f in $(find . -name "*.vue" -not -path "./already-done.vue"); do
  sed -i 's/dark:border-gray-600/dark:border-surface-border/g' "$f"
  sed -i 's/dark:border-gray-800/dark:border-surface-border/g' "$f"
  sed -i 's/dark:hover:bg-gray-700/dark:hover:bg-surface-hover/g' "$f"
  sed -i 's/dark:hover:bg-gray-600/dark:hover:bg-surface-layer/g' "$f"
  sed -i 's/dark:hover:bg-gray-500/dark:hover:bg-surface-hover/g' "$f"
  sed -i 's/dark:hover:bg-gray-200/dark:hover:bg-surface-hover/g' "$f"
done
```

### Phase 4: Card Pattern Unification

```bash
for f in $(find . -name "*.vue"); do
  # rounded-lg shadow → rounded-xl shadow (card consistency)
  sed -i 's/rounded-lg shadow/rounded-xl shadow/g' "$f"
  # Inline card styles → card component class
  sed -i 's/bg-white dark:bg-surface-card rounded-xl shadow p-6/card p-6/g' "$f"
  sed -i 's/bg-white dark:bg-surface-card rounded-xl shadow p-8/card p-8/g' "$f"
  sed -i 's/bg-white dark:bg-surface-card rounded-xl shadow p-5/card/g' "$f"
done
```

### Phase 5: Verify No Hardcoded Tokens Remain

```bash
grep -rn "dark:bg-gray-\|dark:text-gray-\|dark:border-gray-\|dark:hover:bg-gray-" --include="*.vue"
```

Expected: Only intentional exceptions (e.g., inverted color schemes like dark-on-light buttons).

## Step 5: Apply Acrylic/Glass Effects

### Which Surfaces Get Acrylic

| Surface | Acrylic? | Rationale |
|---------|----------|-----------|
| Sidebar | Yes | Navigation chrome, not content |
| Header | Yes | Navigation chrome |
| Content area | No | Needs full readability |
| Modal backdrop | Yes (blur only) | Already common pattern |
| Cards | No | Content needs opaque background |
| StatusBar | No | Too small, needs readability |

### Implementation Pattern

1. Define `.sidebar-acrylic` and `.header-acrylic` in CSS `@layer components`
2. Define `.dark .sidebar-acrylic` and `.dark .header-acrylic` variants
3. Replace `bg-white dark:bg-surface-card` with the acrylic class in Sidebar/Header
4. Remove `dark:bg-surface-card` override so the acrylic CSS takes effect in dark mode
5. Add `border-r border-gray-200/50 dark:border-surface-border` for edge definition

### Key Acrylic Parameters

- **Light mode**: `rgba(255,255,255,0.82)` → `rgba(240,244,255,0.78)`, blur 20px, saturate 180%
- **Dark mode**: `rgba(15,16,24,0.88)` → `rgba(26,27,40,0.85)`, blur 20px, saturate 150%
- Dark mode uses lower saturation (150% vs 180%) to avoid over-brightening
- Dark mode uses higher opacity (0.88 vs 0.82) to maintain text contrast

## Step 6: Dashboard/Home Page Redesign

When the reference style is a "device management tool" / "information dashboard":

1. **Stat cards**: Use `stat-card` class with icon containers (`w-12 h-12 rounded-xl bg-{color}-50 dark:bg-{color}-900/20`)
2. **Info labels**: Use `info-label` (uppercase, muted, tracking-wider) and `info-value` (font-medium, primary color)
3. **Status indicators**: Colored dots (`w-2 h-2 rounded-full bg-status-{healthy/warning/error}`)
4. **Storage/progress bars**: Use `storage-bar` and `storage-bar-fill`
5. **Project list items**: Larger icon containers (`w-10 h-10 rounded-lg`) with border, version badge below name

## Step 7: Verify and Validate

After all changes:

1. **TypeScript check**: `npx vue-tsc --noEmit` (or equivalent)
2. **Rust check** (if Tauri): `cargo check`
3. **Visual spot-check**: Grep for remaining hardcoded tokens
4. **Theme switching**: Verify light/dark/system/volcano all render correctly
5. **Acrylic check**: Verify sidebar/header show glass effect in light mode, dark surface in dark mode

## Token Mapping Reference

Complete mapping table for migrating from Tailwind default gray to semantic surface/content tokens:

### Background

| Old | New | Context |
|-----|-----|---------|
| `dark:bg-gray-800` | `dark:bg-surface-card` | Card/panel backgrounds |
| `dark:bg-gray-700` | `dark:bg-surface-hover` | Hover/active states |
| `dark:bg-gray-600` | `dark:bg-surface-layer` | Secondary backgrounds |
| `dark:bg-gray-900/30` | `dark:bg-surface-layer/50` | Subtle tinted backgrounds |
| `dark:bg-gray-900/50` | `dark:bg-surface-base/50` | Very subtle backgrounds |

### Border

| Old | New | Context |
|-----|-----|---------|
| `dark:border-gray-700` | `dark:border-surface-border` | Standard borders |
| `dark:border-gray-600` | `dark:border-surface-border` | Input/secondary borders |
| `dark:border-gray-800` | `dark:border-surface-border` | Divider borders |

### Text

| Old | New | Context |
|-----|-----|---------|
| `dark:text-gray-100` | `dark:text-content-primary` | Primary text |
| `dark:text-gray-200` | `dark:text-content-primary` | Primary text (alt) |
| `dark:text-gray-300` | `dark:text-content-secondary` | Secondary text |
| `dark:text-gray-400` | `dark:text-content-muted` | Muted/placeholder text |
| `dark:text-gray-500` | `dark:text-content-muted` | Muted text (alt) |
| `dark:text-gray-600` | `dark:text-content-muted` | Very muted text |

### Interactive

| Old | New | Context |
|-----|-----|---------|
| `dark:hover:bg-gray-700` | `dark:hover:bg-surface-hover` | Hover on items |
| `dark:hover:bg-gray-600` | `dark:hover:bg-surface-layer` | Hover on inputs |
| `dark:hover:bg-gray-500` | `dark:hover:bg-surface-hover` | Hover on buttons |
| `dark:focus:ring-offset-gray-800` | `dark:focus:ring-offset-surface-card` | Focus ring offset |

## Common Pitfalls

1. **Don't replace `dark:bg-gray-100` or `dark:text-gray-900`** — these are inverted color schemes (light-on-dark), not standard dark mode patterns
2. **Don't apply Acrylic to content areas** — text readability suffers on semi-transparent backgrounds
3. **Don't forget `.dark` CSS class variants** for Acrylic — without them, dark mode falls back to opaque `dark:bg-surface-card`
4. **Don't batch-replace `rounded-lg` globally** — only replace in card contexts (those with `shadow` or `p-6`), not in buttons/inputs/badges
5. **Verify after each sed batch** — run `vue-tsc --noEmit` to catch any broken class combinations
6. **Exclude already-migrated files** from sed loops to avoid double-replacement
