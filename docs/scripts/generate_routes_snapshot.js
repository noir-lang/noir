// Rewrites routes.snapshot.json from the last Docusaurus build. Run `yarn build` first.

const path = require('path');

const { DOCS_DIR, SNAPSHOT_PATH, readCurrentVersionPaths, writeSnapshot } = require('./docs_routes');

function main() {
  const paths = readCurrentVersionPaths();
  writeSnapshot(paths);
  console.log(`Wrote ${paths.length} paths to ${path.relative(DOCS_DIR, SNAPSHOT_PATH)}.`);
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
