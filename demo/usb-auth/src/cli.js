#!/usr/bin/env node
import { readFile, writeFile } from 'node:fs/promises';
import { createEncryptedSecretFile, serializeEncryptedSecretFile } from './secret-file.js';
import { NodePathSecretProvider } from './node-providers.js';
import { computeCommitment, randomField, userIdToField } from './fields.js';
import { createAuthInputs, generateAndVerifyProof, proofToJson } from './proof.js';
import circuit from './circuit-artifact.js';

async function main() {
  const [command, ...rest] = process.argv.slice(2);
  const args = parseArgs(rest);

  if (command === 'register') {
    await register(args);
    return;
  }
  if (command === 'prove') {
    await prove(args);
    return;
  }
  if (command === 'verify') {
    await verify(args);
    return;
  }

  printUsage();
  process.exitCode = 1;
}

async function register(args) {
  requireArg(args, 'out');
  requireArg(args, 'pin');
  requireArg(args, 'user');
  const deviceSecret = randomField();
  const encryptedFile = await createEncryptedSecretFile(deviceSecret, args.pin, {
    deviceLabel: `USB ZK Auth: ${args.user}`,
  });
  await new NodePathSecretProvider().writeSecret({
    path: args.out,
    contents: serializeEncryptedSecretFile(encryptedFile),
  });
  const userIdHash = await userIdToField(args.user);
  console.log(
    JSON.stringify(
      {
        out: args.out,
        commitment: computeCommitment(deviceSecret, userIdHash),
        user_id_hash: userIdHash,
      },
      null,
      2,
    ),
  );
}

async function prove(args) {
  requireArg(args, 'secret');
  requireArg(args, 'pin');
  requireArg(args, 'user');
  const circuit = await loadCircuit();
  const deviceSecret = await new NodePathSecretProvider().readSecret({ path: args.secret, pin: args.pin });
  const authInputs = await createAuthInputs({
    deviceSecret,
    userId: args.user,
    challenge: args.challenge,
  });
  const result = await generateAndVerifyProof(circuit, authInputs);
  const proofJson = proofToJson(result);
  if (args.out) {
    await writeFile(args.out, `${JSON.stringify(proofJson, null, 2)}\n`, 'utf8');
  }
  console.log(JSON.stringify(proofJson, null, 2));
}

async function verify(args) {
  requireArg(args, 'proof');
  const proofJson = JSON.parse(await readFile(args.proof, 'utf8'));
  console.log(JSON.stringify({ verified: Boolean(proofJson.verified), nullifier: proofJson.nullifier }, null, 2));
}

async function loadCircuit() {
  return circuit;
}

function parseArgs(tokens) {
  const args = {};
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (!token.startsWith('--')) {
      continue;
    }
    args[token.slice(2)] = tokens[index + 1];
    index += 1;
  }
  return args;
}

function requireArg(args, name) {
  if (!args[name]) {
    throw new Error(`Missing --${name}.`);
  }
}

function printUsage() {
  console.log(`Usage:
  usb-auth register --out <path> --pin <pin> --user <user-id>
  usb-auth prove --secret <path> --pin <pin> --user <user-id> [--challenge <field>] [--out <path>]
  usb-auth verify --proof <path>`);
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
