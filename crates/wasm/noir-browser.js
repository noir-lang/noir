// WIP - need to test with bundlers

import { init } from "./init.js";

async function compileWasm(filename) {
  const base = new URL("./binding/", import.meta.url);
  const path = new URL(filename, base);
  const wasm = await fetch(path);
  return WebAssembly.compile(wasm);
}

const { compile, acirToBytes, acirFromBytes } = await init(compileWasm, {
  readFile(file) {
    return fetch(file).then((res) => res.text());
  },
});

export { compile, acirToBytes, acirFromBytes };
