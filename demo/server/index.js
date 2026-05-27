const express = require('express');
const cors = require('cors');
const path = require('path');
const fs = require('fs');
const { Barretenberg, UltraHonkBackend } = require('@aztec/bb.js');

const app = express();
app.use(cors());
app.use(express.json());

// Singleton Barretenberg API for server
let barretenbergAPI;
async function getBarretenbergAPI() {
    if (!barretenbergAPI) {
        barretenbergAPI = await Barretenberg.new();
    }
    return barretenbergAPI;
}

// In-memory DB for demo
const users = {}; // { userId: commitment }
const usedNullifiers = new Set();
const activeChallenges = {}; // { userId: challenge }

// Load circuit artifact (Expected at this path after build script)
const circuitPath = path.join(__dirname, 'artifacts', 'recovery.json');
let recoveryCircuit;
if (fs.existsSync(circuitPath)) {
    recoveryCircuit = JSON.parse(fs.readFileSync(circuitPath, 'utf8'));
}

app.post('/v1/register', (req, res) => {
    const { userId, commitment } = req.body;
    users[userId] = commitment;
    console.log(`Registered user ${userId} with commitment ${commitment}`);
    res.json({ status: 'success' });
});

app.get('/v1/challenge/:userId', (req, res) => {
    const challenge = Math.floor(Math.random() * 1000000).toString();
    activeChallenges[req.params.userId] = challenge;
    res.json({ challenge });
});

app.post('/v1/recovery/verify', async (req, res) => {
    const { userId, proof, publicInputs } = req.body;
    
    if (!recoveryCircuit) {
        return res.status(500).json({ error: 'Circuit artifact not found. Run build script.' });
    }

    try {
        const api = await getBarretenbergAPI();
        const backend = new UltraHonkBackend(recoveryCircuit.bytecode, api);

        // Public Inputs Structure (from circuit main function returns/inputs):
        // Note: The order depends on how Noir organizes public inputs and return values.
        // In Noir circuits, parameters are usually first, then the return value.
        // Input: commitment (pub), challenge (pub), user_id_hash (pub)
        // Return: nullifier (pub)
        // So publicInputs might be [commitment, challenge, user_id_hash, nullifier]
        const [commitment, challenge, userIdHash, nullifier] = publicInputs;

        // 1. Cryptographic Verification
        const isValid = await backend.verifyProof({ proof, publicInputs });

        if (!isValid) {
            return res.status(400).json({ error: 'Invalid ZK Proof' });
        }

        // 2. Business Logic Verification
        if (users[userId] !== commitment) {
            return res.status(400).json({ error: 'Commitment mismatch' });
        }

        if (activeChallenges[userId] !== challenge) {
            return res.status(400).json({ error: 'Invalid or expired challenge' });
        }

        if (usedNullifiers.has(nullifier)) {
            return res.status(400).json({ error: 'Proof already used (nullifier hit)' });
        }

        // 3. Success
        usedNullifiers.add(nullifier);
        delete activeChallenges[userId]; // Consume challenge
        
        console.log(`User ${userId} successfully recovered via ZK!`);
        res.json({ status: 'success', message: 'Recovery authorized' });

    } catch (err) {
        console.error(err);
        res.status(500).json({ error: err.message });
    }
});

const PORT = process.env.PORT || 3002;
app.listen(PORT, () => {
    console.log(`ZK Recovery Server running on http://localhost:${PORT}`);
});
