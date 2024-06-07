const { spawnSync } = require("child_process");
const { expect } = require("chai");

async function runNargoCheck() {
  // Check for compile error
  const result = spawnSync("nargo", ["check"] );
  expect(result.status).to.equal(0, result.stderr.toString());
}

function runNargoTest(testFile, testName) {
  const testPath = `tests::${testFile}::${testName}`;
  const result = spawnSync("nargo", ["test", testPath] );

  expect(result.status).to.equal(0, "\n" + result.stderr.toString());
}

module.exports = {
  runNargoCheck,
  runNargoTest
};