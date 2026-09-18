// Fails when a docs page that used to be served has no redirect pointing at its successor, and
// when a redirect in the table points at a page that no longer exists. Run after `yarn build`.

const path = require('path');

const {
  ALLOWLIST_PATH,
  DOCS_DIR,
  SNAPSHOT_PATH,
  legacyPathRedirects,
  matchRedirect,
  readAllowlist,
  readCurrentVersionPaths,
  readSnapshot,
} = require('./docs_routes');

const SNAPSHOT_FILE = path.relative(DOCS_DIR, SNAPSHOT_PATH);
const ALLOWLIST_FILE = path.relative(DOCS_DIR, ALLOWLIST_PATH);

function checkRemovedPaths(removed, currentPaths, allowlist) {
  const errors = [];

  for (const removedPath of removed) {
    if (allowlist.has(removedPath)) continue;

    const match = matchRedirect(removedPath);
    if (!match) {
      errors.push(
        `${removedPath} is no longer served and nothing redirects it.\n` +
          `    Add a rule to redirects.js, e.g. ['${removedPath}', '<new path>'], or record the\n` +
          `    path in ${ALLOWLIST_FILE} with a reason if the page has no successor.`,
      );
    } else if (!currentPaths.has(match.target)) {
      errors.push(
        `${removedPath} redirects to ${match.target}, which is not a page.\n` +
          `    The rule ['${match.from}', '${match.to}'] in redirects.js sends readers to a 404.`,
      );
    }
  }

  return errors;
}

function checkRedirectTargets(currentPaths) {
  const errors = [];

  for (const [from, to] of legacyPathRedirects) {
    if (to.includes(':splat')) {
      // A wildcard target can only be resolved against a concrete request, so check the fixed
      // part of it still names somewhere pages live.
      const prefix = to.slice(0, to.indexOf(':splat'));
      if (![...currentPaths].some((p) => p.startsWith(prefix))) {
        errors.push(
          `The rule ['${from}', '${to}'] in redirects.js targets ${prefix}, where no page lives.`,
        );
      }
    } else if (!currentPaths.has(to)) {
      errors.push(`The rule ['${from}', '${to}'] in redirects.js targets a page that does not exist.`);
    }
  }

  return errors;
}

function main() {
  const currentPaths = new Set(readCurrentVersionPaths());
  const snapshot = readSnapshot();

  if (!snapshot) {
    console.error(`${SNAPSHOT_FILE} is missing. Run \`yarn routes:snapshot\` and commit it.`);
    process.exit(1);
  }

  const removed = snapshot.filter((p) => !currentPaths.has(p));
  const added = [...currentPaths].filter((p) => !snapshot.includes(p));

  const errors = [...checkRemovedPaths(removed, currentPaths, readAllowlist()), ...checkRedirectTargets(currentPaths)];
  const deduped = [...new Set(errors)];

  for (const error of deduped) {
    console.error(`::error::${error}`);
  }

  if (removed.length === 0 && added.length === 0) {
    if (deduped.length > 0) process.exit(1);
    console.log(`${SNAPSHOT_FILE} is up to date (${snapshot.length} paths).`);
    return;
  }

  console.error(
    `\n${SNAPSHOT_FILE} is out of date: ${added.length} page(s) added, ${removed.length} removed.\n` +
      'Run `yarn build && yarn routes:snapshot` in docs/ and commit the result.',
  );
  for (const p of added) console.error(`  + ${p}`);
  for (const p of removed) console.error(`  - ${p}`);
  process.exit(1);
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
