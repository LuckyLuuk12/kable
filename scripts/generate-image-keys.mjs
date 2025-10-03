#!/usr/bin/env node
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

// Resolve script location reliably across platforms (Windows path handling)
const __filename = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(__filename), '..');
const REPO_GITHUB_BASE = 'https://github.com/LuckyLuuk12/kable/blob/main';
const SEARCH_EXTS = ['.svelte', '.ts', '.js', '.mjs', '.tsx', '.jsx', '.html'];

async function walk(dir) {
  const entries = await fs.readdir(dir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'node_modules' || entry.name === 'build' || entry.name === '.git' || entry.name === 'target') continue;
      files.push(...await walk(p));
    } else if (SEARCH_EXTS.includes(path.extname(entry.name))) {
      files.push(p);
    }
  }
  return files;
}

function extractKeysFromTextByLine(text, filePath) {
  const imageKeys = new Map(); // key -> [{line, snippet}]
  const iconKeys = new Map();

  const imgRegex = /<Image\s[^>]*key\s*=\s*(?:\{\s*['\"]([^'\"]+)['\"]\s*\}|['\"]([^'\"]+)['\"])/;
  const iconRegex = /<Icon\s[^>]*name\s*=\s*(?:\{\s*['\"]([^'\"]+)['\"]\s*\}|['\"]([^'\"]+)['\"])/;
  const imgPathRegex = /\/img\/([a-zA-Z0-9_\-]+)\.(?:webp|png|jpg|jpeg|svg|gif)/;

  const lines = text.split(/\r?\n/);
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    let m;
    if ((m = imgRegex.exec(line))) {
      const key = m[1] || m[2];
      if (key) {
        const arr = imageKeys.get(key) || [];
        arr.push({ file: path.relative(ROOT, filePath).replace(/\\/g, '/'), line: i + 1, snippet: line.trim() });
        imageKeys.set(key, arr);
      }
    }
    if ((m = iconRegex.exec(line))) {
      const key = m[1] || m[2];
      if (key) {
        const arr = iconKeys.get(key) || [];
        arr.push({ file: path.relative(ROOT, filePath).replace(/\\/g, '/'), line: i + 1, snippet: line.trim() });
        iconKeys.set(key, arr);
      }
    }
    if ((m = imgPathRegex.exec(line))) {
      const key = m[1];
      if (key) {
        const arr = imageKeys.get(key) || [];
        arr.push({ file: path.relative(ROOT, filePath).replace(/\\/g, '/'), line: i + 1, snippet: line.trim() });
        imageKeys.set(key, arr);
      }
    }
  }

  return { imageKeys, iconKeys };
}

async function main() {
  const files = await walk(ROOT);
  const allImageKeys = new Set();
  const allIconKeys = new Set();

  // Gather occurrences with file+line information
  const imageOccurrences = new Map();
  const iconOccurrences = new Map();

  for (const file of files) {
    try {
      const txt = await fs.readFile(file, 'utf8');
      const { imageKeys, iconKeys } = extractKeysFromTextByLine(txt, file);
      for (const [k, arr] of imageKeys.entries()) {
        const existing = imageOccurrences.get(k) || [];
        imageOccurrences.set(k, existing.concat(arr));
      }
      for (const [k, arr] of iconKeys.entries()) {
        const existing = iconOccurrences.get(k) || [];
        iconOccurrences.set(k, existing.concat(arr));
      }
    } catch (e) {
      // ignore unreadable files
    }
  }

  const outDir = path.join(ROOT, 'build', 'generated');
  await fs.mkdir(outDir, { recursive: true });

  const result = {
    imageKeys: Array.from(imageOccurrences.keys()).sort(),
    iconKeys: Array.from(iconOccurrences.keys()).sort(),
    occurrences: {
      images: Object.fromEntries(Array.from(imageOccurrences.entries())),
      icons: Object.fromEntries(Array.from(iconOccurrences.entries()))
    },
    generatedAt: new Date().toISOString()
  };

  const outPath = path.join(outDir, 'keys.json');
  await fs.writeFile(outPath, JSON.stringify(result, null, 2), 'utf8');

  // Also emit a human-friendly markdown in the project root
  const mdPath = path.join(ROOT, 'IMAGE_KEYS.md');
  let md = '# IMAGE_KEYS\n\n';
  md += 'This file is generated automatically. It lists image and icon keys used across the codebase and points to example locations in the source.\n\n';
  md += 'If you want to override a key with a custom image, place a file named `<key>.<ext>` in the launcher config images directory (`<kable_launcher_dir>/config/images/`) or add a static asset under `static/img/`.\n\n';

  md += '## Image keys\n\n';
  md += '| Key | Source | Snippet |\n';
  md += '| --- | --- | --- |\n';
  // shorten displayed source links by removing common source prefixes
  const STRIP_PREFIXES = ['src/lib/components/', 'src/lib/', 'src/routes/'];
  for (const key of result.imageKeys) {
    const occ = imageOccurrences.get(key) || [];
    if (occ.length === 0) {
      md += `| ${key} | - | - |\n`;
      continue;
    }
    for (const o of occ) {
      const url = `${REPO_GITHUB_BASE}/${o.file}#L${o.line}`;
      let display = o.file;
      for (const p of STRIP_PREFIXES) {
        if (display.startsWith(p)) {
          display = display.slice(p.length);
          break;
        }
      }
      // If this came from a routes +page file, show a friendly "<PageName> Page"
      if (o.file.startsWith('src/routes/')) {
        const rem = o.file.slice('src/routes/'.length); // e.g. "logs/+page.svelte" or "+page.svelte"
        let pageName = 'Home';
        if (rem.startsWith('+page')) {
          pageName = 'Home';
        } else if (rem.includes('/+page.')) {
          const parts = rem.split('/+page.');
          pageName = parts[0].split('/').pop() || 'Home';
        }
        // Capitalize first letter
        pageName = pageName.charAt(0).toUpperCase() + pageName.slice(1);
        display = `${pageName} Page`;
      } else {
        // show only the filename without extension for compactness
        display = path.basename(display, path.extname(display));
      }
      const sourceLink = `[${display}](${url})`;
  const snippet = o.snippet.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/\|/g, '&#124;');
  md += `| ${key} | ${sourceLink} | <pre><code class="language-html">${snippet}</code></pre> |\n`;
    }
  }

  md += '\n## Icon keys\n\n';
  md += '| Key | Source | Snippet |\n';
  md += '| --- | --- | --- |\n';
  for (const key of result.iconKeys) {
    const occ = iconOccurrences.get(key) || [];
    if (occ.length === 0) {
      md += `| ${key} | - | - |\n`;
      continue;
    }
    for (const o of occ) {
      const url = `${REPO_GITHUB_BASE}/${o.file}#L${o.line}`;
      let display = o.file;
      for (const p of STRIP_PREFIXES) {
        if (display.startsWith(p)) {
          display = display.slice(p.length);
          break;
        }
      }
      // If this came from a routes +page file, show a friendly "<PageName> Page"
      if (o.file.startsWith('src/routes/')) {
        const rem = o.file.slice('src/routes/'.length);
        let pageName = 'Home';
        if (rem.startsWith('+page')) {
          pageName = 'Home';
        } else if (rem.includes('/+page.')) {
          const parts = rem.split('/+page.');
          pageName = parts[0].split('/').pop() || 'Home';
        }
        pageName = pageName.charAt(0).toUpperCase() + pageName.slice(1);
        display = `${pageName} Page`;
      } else {
        display = path.basename(display, path.extname(display));
      }
      const sourceLink = `[${display}](${url})`;
  const snippet = o.snippet.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/\|/g, '&#124;');
  md += `| ${key} | ${sourceLink} | <pre><code class="language-html">${snippet}</code></pre> |\n`;
    }
  }

  md += `\n_Generated: ${result.generatedAt}_\n`;

  await fs.writeFile(mdPath, md, 'utf8');
  console.log(`Generated keys: ${outPath} and ${mdPath} (imageKeys=${result.imageKeys.length}, iconKeys=${result.iconKeys.length})`);
}

main().catch(err => { console.error(err); process.exit(1); });
