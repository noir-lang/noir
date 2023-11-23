import { suite } from "uvu";
import * as assert from "uvu/assert";
import { NARGO_BIN } from "./utils/nargo.js";
import "./utils/zx.js";

const test = suite("nargo");

// Helps detect unresolved ProcessPromise.
let promiseResolved = false;
process.on("exit", () => {
  if (!promiseResolved) {
    console.error("Error: ProcessPromise never resolved.");
    process.exitCode = 1;
  }
});

test("promise resolved", async () => {
  await $`echo PromiseHelper`;
  promiseResolved = true;
});

test("prints version", async () => {
  const processOutput = (await $`${NARGO_BIN} --version`).toString();
  
  // Regex to match the "nargo version" part of the output
  assert.match(processOutput, /nargo version = \d{1,2}\.\d{1,2}\.\d{1,2}/);
});


test("reports a clean commit", async () => {
  const processOutput = (await $`${NARGO_BIN} --version`).toString();
  assert.not.match(processOutput, /is dirty: true/)
});

test.run();
