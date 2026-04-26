const fs = require('fs');

const zhPath = 'src/locales/zh-CN.ts';
const enPath = 'src/locales/en.ts';

function dedupeLocaleFile(filepath) {
  const content = fs.readFileSync(filepath, 'utf8');
  const lines = content.split('\n');

  const sectionsToDedupe = ['linker', 'plugins', 'engines', 'settings', 'common'];

  const firstOcc = {};
  const secondOcc = {};

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const regex = /^(\s+)(\w+):\s*\{/;
    const m = regex.exec(line);
    if (m) {
      const indent = m[1];
      const name = m[2];
      if (sectionsToDedupe.includes(name)) {
        let braceCount = 0;
        let j = i;
        do {
          if (lines[j].includes('{')) braceCount += (lines[j].match(/{/g) || []).length;
          if (lines[j].includes('}')) braceCount -= (lines[j].match(/}/g) || []).length;
          j++;
        } while (braceCount !== 0 && j < lines.length);
        const end = j - 1;

        if (!(name in firstOcc)) {
          firstOcc[name] = { start: i, end: end, indent };
        } else {
          secondOcc[name] = { start: i, end: end, indent };
        }
      }
    }
  }

  console.log('First occurrences:', Object.entries(firstOcc).map(function(e) { return e[0] + ': ' + e[1].start + '-' + e[1].end; }));
  console.log('Second occurrences:', Object.entries(secondOcc).map(function(e) { return e[0] + ': ' + e[1].start + '-' + e[1].end; }));

  function parseSection(start, end) {
    const result = {};
    let i = start + 1;
    let currentKey = null;
    let currentValueLines = [];
    let braceCount = 1;
    const keyRegex = /^(\s+)(\w+):\s*(.*)$/;

    while (i < end) {
      const line = lines[i];
      const m = keyRegex.exec(line);
      if (m && braceCount === 1) {
        if (currentKey !== null) {
          result[currentKey] = currentValueLines.join('\n').trim();
        }
        currentKey = m[2];
        currentValueLines = [m[3].trim()];
      } else {
        if (currentKey !== null) {
          currentValueLines.push(line);
        }
      }
      if (line.includes('{')) braceCount += (line.match(/{/g) || []).length;
      if (line.includes('}')) braceCount -= (line.match(/}/g) || []).length;
      i++;
    }
    if (currentKey !== null) {
      result[currentKey] = currentValueLines.join('\n').trim();
    }
    return result;
  }

  const uniqueKeys = {};
  for (const name of sectionsToDedupe) {
    if (firstOcc[name] && secondOcc[name]) {
      const first = parseSection(firstOcc[name].start, firstOcc[name].end);
      const second = parseSection(secondOcc[name].start, secondOcc[name].end);
      const unique = Object.keys(first).filter(function(k) { return !Object.prototype.hasOwnProperty.call(second, k); });
      uniqueKeys[name] = unique;
      console.log('\n' + name + ' unique from first (' + unique.length + '): ' + unique.join(', '));
      console.log('  overlap keys: ' + Object.keys(first).filter(function(k) { return Object.prototype.hasOwnProperty.call(second, k); }).join(', '));
    }
  }
  return { firstOcc, secondOcc, uniqueKeys };
}

dedupeLocaleFile(zhPath);