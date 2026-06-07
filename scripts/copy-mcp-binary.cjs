const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const rootDir = path.resolve(__dirname, '..');
const srcTauriDir = path.join(rootDir, 'src-tauri');
const binariesDir = path.join(srcTauriDir, 'binaries');

const isDev = process.argv.includes('--dev');
const profile = isDev ? 'debug' : 'release';

function getTargetTriple() {
  try {
    const output = execSync('rustc -vV', { encoding: 'utf8' });
    const match = output.match(/host:\s*(\S+)/);
    if (match) return match[1];
  } catch {}
  if (process.platform === 'win32') return 'x86_64-pc-windows-msvc';
  if (process.platform === 'darwin') return 'aarch64-apple-darwin';
  return 'x86_64-unknown-linux-gnu';
}

const targetTriple = getTargetTriple();
const exeName = process.platform === 'win32' ? 'harbor-mcp-server.exe' : 'harbor-mcp-server';
const sourcePath = path.join(srcTauriDir, 'target', profile, exeName);
const destName = `harbor-mcp-server-${targetTriple}${process.platform === 'win32' ? '.exe' : ''}`;
const destPath = path.join(binariesDir, destName);

console.log(`Building harbor-mcp-server (${profile})...`);
execSync(`cargo build --profile ${profile === 'debug' ? 'dev' : 'release'} --bin harbor-mcp-server --manifest-path "${path.join(srcTauriDir, 'Cargo.toml')}"`, {
  stdio: 'inherit',
  cwd: rootDir
});

if (!fs.existsSync(sourcePath)) {
  console.error(`Error: Built binary not found at ${sourcePath}`);
  process.exit(1);
}

if (!fs.existsSync(binariesDir)) {
  fs.mkdirSync(binariesDir, { recursive: true });
}

fs.copyFileSync(sourcePath, destPath);
console.log(`Copied: ${sourcePath} -> ${destPath}`);
