const fs = require('fs');

function dedupeFile(filepath) {
  let content = fs.readFileSync(filepath, 'utf8');
  let lines = content.split('\n');
  console.log('\nProcessing: ' + filepath + ' (' + lines.length + ' lines)');

  const sectionsToDedupe = ['linker', 'plugins', 'engines', 'settings', 'common'];

  function findSectionBounds(startFrom) {
    const occ = {};
    for (let i = startFrom; i < lines.length; i++) {
      const line = lines[i];
      const m = /^(\s+)(\w+):\s*\{/.exec(line);
      if (m && sectionsToDedupe.includes(m[2])) {
        let braceCount = 0;
        let j = i;
        do {
          if (lines[j].includes('{')) braceCount += (lines[j].match(/{/g) || []).length;
          if (lines[j].includes('}')) braceCount -= (lines[j].match(/}/g) || []).length;
          j++;
        } while (braceCount !== 0 && j < lines.length);
        const name = m[2];
        if (!(name in occ)) {
          occ[name] = { start: i, end: j - 1, indent: m[1] };
        } else {
          occ[name + '_2'] = { start: i, end: j - 1, indent: m[1] };
        }
      }
    }
    return occ;
  }

  let occ = findSectionBounds(0);
  console.log('Step 0 - found sections:', Object.keys(occ).join(', '));

  const first = {};
  const second = {};
  for (const k of Object.keys(occ)) {
    if (k.endsWith('_2')) {
      second[k.replace('_2', '')] = occ[k];
    } else {
      first[k] = occ[k];
    }
  }

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
        if (currentKey !== null) currentValueLines.push(line);
      }
      if (line.includes('{')) braceCount += (line.match(/{/g) || []).length;
      if (line.includes('}')) braceCount -= (line.match(/}/g) || []).length;
      i++;
    }
    if (currentKey !== null) result[currentKey] = currentValueLines.join('\n').trim();
    return result;
  }

  // Find unique keys from first sections
  const uniqueToAdd = {};
  for (const name of sectionsToDedupe) {
    if (first[name] && second[name]) {
      const firstParsed = parseSection(first[name].start, first[name].end);
      const secondParsed = parseSection(second[name].start, second[name].end);
      uniqueToAdd[name] = {};
      for (const k of Object.keys(firstParsed)) {
        if (!Object.prototype.hasOwnProperty.call(secondParsed, k)) {
          uniqueToAdd[name][k] = firstParsed[k];
        }
      }
      console.log(name + ' unique keys: ' + Object.keys(uniqueToAdd[name]).join(', '));
    }
  }

  // Remove first occurrences (in reverse order)
  const removalList = Object.values(first).sort(function(a, b) { return b.start - a.start; });
  for (const r of removalList) {
    lines.splice(r.start, r.end - r.start + 1);
    console.log('Removed first ' + Object.keys(first).find(function(k) { return first[k] === r; }) + ' at ' + r.start + '-' + r.end);
  }

  // Now find second occurrences in the modified lines
  const newSecond = findSectionBounds(0);
  console.log('After removal, found:', Object.keys(newSecond).join(', '));

  // For each second occurrence, insert unique keys
  // Process in reverse order by start line
  const insertList = Object.entries(newSecond)
    .filter(function(e) { return uniqueToAdd[e[0]] && Object.keys(uniqueToAdd[e[0]]).length > 0; })
    .sort(function(a, b) { return b[1].start - a[1].start; });

  for (const entry of insertList) {
    const name = entry[0];
    const info = entry[1];
    const start = info.start;
    const end = info.end;
    const indent = info.indent;

    // Find where to insert: just before the closing brace
    // The closing brace is at 'end'. Find last content line before it.
    let insertAfter = start;
    for (let i = end - 1; i >= start; i--) {
      if (!lines[i]) { console.log('UNDEFINED at i=' + i + ' (end=' + end + ')'); continue; }
      const trimmed = lines[i].trim();
      if (trimmed.length > 0 && trimmed !== '},' && trimmed !== '}') {
        insertAfter = i;
        break;
      }
    }

    // Make sure last content line ends with comma
    if (insertAfter > start) {
      const lastLine = lines[insertAfter];
      if (lastLine && !lastLine.trim().endsWith(',')) {
        lines[insertAfter] = lastLine + ',';
      }
    }

    // Build and insert new lines
    const keys = Object.keys(uniqueToAdd[name]);
    for (let i = 0; i < keys.length; i++) {
      const k = keys[i];
      const v = uniqueToAdd[name][k];
      lines.splice(insertAfter + 1 + i, 0, indent + '    ' + k + ': ' + v + ',');
    }
    console.log('Inserted ' + keys.length + ' keys into ' + name + ' at pos ' + insertAfter);
  }

  fs.writeFileSync(filepath, lines.join('\n'), 'utf8');
  console.log('Written: ' + filepath + ' (' + lines.length + ' lines)');
}

dedupeFile('src/locales/zh-CN.ts');
dedupeFile('src/locales/en.ts');