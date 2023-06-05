/* eslint-disable camelcase */
import { AztecNodeService } from '@aztec/aztec-node';
import {
  AztecAddress,
  AztecRPCServer,
  Contract,
  ContractDeployer,
  ContractFunctionInteraction,
  Fr,
  TxStatus,
} from '@aztec/aztec.js';
import { ContractAbi, FunctionType } from '@aztec/foundation/abi';
import { DebugLogger } from '@aztec/foundation/log';
import { AccountContractAbi, ChildAbi } from '@aztec/noir-contracts/examples';

import { ARGS_LENGTH } from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { sha256 } from '@aztec/foundation/crypto';
import { toBigInt } from '@aztec/foundation/serialize';
import { secp256k1 } from '@noble/curves/secp256k1';
import times from 'lodash.times';
import { setup } from './utils.js';

describe('e2e_account_contract', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let account: Contract;
  let child: Contract;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, accounts, logger } = await setup());

    account = await deployContract(AccountContractAbi);
    child = await deployContract(ChildAbi);
  }, 30_000);

  afterEach(async () => {
    await aztecNode.stop();
    await aztecRpcServer.stop();
  });

  const deployContract = async (abi: ContractAbi) => {
    logger(`Deploying L2 contract ${abi.name}...`);
    const deployer = new ContractDeployer(abi, aztecRpcServer);
    const tx = deployer.deploy().send();

    await tx.isMined(0, 0.1);

    const receipt = await tx.getReceipt();
    const contract = new Contract(receipt.contractAddress!, abi, aztecRpcServer);
    logger(`L2 contract ${abi.name} deployed at ${contract.address}`);
    return contract;
  };

  const callChildPubStoreValue = (value: number) => ({
    args: [new Fr(value)],
    selector: child.methods.pubStoreValue.selector,
    target: child.address,
  });

  const callChildValue = (value: number) => ({
    args: [new Fr(value)],
    selector: child.methods.value.selector,
    target: child.address,
  });

  // Copied from yarn-project/noir-contracts/src/contracts/account_contract/src/entrypoint.nr
  const ACCOUNT_MAX_PRIVATE_CALLS = 1;
  const ACCOUNT_MAX_PUBLIC_CALLS = 1;

  type FunctionCall = {
    args: Fr[];
    selector: Buffer;
    target: AztecAddress;
  };

  type EntrypointPayload = {
    flattened_args: Fr[];
    flattened_selectors: Fr[];
    flattened_targets: Fr[];
    nonce: Fr;
  };

  const flattenPayload = (payload: EntrypointPayload) => {
    return [...payload.flattened_args, ...payload.flattened_selectors, ...payload.flattened_targets, payload.nonce];
  };

  const toFrArray = (buf: Buffer) => Array.from(buf).map(byte => new Fr(byte));

  const buildPayload = (privateCalls: FunctionCall[], publicCalls: FunctionCall[]): EntrypointPayload => {
    const nonce = Fr.random();
    const emptyCall = { args: times(ARGS_LENGTH, Fr.zero), selector: Buffer.alloc(32), target: AztecAddress.ZERO };

    const calls = [
      ...padArrayEnd(privateCalls, emptyCall, ACCOUNT_MAX_PRIVATE_CALLS),
      ...padArrayEnd(publicCalls, emptyCall, ACCOUNT_MAX_PUBLIC_CALLS),
    ];

    return {
      flattened_args: calls.flatMap(call => padArrayEnd(call.args, Fr.ZERO, ARGS_LENGTH)),
      flattened_selectors: calls.map(call => Fr.fromBuffer(call.selector)),
      flattened_targets: calls.map(call => call.target.toField()),
      nonce,
    };
  };

  const buildCall = (payload: EntrypointPayload, opts: { privKey?: string } = {}) => {
    // Hash the payload object, so we sign over it
    // TODO: Switch to keccak when avaiable in Noir
    const payloadHash = sha256(Buffer.concat(flattenPayload(payload).map(fr => fr.toBuffer())));
    logger(`Payload hash: ${payloadHash.toString('hex')} (${payloadHash.length} bytes)`);

    // Sign using the private key that matches account contract's pubkey by default
    const privKeyString = opts.privKey ?? 'ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80';
    const privKey = Buffer.from(privKeyString, 'hex');
    const signatureObject = secp256k1.sign(payloadHash, privKey);
    const signature = Buffer.from(signatureObject.toCompactRawBytes());
    logger(`Signature: ${signature.toString('hex')} (${signature.length} bytes)`);

    // Create the method call using the actual args to send into Noir
    return new ContractFunctionInteraction(
      aztecRpcServer,
      account.address,
      'entrypoint',
      [...flattenPayload(payload), ...toFrArray(signature)],
      FunctionType.SECRET,
    );
  };

  it('calls a private function', async () => {
    const payload = buildPayload([callChildValue(42)], []);
    const call = buildCall(payload);
    const tx = call.send({ from: accounts[0] });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
  });

  it('calls a public function', async () => {
    const payload = buildPayload([], [callChildPubStoreValue(42)]);
    const call = buildCall(payload);
    const tx = call.send({ from: accounts[0] });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
    expect(toBigInt((await aztecNode.getStorageAt(child.address, 1n))!)).toEqual(42n);
  });

  it('rejects ecdsa signature from a different key', async () => {
    const payload = buildPayload([callChildValue(42)], []);
    const call = buildCall(payload, { privKey: '2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6' });
    await expect(call.create({ from: accounts[0] })).rejects.toMatch(/could not satisfy all constraints/);
  });
});
