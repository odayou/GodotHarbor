const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

function generateHotUpdateManifest(options) {
  const { version, zipPath, downloadUrl, minVersion, maxVersion, releaseNotes, outputPath } = options;

  const zipBuffer = fs.readFileSync(zipPath);
  const checksum = crypto.createHash('sha256').update(zipBuffer).digest('hex');
  const downloadSize = zipBuffer.length;

  const manifest = {
    version,
    min_compatible_app_version: minVersion || version,
    max_compatible_app_version: maxVersion || incrementMinor(version),
    release_notes: releaseNotes || '',
    pub_date: new Date().toISOString(),
    download_url: downloadUrl || '',
    download_size: downloadSize,
    checksum,
    files: []
  };

  const output = outputPath || 'hotupdate-manifest.json';
  fs.writeFileSync(output, JSON.stringify(manifest, null, 2));
  console.log(`Hot update manifest written to ${output}`);
  console.log(`  Version: ${version}`);
  console.log(`  Compatible: ${manifest.min_compatible_app_version} ~ ${manifest.max_compatible_app_version}`);
  console.log(`  Download size: ${downloadSize} bytes`);
  console.log(`  Checksum: ${checksum}`);
}

function generateHotUpdateManifestFromDir(options) {
  const { version, assetsDir, downloadUrl, minVersion, maxVersion, releaseNotes, outputPath } = options;
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
          checksum: hash,
          size: content.length
        });
      }
    }
  }

  if (fs.existsSync(assetsDir)) {
    walkDir(assetsDir);
  }

  const totalSize = files.reduce((sum, f) => sum + f.size, 0);

  const manifest = {
    version,
    min_compatible_app_version: minVersion || version,
    max_compatible_app_version: maxVersion || incrementMinor(version),
    release_notes: releaseNotes || '',
    pub_date: new Date().toISOString(),
    download_url: downloadUrl || '',
    download_size: totalSize,
    checksum: '',
    files
  };

  const output = outputPath || 'hotupdate-manifest.json';
  fs.writeFileSync(output, JSON.stringify(manifest, null, 2));
  console.log(`Hot update manifest written to ${output}`);
  console.log(`  Version: ${version}`);
  console.log(`  Files: ${files.length}`);
  console.log(`  Total size: ${totalSize} bytes`);
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

if (command === 'from-zip') {
  generateHotUpdateManifest({
    version: args[1] || '0.1.0',
    zipPath: args[2] || 'hotupdate.zip',
    downloadUrl: args[3] || '',
    minVersion: args[4] || '',
    maxVersion: args[5] || '',
    releaseNotes: args[6] || '',
    outputPath: args[7] || 'hotupdate-manifest.json'
  });
} else if (command === 'from-dir') {
  generateHotUpdateManifestFromDir({
    version: args[1] || '0.1.0',
    assetsDir: args[2] || './dist',
    downloadUrl: args[3] || '',
    minVersion: args[4] || '',
    maxVersion: args[5] || '',
    releaseNotes: args[6] || '',
    outputPath: args[7] || 'hotupdate-manifest.json'
  });
} else {
  console.log('Usage:');
  console.log('  node generate-update-manifest.js from-zip <version> <zipPath> [downloadUrl] [minVersion] [maxVersion] [releaseNotes] [outputPath]');
  console.log('  node generate-update-manifest.js from-dir <version> <assetsDir> [downloadUrl] [minVersion] [maxVersion] [releaseNotes] [outputPath]');
}
