const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

function getLatestTag() {
  try {
    return execSync('git describe --tags --abbrev=0', { encoding: 'utf8' }).trim();
  } catch {
    return null;
  }
}

function parseSemver(tag) {
  const version = tag.replace(/^v/, '');
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/);
  if (!match) return null;
  return { major: parseInt(match[1]), minor: parseInt(match[2]), patch: parseInt(match[3]) };
}

function bumpVersion(semver, type) {
  if (type === 'major') return `${semver.major + 1}.0.0`;
  if (type === 'minor') return `${semver.major}.${semver.minor + 1}.0`;
  return `${semver.major}.${semver.minor}.${semver.patch + 1}`;
}

const input = process.argv[2];

if (!input) {
  const latestTag = getLatestTag();
  if (latestTag) {
    const semver = parseSemver(latestTag);
    if (semver) {
      console.log(`Current version: ${latestTag}`);
      console.log(`Usage:`);
      console.log(`  npm run release patch     → v${bumpVersion(semver, 'patch')}`);
      console.log(`  npm run release minor     → v${bumpVersion(semver, 'minor')}`);
      console.log(`  npm run release major     → v${bumpVersion(semver, 'major')}`);
      console.log(`  npm run release 0.2.0     → v0.2.0 (explicit)`);
    }
  } else {
    console.error('Usage: npm run release <version | patch | minor | major>');
    console.error('Example: npm run release patch');
    console.error('         npm run release 0.2.0');
  }
  process.exit(1);
}

let newVersion;

if (['patch', 'minor', 'major'].includes(input)) {
  const latestTag = getLatestTag();
  if (!latestTag) {
    console.error('No existing tags found. Please specify an explicit version for the first release.');
    console.error('Example: npm run release 0.1.0');
    process.exit(1);
  }
  const semver = parseSemver(latestTag);
  if (!semver) {
    console.error(`Cannot parse tag "${latestTag}" as semver. Please specify an explicit version.`);
    process.exit(1);
  }
  newVersion = bumpVersion(semver, input);
  console.log(`Bumping ${input}: ${latestTag} → v${newVersion}`);
} else {
  if (!/^\d+\.\d+\.\d+/.test(input)) {
    console.error(`Invalid version "${input}". Use patch/minor/major or explicit version (e.g., 0.2.0).`);
    process.exit(1);
  }
  newVersion = input;
}

const rootDir = path.resolve(__dirname, '..');
const tag = `v${newVersion}`;

const existingTag = getLatestTag();
if (existingTag === tag) {
  console.error(`Tag ${tag} already exists. Use a different version.`);
  process.exit(1);
}

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
