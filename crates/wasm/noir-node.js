import fs from "node:fs";

import { init } from "./init.js";

async function compileWasm(filename) {
  const base = new URL("./binding/", import.meta.url);
  const path = new URL(filename, base);
  const wasm = await fs.promises.readFile(path);
  return WebAssembly.compile(wasm);
}

const { compile, acirToBytes, acirFromBytes } = await init(compileWasm, {
  readFile(file) {
    return fs.readFileSync(file, "utf8");
  },
});

export { compile, acirToBytes, acirFromBytes };
