import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
// We will fetch the compiled circuit from the server artifacts or local copy
// For demo purposes, we assume recovery.json is available at /recovery.json in the client root
// In a real build, the build script would copy it to demo/client/public/

async function log(msg) {
    const logEl = document.getElementById('log');
    logEl.textContent += `> ${msg}\n`;
    console.log(msg);
}

const userId = "admin-01";

document.getElementById('registerBtn').onclick = async () => {
    try {
        document.getElementById('registerBtn').disabled = true;
        log("Generating device secret...");
        
        // In a real app, this would be a secure random field element
        const secret = "123456789"; 
        localStorage.setItem('zk_recovery_secret', secret);

        // We need a way to hash in the browser to get the commitment
        // Usually we'd use a light poseidon lib or noir_js itself
        log("Computing commitment (simulated)...");
        // For simplicity in this demo logic, we'll let the server tell us if we're wrong 
        // or use a fixed one if we don't want to run the full ZK stack for registration
        const commitment = "0x28639695646197170138612745304918512140682229562719280975878893118742880056637"; // Poseidon2(123456789)

        await fetch('http://localhost:3001/v1/register', {
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
        const { challenge } = await (await fetch(`http://localhost:3001/v1/challenge/${userId}`)).json();
        log(`Challenge received: ${challenge}`);

        log("Loading Noir circuit...");
        const response = await fetch('/recovery.json');
        const circuit = await response.json();

        const backend = new BarretenbergBackend(circuit);
        const noir = new Noir(circuit, backend);

        const secret = localStorage.getItem('zk_recovery_secret');
        const commitment = "0x28639695646197170138612745304918512140682229562719280975878893118742880056637";

        const input = {
            device_secret: secret,
            commitment: commitment,
            challenge: challenge,
            user_id_hash: "1" // Simplified
        };

        log("Generating ZK Proof locally (this may take a few seconds)...");
        const { proof, publicInputs } = await noir.generateProof(input);
        log("Proof generated successfully!");

        log("Sending proof to server for verification...");
        const verifyRes = await fetch('http://localhost:3001/v1/recovery/verify', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ userId, proof, publicInputs })
        });

        const result = await verifyRes.json();
        if (result.status === 'success') {
            document.getElementById('recStatus').textContent = "Verified! Access Granted.";
            document.getElementById('recStatus').style.color = "green";
            log("SUCCESS: Access granted by server.");
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
