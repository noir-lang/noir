import { Noir } from '@noir-lang/noir_js';
import { Barretenberg, UltraHonkBackend } from '@aztec/bb.js';
import initNoirC from '@noir-lang/noirc_abi';
import initACVM from '@noir-lang/acvm_js';

// Initialize WASM modules
async function initNoir() {
    try {
        await Promise.all([initACVM(), initNoirC()]);
        log("Noir WASM initialized.");
    } catch (e) {
        log("WASM Init Warning: " + e.message);
    }
}

initNoir();

async function log(msg) {
    const logEl = document.getElementById('log');
    if (logEl) {
        logEl.textContent += `> ${msg}\n`;
    }
    console.log(msg);
}

const userId = "admin-01";
const SERVER_URL = 'http://localhost:3002';

document.getElementById('registerBtn').onclick = async () => {
    try {
        document.getElementById('registerBtn').disabled = true;
        log("Generating device secret...");
        
        const secret = "123456789"; 
        localStorage.setItem('zk_recovery_secret', secret);

        log("Computing commitment...");
        const commitment = "0x28639695646197170138612745304918512140682229562719280975878893118742880056637"; 

        await fetch(`${SERVER_URL}/v1/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ userId, commitment })
        });

        document.getElementById('regStatus').textContent = "Registered successfully!";
        log("Commitment registered on server.");
    } catch (e) {
        log(`Error: ${e.message}`);
    } finally {
        document.getElementById('registerBtn').disabled = false;
    }
};

document.getElementById('recoverBtn').onclick = async () => {
    try {
        document.getElementById('recoverBtn').disabled = true;
        log("Fetching challenge from server...");
        const challengeRes = await fetch(`${SERVER_URL}/v1/challenge/${userId}`);
        const { challenge } = await challengeRes.json();
        log(`Challenge received: ${challenge}`);

        log("Loading Noir circuit...");
        const response = await fetch('/recovery.json');
        const circuit = await response.json();

        const noir = new Noir(circuit);
        log("Creating Barretenberg... ⏳");
        const barretenbergAPI = await Barretenberg.new();
        log("Creating UltraHonkBackend...");
        const backend = new UltraHonkBackend(circuit.bytecode, barretenbergAPI);

        const secret = localStorage.getItem('zk_recovery_secret');
        const commitment = "0x28639695646197170138612745304918512140682229562719280975878893118742880056637";

        const input = {
            device_secret: secret,
            commitment: commitment,
            challenge: challenge,
            user_id_hash: "1" 
        };

        log("Generating witness... ⏳");
        const { witness } = await noir.execute(input);
        
        log("Generating ZK Proof locally...");
        const proofData = await backend.generateProof(witness);
        log("Proof generated successfully!");

        log("Sending proof to server for verification...");
        const verifyRes = await fetch(`${SERVER_URL}/v1/recovery/verify`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ 
                userId, 
                proof: Array.from(proofData.proof), 
                publicInputs: proofData.publicInputs 
            })
        });

        const result = await verifyRes.json();
        if (result.status === 'success') {
            document.getElementById('recStatus').textContent = "Verified! Access Granted.";
            document.getElementById('recStatus').style.color = "green";
            log("SUCCESS: Access granted.");
        } else {
            throw new Error(result.error);
        }

    } catch (e) {
        log(`Error: ${e.message}`);
        document.getElementById('recStatus').textContent = `Recovery failed: ${e.message}`;
        document.getElementById('recStatus').style.color = "red";
    } finally {
        document.getElementById('recoverBtn').disabled = false;
    }
};
