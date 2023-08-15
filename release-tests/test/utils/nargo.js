import { default as path } from "node:path";

export const NARGO_BIN = process.env.NARGO_BIN ? path.resolve(process.env.NARGO_BIN) : "nargo";
