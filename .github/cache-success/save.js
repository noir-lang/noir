const cache = require("@actions/cache");
const core = require("@actions/core");
const fs = require("fs");

async function main() {
  const successData = `https://github.com/AztecProtocol/aztec-packages/actions/runs/${process.env.RUN_ID}`;
  fs.writeFileSync("success.txt", successData);
  await cache.saveCache(["success.txt"], core.getInput("success_key"));
}

main();
