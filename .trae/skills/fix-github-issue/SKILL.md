---
name: "fix-github-issue"
description: "Fetches open GitHub issues, analyzes and fixes them in codebase, then commits and pushes. Invoke when user asks to fix GitHub issues or handle issue backlog."
---

# Fix GitHub Issue

Fetch open issues from the project's GitHub repository, analyze each issue, implement fixes in the codebase, commit with issue references, push, and attempt to close the issues.

## When to Invoke

- User asks to fix GitHub issues
- User mentions "issue", "bug report", "backlog" in context of GitHub
- User wants to process open issues from the repository

## Procedure

### Step 1: Identify Repository

1. Run `git remote -v` to find the repository URL
2. If the remote is on Gitee or other non-GitHub platforms, ask the user for the GitHub repository URL
3. Extract owner/repo from the GitHub URL (e.g., `odayou/GodotHarbor`)

### Step 2: Fetch Open Issues

Use GitHub API to fetch all open issues:

```bash
curl -s "https://api.github.com/repos/{owner}/{repo}/issues?state=open&per_page=100"
```

If the API returns errors or the repo is private, try with a GitHub token:

```bash
curl -s -H "Authorization: token {GITHUB_TOKEN}" "https://api.github.com/repos/{owner}/{repo}/issues?state=open&per_page=100"
```

Parse the response and extract:
- `number` - Issue number
- `title` - Issue title
- `body` - Issue description
- `labels` - Issue labels (bug, enhancement, etc.)

### Step 3: Analyze Each Issue

For each issue:

1. Read the issue title and body carefully
2. Identify the affected files by searching the codebase for relevant keywords
3. Determine the root cause and the fix needed
4. Classify the fix type:
   - **i18n missing key**: Add translations to `src/locales/zh-CN.ts` and `src/locales/en.ts`
   - **UI bug**: Fix template/logic in Vue components
   - **Logic bug**: Fix business logic in stores/views
   - **Missing feature**: Implement the feature
   - **Icon/visual**: Update SVG or CSS

### Step 4: Implement Fixes

For each issue, implement the fix:

1. Locate the exact code location using Grep/Read tools
2. Apply the minimal fix using SearchReplace
3. Verify with `npx vue-tsc --noEmit` (or the project's typecheck command)
4. Run the project's lint command if available

### Step 5: Commit and Push

Commit all fixes with a message referencing the issue numbers:

```
fix: resolve GitHub issue #{number}

{Issue title}
- {Description of fix}
```

If multiple issues are fixed:

```
fix: resolve GitHub issues #{num1} #{num2}

#1: {Issue 1 title}
- {Fix description}

#2: {Issue 2 title}
- {Fix description}
```

Push to the remote repository.

### Step 6: Close Issues (Optional)

Attempt to close the fixed issues via GitHub API:

```bash
curl -s -X PATCH \
  -H "Authorization: token {GITHUB_TOKEN}" \
  -H "Accept: application/vnd.github.v3+json" \
  "https://api.github.com/repos/{owner}/{repo}/issues/{number}" \
  -d '{"state":"closed"}'
```

If no token is available or permission is denied, skip this step and inform the user to close manually.

## Key Principles

1. **Evidence-based**: Every fix must reference the specific issue number
2. **Minimal changes**: Fix only what the issue describes, don't over-engineer
3. **Verify before commit**: Always run typecheck/lint after changes
4. **One commit per batch**: Group related issue fixes in a single commit if they're small
5. **Respect i18n**: Always add both zh-CN and en translations for any missing keys
6. **No assumptions**: If an issue is unclear, search the codebase for context rather than guessing
