const fs = require('fs');
const path = require('path');
const axios = require('axios');

const GITHUB_PAGES = 3;
const IGNORE_VERSIONS = ['0.16.0'];
const NUMBER_OF_VERSIONS_TO_SHOW = 2;

const MAX_RETRIES = 3;
const INITIAL_DELAY_MS = 1000;

async function fetchWithRetry(url, options, retries = MAX_RETRIES) {
  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      return await axios.get(url, options);
    } catch (error) {
      const isRetryable =
        error.code === 'ECONNRESET' || error.code === 'ETIMEDOUT' || (error.response && error.response.status >= 500);

      if (isRetryable && attempt < retries) {
        const delay = INITIAL_DELAY_MS * Math.pow(2, attempt - 1);
        console.log(`Request failed (attempt ${attempt}/${retries}), retrying in ${delay}ms...`);
        await new Promise((resolve) => setTimeout(resolve, delay));
      } else {
        throw error;
      }
    }
  }
}

async function main() {
  const axiosOpts = {
    params: { per_page: 100 },
    headers: {},
    timeout: 30000,
  };

  if (process.env.GITHUB_TOKEN) axiosOpts.headers = { Authorization: `token ${process.env.GITHUB_TOKEN}` };

  let stables = [];
  console.log('Retrieved versions:');

  for (let i = 0; i < GITHUB_PAGES; i++) {
    const { data } = await fetchWithRetry(
      `https://api.github.com/repos/noir-lang/noir/releases?page=${i + 1}`,
      axiosOpts,
    );

    console.log(data.map((release) => release.tag_name));
    stables.push(
      ...data
        .filter(
          (release) =>
            !release.prerelease && !release.tag_name.includes('aztec') && !release.tag_name.includes('aztec'),
        )
        .filter((release) => !IGNORE_VERSIONS.includes(release.tag_name.replace('v', '')))
        .map((release) => release.tag_name),
    );
  }

  stables = stables.slice(0, NUMBER_OF_VERSIONS_TO_SHOW);

  console.log('Filtered down to stables: ', stables);

  // Temporarily disable omission of patch versions, as it omits all 1.0.0-beta.n versions that are not the latest
  // To restore when versioning scheme upgrades from 1.0.0-beta.n to 1.x.y
  /*
  const onlyLatestPatches = [];
  const minorsSet = new Set(stables.map((el) => el.split('.')[1]));
  for (const minor of minorsSet) {
    const minorVersions = stables.filter((el) => el.split('.')[1] === minor);
    const max = minorVersions.reduce((prev, current) => {
      return prev > current ? prev : current;
    });
    onlyLatestPatches.push(max);
  }

  console.log('Only latest patches: ', onlyLatestPatches);

  fs.writeFileSync(path.resolve(__dirname, '../versions.json'), JSON.stringify(onlyLatestPatches, null, 2));
  */

  // To delete when versioning scheme upgrades from 1.0.0-beta.n to 1.x.y
  fs.writeFileSync(path.resolve(__dirname, '../versions.json'), JSON.stringify(stables, null, 2));
}

main();
