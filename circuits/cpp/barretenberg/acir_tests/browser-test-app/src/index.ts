import createDebug from "debug";
import { inflate } from "pako";

createDebug.enable("*");
const debug = createDebug("simple_test");

async function runTest(
  bytecode: Uint8Array,
  witness: Uint8Array,
  threads?: number
) {
  const { Barretenberg, RawBuffer, Crs } = await import("@aztec/bb.js");
  const CIRCUIT_SIZE = 2 ** 19;

  debug("starting test...");
  const api = await Barretenberg.new(threads);

  // Important to init slab allocator as first thing, to ensure maximum memory efficiency.
  await api.commonInitSlabAllocator(CIRCUIT_SIZE);

  // Plus 1 needed!
  const crs = await Crs.new(CIRCUIT_SIZE + 1);
  await api.srsInitSrs(
    new RawBuffer(crs.getG1Data()),
    crs.numPoints,
    new RawBuffer(crs.getG2Data())
  );

  const acirComposer = await api.acirNewAcirComposer(CIRCUIT_SIZE);
  const proof = await api.acirCreateProof(
    acirComposer,
    bytecode,
    witness,
    true
  );
  debug(`verifying...`);
  const verified = await api.acirVerifyProof(acirComposer, proof, true);
  debug(`verified: ${verified}`);

  await api.destroy();

  debug("test complete.");
  return verified;
}

(window as any).runTest = runTest;

function base64ToUint8Array(base64: string) {
  let binaryString = atob(base64);
  let len = binaryString.length;
  let bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

// This is the 1_mul test, for triggering via the button click.
const acir = inflate(
  base64ToUint8Array(
    "H4sIAAAAAAAA/+2W3W6CQBCFB7AqVak/VdM0TXmAXuzyo3BXH6Wm+P6P0G5cZCS2N5whkrCJmWUjh505zPKFRPRB5+H8/lwbQ3bt1q49e82HY+OnjbHaJUmxjwod6y8V5ccsVUl63GU602mWfkdZHBdZku3zY75XuU7iQp/SPD6p8xg014qslvbY/v7bs2o29ACnpfh+H9h8YKPL1jwbhwI5Ue059ToGN9agD5cw6UFAd0i4l18q7yHeI8UkRWuqGg6PqkaR2Gt5OErWF6StBbUvz+C1GNk4Zmu+jeUHxowh86b0yry3B3afw6LDNA7snlv/cf7Q8dlaeX9AMr0icEAr0QO4fKmNgSFVBDCmigCkGglNFC8k05QeZp8XWhkBcx4DfZGqH9pnH+hFW+To47SuyPGRzXtybKjp24KidSd03+Ro8p7gPRIlxwlwn9LkaA7psXB9Qdqtk+PUxhlb68kRo9kKORoDQ6rIcUZy5Fg2EpooXkmmKdHkOAXmPAP6IlU/tM8BdY8cA5Ihxyc278mxoWZgC4rWndN9k6PJe473SJQc59QdcjSH9Ey4viDt1slxYeOSrfXkiNFshRyNgSFV5LgkOXIsGwlNFG8k05RoclwAc14CfZGqH9rnFXWPHFckQ47PbN6TY0PNlS0oWndN902OJu813iNRclxTd8jRHNJL4fqCtFsnx42NW7bWkyNGsxVyNAaGVJHjluTIsWwkNFG8k0xToslxA8x5C/RFqn4u2GcPmDOwfoofTi5df4zq4we8wQQCRCoAAA=="
  )
);

const witness = inflate(
  base64ToUint8Array(
    "H4sIAAAAAAAC/63UR84DIQyG4b/3mqooinIFG2xsdrkKTOD+R0ibRfb5kEbD6hF6BV7fXdb98duNe7ptxQecJY8Ai3Ph0//pyoqURJqFxpELhVxdSbQmZ2d13QePsbm45ZqNMkts3DWHPpLP1+caMW1myU1tqKmIa6DkWuOgQpJT49KHbl3Z9928t+LhbMoLrhe9Aq03nDW8A9t/ANt/Ant9Aa1vmJXpB9j+F9j+D9jrH2hNQJYep84U2H4GbD8H9loArSVw3q+A82sNfI8b4P3aAnsdAI07wlwMCAAA"
  )
);

document.addEventListener("DOMContentLoaded", function () {
  const button = document.createElement("button");
  button.innerText = "Run Test";
  button.addEventListener("click", () => runTest(acir, witness));
  document.body.appendChild(button);
});
