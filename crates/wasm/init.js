import { instantiate } from "./binding/noir_wasm.component.js";

export async function init(compileWasm, fs) {
  return await instantiate(compileWasm, {
    fs,
    console: {
      log(msg) {
        console.log(msg);
      },
      error(msg) {
        console.error(msg);
      },
    },
  });
}
