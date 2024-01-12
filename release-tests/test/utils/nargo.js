import { default as path } from "path";

export const NARGO_BIN = process.env.NARGO_BIN ? path.resolve(process.env.NARGO_BIN) : "nargo";
