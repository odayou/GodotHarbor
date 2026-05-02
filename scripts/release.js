const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const newVersion = process.argv[2];

if (!newVersion) {
  console.error('Usage: npm run release <version>');
  console.error('Example: npm run release 0.2.0');
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+/.test(newVersion)) {
  console.error('Version must be in semver format (e.g., 0.2.0)');
  process.exit(1);
}

const rootDir = path.resolve(__dirname, '..');
const tag = `v${newVersion}`;

const files = [
  { path: path.join(rootDir, 'package.json'), pattern: /"version":\s*"[^"]*"/, replacement: `"version": "${newVersion}"` },
  { path: path.join(rootDir, 'src-tauri', 'tauri.conf.json'), pattern: /"version":\s*"[^"]*"/, replacement: `"version": "${newVersion}"` },
  { path: path.join(rootDir, 'src-tauri', 'Cargo.toml'), pattern: /^version\s*=\s*"[^"]*"/m, replacement: `version = "${newVersion}"` }
];

for (const file of files) {
  const content = fs.readFileSync(file.path, 'utf8');
  if (!file.pattern.test(content)) {
    console.error(`Could not find version pattern in ${file.path}`);
    process.exit(1);
  }
  fs.writeFileSync(file.path, content.replace(file.pattern, file.replacement));
  console.log(`  ${path.relative(rootDir, file.path)} → ${newVersion}`);
}

execSync('git add -A', { stdio: 'inherit' });
execSync(`git commit -m "chore: release ${tag}"`, { stdio: 'inherit' });
execSync(`git tag ${tag}`, { stdio: 'inherit' });
execSync('git push', { stdio: 'inherit' });
execSync('git push --tags', { stdio: 'inherit' });

console.log(`\n✅ Released ${tag} — CI will build and publish automatically.`);
