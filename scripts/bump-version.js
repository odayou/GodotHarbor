const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const newVersion = process.argv[2];

if (!newVersion) {
  console.error('Usage: npm run bump-version <version>');
  console.error('Example: npm run bump-version 0.2.0');
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+/.test(newVersion)) {
  console.error('Version must be in semver format (e.g., 0.2.0)');
  process.exit(1);
}

const files = [
  {
    path: path.join(rootDir, 'package.json'),
    pattern: /"version":\s*"[^"]*"/,
    replacement: `"version": "${newVersion}"`
  },
  {
    path: path.join(rootDir, 'src-tauri', 'tauri.conf.json'),
    pattern: /"version":\s*"[^"]*"/,
    replacement: `"version": "${newVersion}"`
  },
  {
    path: path.join(rootDir, 'src-tauri', 'Cargo.toml'),
    pattern: /^version\s*=\s*"[^"]*"/m,
    replacement: `version = "${newVersion}"`
  }
];

for (const file of files) {
  const content = fs.readFileSync(file.path, 'utf8');
  if (!file.pattern.test(content)) {
    console.error(`Could not find version pattern in ${file.path}`);
    process.exit(1);
  }
  const updated = content.replace(file.pattern, file.replacement);
  fs.writeFileSync(file.path, updated);
  console.log(`Updated ${path.relative(rootDir, file.path)} → ${newVersion}`);
}

console.log(`\nVersion bumped to ${newVersion}`);
console.log('Next steps:');
console.log(`  git add -A && git commit -m "chore: bump version to v${newVersion}"`);
console.log(`  git tag v${newVersion}`);
console.log('  git push && git push --tags');
