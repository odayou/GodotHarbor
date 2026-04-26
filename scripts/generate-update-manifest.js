const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

function generateUpdateManifest(version, releaseNotes, assetsDir, outputPath) {
  const files = [];

  function walkDir(dir, base = '') {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      const relativePath = base ? `${base}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        walkDir(fullPath, relativePath);
      } else {
        const content = fs.readFileSync(fullPath);
        const hash = crypto.createHash('sha256').update(content).digest('hex');
        files.push({
          path: relativePath,
          checksum: `sha256:${hash}`,
          size: content.length
        });
      }
    }
  }

  if (fs.existsSync(assetsDir)) {
    walkDir(assetsDir);
  }

  const totalSize = files.reduce((sum, f) => sum + f.size, 0);
  const allChecksums = files.map(f => f.checksum).join('');
  const checksum = crypto.createHash('sha256').update(allChecksums).digest('hex');

  const manifest = {
    latest_version: version,
    min_compatible_app_version: version,
    max_compatible_app_version: incrementMinor(version),
    release_notes: releaseNotes || '',
    pub_date: new Date().toISOString(),
    checksum: `sha256:${checksum}`,
    download_url: '',
    signature: '',
    files
  };

  const output = outputPath || 'hotfix-manifest.json';
  fs.writeFileSync(output, JSON.stringify(manifest, null, 2));
  console.log(`Manifest written to ${output}`);
  console.log(`  Version: ${version}`);
  console.log(`  Files: ${files.length}`);
  console.log(`  Total size: ${totalSize} bytes`);
}

function generateAppUpdateJson(version, releaseNotes, platformAssets) {
  const platforms = {};
  for (const [key, asset] of Object.entries(platformAssets || {})) {
    platforms[key] = {
      signature: asset.signature || '',
      url: asset.url || ''
    };
  }

  const update = {
    version,
    notes: releaseNotes || '',
    pub_date: new Date().toISOString(),
    platforms
  };

  const output = `app-update-${version}.json`;
  fs.writeFileSync(output, JSON.stringify(update, null, 2));
  console.log(`App update JSON written to ${output}`);
}

function incrementMinor(version) {
  const parts = version.split('.');
  if (parts.length >= 2) {
    parts[1] = parseInt(parts[1]) + 1;
    if (parts.length > 2) parts.length = 2;
    parts.push('0');
  }
  return parts.join('.');
}

const args = process.argv.slice(2);
const command = args[0];

if (command === 'hot-update') {
  const version = args[1] || '0.1.0';
  const assetsDir = args[2] || './dist';
  const outputPath = args[3];
  const releaseNotes = args[4] || '';
  generateUpdateManifest(version, releaseNotes, assetsDir, outputPath);
} else if (command === 'app-update') {
  const version = args[1] || '0.1.0';
  const releaseNotes = args[2] || '';
  generateAppUpdateJson(version, releaseNotes, {});
} else {
  console.log('Usage:');
  console.log('  node generate-update-manifest.js hot-update <version> <assetsDir> [outputPath] [releaseNotes]');
  console.log('  node generate-update-manifest.js app-update <version> [releaseNotes]');
}
