const express = require('express');
const cors = require('cors');
const path = require('path');
const fs = require('fs');
const crypto = require('crypto');
const { Barretenberg, UltraHonkBackend } = require('@aztec/bb.js');

const app = express();
app.use(cors());
app.use(express.json());

// Singleton Barretenberg API for server
// Use single-threaded mode: verification doesn't need workers, and Vercel
// serverless functions can't use SharedArrayBuffer-based threading.
let barretenbergAPI;
async function getBarretenbergAPI() {
    if (!barretenbergAPI) {
        barretenbergAPI = await Barretenberg.new(1);
    }
    return barretenbergAPI;
}

// In-memory DB for demo
const users = {}; // { userId: commitment }
const usedNullifiers = new Set();
const activeChallenges = {}; // { userId: challenge }
const DEMO_USER_ID = 'admin-01';
const DEMO_COMMITMENT = '15241578750190522';
const CHALLENGE_TTL_MS = 5 * 60 * 1000;
const CHALLENGE_SECRET = process.env.RECOVERY_CHALLENGE_SECRET || 'local-demo-challenge-secret';

function normalizeField(value) {
    return BigInt(value).toString();
}

function getRegisteredCommitment(userId) {
    if (users[userId]) {
        return users[userId];
    }

    if (userId === DEMO_USER_ID) {
        return DEMO_COMMITMENT;
    }

    return undefined;
}

function signChallenge(userId, challenge, expiresAt) {
    const payload = `${userId}:${challenge}:${expiresAt}`;
    const signature = crypto.createHmac('sha256', CHALLENGE_SECRET).update(payload).digest('hex');
    return Buffer.from(`${payload}:${signature}`, 'utf8').toString('base64url');
}

function verifyChallengeToken(token, userId, challenge) {
    if (!token) {
        return activeChallenges[userId] === challenge;
    }

    const decoded = Buffer.from(token, 'base64url').toString('utf8');
    const parts = decoded.split(':');
    if (parts.length !== 4) {
        return false;
    }

    const [tokenUserId, tokenChallenge, expiresAt, signature] = parts;
    if (tokenUserId !== userId || tokenChallenge !== challenge || Number(expiresAt) < Date.now()) {
        return false;
    }

    const expected = crypto.createHmac('sha256', CHALLENGE_SECRET)
        .update(`${tokenUserId}:${tokenChallenge}:${expiresAt}`)
        .digest('hex');

    return crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expected));
}

// Load circuit artifact (Expected at this path after build script)
const circuitPath = path.join(__dirname, 'artifacts', 'recovery.json');
let recoveryCircuit;
if (fs.existsSync(circuitPath)) {
    recoveryCircuit = JSON.parse(fs.readFileSync(circuitPath, 'utf8'));
} else {
    console.warn(`Circuit artifact not found at ${circuitPath}. Run scripts/zk/build-recovery.ps1 before recovery.`);
}

app.post('/v1/register', (req, res) => {
    const { userId, commitment } = req.body;
    users[userId] = normalizeField(commitment);
    console.log(`Registered user ${userId} with commitment ${commitment}`);
    res.json({ status: 'success' });
});

app.get('/v1/challenge/:userId', (req, res) => {
    const challenge = Math.floor(Math.random() * 1000000).toString();
    activeChallenges[req.params.userId] = challenge;
    const challengeToken = signChallenge(req.params.userId, challenge, Date.now() + CHALLENGE_TTL_MS);
    res.json({ challenge, challengeToken });
});

app.get('/v1/circuit', (req, res) => {
    if (!recoveryCircuit) {
        return res.status(500).json({ error: 'Circuit artifact not found. Run scripts/zk/build-recovery.ps1.' });
    }

    res.json(recoveryCircuit);
});

app.post('/v1/recovery/verify', async (req, res) => {
    const { userId, proof, publicInputs, challengeToken } = req.body;
    
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
        const [commitment, challenge, userIdHash, nullifier] = publicInputs.map(normalizeField);

        // 1. Cryptographic Verification
        const isValid = await backend.verifyProof({ proof: new Uint8Array(proof), publicInputs });

        if (!isValid) {
            return res.status(400).json({ error: 'Invalid ZK Proof' });
        }

        // 2. Business Logic Verification
        if (getRegisteredCommitment(userId) !== commitment) {
            return res.status(400).json({ error: 'Commitment mismatch' });
        }

        if (!verifyChallengeToken(challengeToken, userId, challenge)) {
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

if (require.main === module) {
    const PORT = process.env.PORT || 3002;
    app.listen(PORT, () => {
        console.log(`ZK Recovery Server running on http://localhost:${PORT}`);
    });
}

module.exports = app;
