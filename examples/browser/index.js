// docs:start:imports
import { UltraHonkBackend } from '@aztec/bb.js';
import { Noir } from '@noir-lang/noir_js';
import initNoirC from '@noir-lang/noirc_abi';
import initACVM from '@noir-lang/acvm_js';
import acvm from '@noir-lang/acvm_js/web/acvm_js_bg.wasm?url';
import noirc from '@noir-lang/noirc_abi/web/noirc_abi_wasm_bg.wasm?url';
import circuit from './circuit/target/circuit.json';
// Initialize WASM modules
await Promise.all([initACVM(fetch(acvm)), initNoirC(fetch(noirc))]);
// docs:end:imports

// docs:start:show_function
const show = (id, content) => {
  const container = document.getElementById(id);
  container.appendChild(document.createTextNode(content));
  container.appendChild(document.createElement('br'));
};
// docs:end:show_function

document.getElementById('submit').addEventListener('click', async () => {
  try {
    // docs:start:init
    const noir = new Noir(circuit);
    const backend = new UltraHonkBackend(circuit.bytecode);
    // docs:end:init
    // docs:start:execute
    const age = document.getElementById('age').value;
    show('logs', 'Generating witness... ⏳');
    const { witness } = await noir.execute({ age });
    show('logs', 'Generated witness... ✅');
    // docs:end:execute
    // docs:start:prove
    show('logs', 'Generating proof... ⏳');
    const proof = await backend.generateProof(witness);
    show('logs', 'Generated proof... ✅');
    show('results', proof.proof);
    // docs:end:prove

    // docs:start:verify
    show('logs', 'Verifying proof... ⌛');
    const isValid = await backend.verifyProof(proof);
    show('logs', `Proof is ${isValid ? 'valid' : 'invalid'}... ✅`);
    // docs:end:verify
  } catch {
    show('logs', 'Oh 💔');
  }
});
