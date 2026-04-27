const fs = require('fs');
const path = require('path');

const docsRoot = path.resolve(__dirname, '..');
const authoredDocsDir = path.join(docsRoot, 'docs');
const outputPath = path.join(docsRoot, 'static', 'docs-manifest.json');

function walkMarkdownFiles(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...walkMarkdownFiles(fullPath));
    } else if (/\.mdx?$/.test(entry.name)) {
      files.push(fullPath);
    }
  }

  return files;
}

function splitFrontmatter(content) {
  if (!content.startsWith('---')) {
    return { frontmatter: '', body: content };
  }

  const end = content.indexOf('\n---', 3);
  if (end === -1) {
    return { frontmatter: '', body: content };
  }

  return {
    frontmatter: content.slice(3, end).trim(),
    body: content.slice(end + 4),
  };
}

function frontmatterValue(frontmatter, key) {
  const match = frontmatter.match(new RegExp(`^${key}:\\s*(.*)$`, 'm'));
  if (!match) {
    return null;
  }

  const value = match[1].trim();
  if (value) {
    return value.replace(/^['"]|['"]$/g, '');
  }

  const lines = frontmatter.split('\n');
  const start = lines.findIndex((line) => line.startsWith(`${key}:`));
  if (start === -1) {
    return null;
  }

  const collected = [];
  for (const line of lines.slice(start + 1)) {
    if (/^[A-Za-z0-9_-]+:\s*/.test(line)) {
      break;
    }
    collected.push(line.trim());
  }

  return collected.join(' ').trim() || null;
}

function stripCodeBlocks(content) {
  return content.replace(/```[\s\S]*?```/g, '');
}

function routeFromRelativePath(relativePath) {
  const withoutExt = relativePath.replace(/\.(md|mdx)$/, '');
  if (withoutExt === 'index') {
    return '/';
  }
  if (withoutExt.endsWith('/index')) {
    return `/${withoutExt.slice(0, -'/index'.length)}`;
  }
  return `/${withoutExt}`;
}

function collectLinks(markdown) {
  return [...markdown.matchAll(/\[[^\]]+\]\(([^)]+)\)/g)]
    .map((match) => match[1])
    .filter((href) => !href.startsWith('#'))
    .sort();
}

function pageRecord(filePath) {
  const relativePath = path.relative(authoredDocsDir, filePath).split(path.sep).join('/');
  const raw = fs.readFileSync(filePath, 'utf8');
  const { frontmatter, body } = splitFrontmatter(raw);
  const bodyWithoutCode = stripCodeBlocks(body);
  const links = collectLinks(bodyWithoutCode);

  return {
    sourcePath: `docs/docs/${relativePath}`,
    routePath: routeFromRelativePath(relativePath),
    section: relativePath.split('/')[0],
    title: frontmatterValue(frontmatter, 'title'),
    description: frontmatterValue(frontmatter, 'description'),
    headings: [...bodyWithoutCode.matchAll(/^(#{1,6})\s+(.+)$/gm)].map((match) => ({
      level: match[1].length,
      text: match[2].trim(),
    })),
    links: {
      internal: links.filter((href) => !/^https?:\/\//.test(href)),
      external: links.filter((href) => /^https?:\/\//.test(href)),
    },
  };
}

function run() {
  const pages = walkMarkdownFiles(authoredDocsDir)
    .sort()
    .map(pageRecord);

  const manifest = {
    schemaVersion: 1,
    source: 'docs/docs',
    pages,
  };

  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`Wrote ${pages.length} docs records to ${path.relative(docsRoot, outputPath)}`);
}

run();
