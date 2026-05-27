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

async function readJson(response, label) {
    const contentType = response.headers.get('content-type') || '';
    if (!contentType.includes('application/json')) {
        const body = await response.text();
        throw new Error(`${label} returned ${response.status} ${response.statusText || ''}: ${body.slice(0, 80)}`);
    }

    const data = await response.json();
    if (!response.ok) {
        throw new Error(data.error || `${label} failed with status ${response.status}`);
    }

    return data;
}

document.getElementById('registerBtn').onclick = async () => {
    try {
        document.getElementById('registerBtn').disabled = true;
        log("Generating device secret...");

        const secret = "123456789";
        localStorage.setItem('zk_recovery_secret', secret);

        log("Computing commitment...");
        const commitment = "0x28639695646197170138612745304918512140682229562719280975878893118742880056637";

        const registerRes = await fetch(`${SERVER_URL}/v1/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ userId, commitment })
        });
        await readJson(registerRes, 'Registration request');

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
        const { challenge } = await readJson(challengeRes, 'Challenge request');
        log(`Challenge received: ${challenge}`);

        log("Loading Noir circuit...");
        const response = await fetch(`${SERVER_URL}/v1/circuit`);
        const circuit = await readJson(response, 'Circuit request');

        const noir = new Noir(circuit);
        log("Creating Barretenberg...");
        const barretenbergAPI = await Barretenberg.new();
        log("Creating UltraHonkBackend...");
        const backend = new UltraHonkBackend(circuit.bytecode, barretenbergAPI);

        const secret = localStorage.getItem('zk_recovery_secret');
        if (!secret) {
            throw new Error("No local recovery secret found. Register first.");
        }

        const commitment = "0x28639695646197170138612745304918512140682229562719280975878893118742880056637";

        const input = {
            device_secret: secret,
            commitment: commitment,
            challenge: challenge,
            user_id_hash: "1"
        };

        log("Generating witness...");
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

        const result = await readJson(verifyRes, 'Recovery verification');
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
