import { replacePaths } from "./utils.js";
import { AZTEC_REPO } from "./config.js";
import tiged from "tiged";

export async function init(folder) {
  const emitter = tiged(`${AZTEC_REPO}/boxes/init${tag && `#${tag}`}`, {
    verbose: true,
  });
  emitter.on("info", ({ message }) => debug(message));
  emitter.on("warn", ({ message }) => error(message));
  await emitter.clone(`${folder}`);

  await replacePaths({
    rootDir: `${folder}`,
    tag,
    version,
    prefix: "",
  });
}
