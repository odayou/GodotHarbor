const fs = require('fs');
const path = require('path');

const { execSync } = require('child_process');
const original = execSync('git show HEAD:src-tauri/src/commands/mod.rs', { cwd: path.resolve(__dirname, '..', '..', '..'), encoding: 'utf8' });
const lines = original.split('\n');

function getLineRange(start1, end1) {
  return lines.slice(start1 - 1, end1).join('\n');
}

const header = `use std::path::{PathBuf, Path};
use std::fs;
use std::io::Write;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager, Emitter, State};
use crate::models::*;
use crate::storage::Storage;
use crate::scanner::ProjectScanner;
use crate::plugin_manager::PluginManager;
use crate::linker::Linker;
use crate::operation_log::{OperationLogger, LogEntry};
use crate::AppState;
use uuid::Uuid;
use futures::future::join_all;
use crate::utils::{copy_dir_all, create_http_client, no_window_cmd};
use super::utils::*;
`;

const utilsHeader = `use std::path::{PathBuf, Path};
use std::fs;
use std::io::Write;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager, Emitter};
use crate::models::*;
use crate::storage::Storage;
use crate::plugin_manager::PluginManager;
use crate::linker::Linker;
use crate::operation_log::{OperationLogger, LogEntry};
use crate::utils::{copy_dir_all, create_http_client, no_window_cmd};
`;

const outDir = path.join(__dirname);

// utils.rs: lines 18-89 (utility functions) + 114-136 (constants + log helpers) + 138-195 (backup helpers) + 197-203 (AddonBackupInfo) + 445-494 (detached_cmd + AutoSetupState + compute_settings_hash)
const utilsCode = utilsHeader + '\n' +
  getLineRange(18, 89) + '\n\n' +
  getLineRange(114, 203) + '\n\n' +
  getLineRange(445, 494) + '\n';

fs.writeFileSync(path.join(outDir, 'utils.rs'), utilsCode);
console.log('utils.rs created');

const modules = {
  backup: [[205, 431], [2853, 3045]],
  settings: [[432, 460], [496, 529], [539, 674]],
  project: [[1016, 1255], [2820, 2852], [3831, 4029], [4062, 4098]],
  plugin: [[1256, 1838], [1839, 2043], [2126, 2166], [2564, 2637], [2800, 2819], [3154, 3363], [4099, 4341]],
  engine: [[461, 495], [675, 1015], [3046, 3153], [4342, 4459]],
  update: [[2167, 2563], [2638, 2786]],
  asset: [[3364, 3799]],
  system: [[91, 204], [1510, 1528], [2044, 2125], [2787, 2799], [3800, 3831], [4460, 4493]]
};

for (const [name, ranges] of Object.entries(modules)) {
  let code = header + '\n';
  for (const [s, e] of ranges) {
    code += getLineRange(s, e) + '\n\n';
  }
  fs.writeFileSync(path.join(outDir, name + '.rs'), code);
  console.log(name + '.rs created, lines: ' + ranges.map(r => r[0] + '-' + r[1]).join(', '));
}

console.log('All modules created successfully');
