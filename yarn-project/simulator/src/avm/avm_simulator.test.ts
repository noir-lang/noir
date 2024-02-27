import { AztecAddress } from '@aztec/foundation/aztec-address';
import { keccak, pedersenHash, poseidonHash, sha256 } from '@aztec/foundation/crypto';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { AvmTestContractArtifact } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { TypeTag } from './avm_memory_types.js';
import { AvmSimulator } from './avm_simulator.js';
import { initContext, initExecutionEnvironment, initGlobalVariables } from './fixtures/index.js';
import { Add, CalldataCopy, Return } from './opcodes/index.js';
import { encodeToBytecode } from './serialization/bytecode_serialization.js';

describe('AVM simulator', () => {
  it('Should execute bytecode that performs basic addition', async () => {
    const calldata: Fr[] = [new Fr(1), new Fr(2)];

    // Construct bytecode
    const bytecode = encodeToBytecode([
      new CalldataCopy(/*indirect=*/ 0, /*cdOffset=*/ 0, /*copySize=*/ 2, /*dstOffset=*/ 0),
      new Add(/*indirect=*/ 0, TypeTag.FIELD, /*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Return(/*indirect=*/ 0, /*returnOffset=*/ 2, /*copySize=*/ 1),
    ]);

    const context = initContext({ env: initExecutionEnvironment({ calldata }) });
    jest
      .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
      .mockReturnValue(Promise.resolve(bytecode));

    const results = await new AvmSimulator(context).execute();

    expect(results.reverted).toBe(false);

    const returnData = results.output;
    expect(returnData.length).toBe(1);
    expect(returnData).toEqual([new Fr(3)]);
  });

  describe('Transpiled Noir contracts', () => {
    it('Should execute contract function that performs addition', async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];

      // Get contract function artifact
      const addArtifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_addArgsReturn')!;

      // Decode bytecode into instructions
      const bytecode = Buffer.from(addArtifact.bytecode, 'base64');

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(bytecode));

      const results = await new AvmSimulator(context).execute();

      expect(results.reverted).toBe(false);

      const returnData = results.output;
      expect(returnData).toEqual([new Fr(3)]);
    });

    describe.each([
      ['avm_setOpcodeUint8', 8n],
      // ['avm_setOpcodeUint16', 60000n],
      ['avm_setOpcodeUint32', 1n << 30n],
      ['avm_setOpcodeUint64', 1n << 60n],
      // ['avm_setOpcodeUint128', 1n << 120n],
      ['avm_setOpcodeSmallField', 200n],
    ])('Should execute contract SET functions', (name: string, res: bigint) => {
      it(`Should execute contract function '${name}'`, async () => {
        // Decode bytecode into instructions
        const artifact = AvmTestContractArtifact.functions.find(f => f.name === name)!;
        const bytecode = Buffer.from(artifact.bytecode, 'base64');

        const context = initContext();
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));

        const results = await new AvmSimulator(context).execute();

        expect(results.reverted).toBe(false);

        const returnData = results.output;
        expect(returnData).toEqual([new Fr(res)]);
      });
    });

    describe.each([
      ['avm_sha256_hash', sha256],
      ['avm_keccak_hash', keccak],
    ])('Hashes with 2 fields returned in noir contracts', (name: string, hashFunction: (data: Buffer) => Buffer) => {
      it(`Should execute contract function that performs ${name} hash`, async () => {
        const calldata = [new Fr(1), new Fr(2), new Fr(3)];
        const hash = hashFunction(Buffer.concat(calldata.map(f => f.toBuffer())));

        // Get contract function artifact
        const artifact = AvmTestContractArtifact.functions.find(f => f.name === name)!;

        // Decode bytecode into instructions
        const bytecode = Buffer.from(artifact.bytecode, 'base64');

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));

        const results = await new AvmSimulator(context).execute();

        expect(results.reverted).toBe(false);

        const returnData = results.output;
        const reconstructedHash = Buffer.concat([
          returnData[0].toBuffer().subarray(16, 32),
          returnData[1].toBuffer().subarray(16, 32),
        ]);
        expect(reconstructedHash).toEqual(hash);
      });
    });

    describe.each([
      ['avm_poseidon_hash', poseidonHash],
      ['avm_pedersen_hash', pedersenHash],
    ])('Hashes with field returned in noir contracts', (name: string, hashFunction: (data: Buffer[]) => Fr) => {
      it(`Should execute contract function that performs ${name} hash`, async () => {
        const calldata = [new Fr(1), new Fr(2), new Fr(3)];
        const hash = hashFunction(calldata.map(f => f.toBuffer()));

        // Get contract function artifact
        const artifact = AvmTestContractArtifact.functions.find(f => f.name === name)!;

        // Decode bytecode into instructions
        const bytecode = Buffer.from(artifact.bytecode, 'base64');

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));

        const results = await new AvmSimulator(context).execute();

        expect(results.reverted).toBe(false);

        const returnData = results.output;
        expect(returnData).toEqual([new Fr(hash)]);
      });
    });

    describe('Test env getters from noir contract', () => {
      const testEnvGetter = async (valueName: string, value: any, functionName: string, globalVar: boolean = false) => {
        const getterArtifact = AvmTestContractArtifact.functions.find(f => f.name === functionName)!;

        // Execute
        let overrides = {};
        if (globalVar === true) {
          const globals = initGlobalVariables({ [valueName]: value });
          overrides = { globals };
        } else {
          overrides = { [valueName]: value };
        }
        const context = initContext({ env: initExecutionEnvironment(overrides) });

        // Decode bytecode into instructions
        const bytecode = Buffer.from(getterArtifact.bytecode, 'base64');
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));
        // Execute

        const results = await new AvmSimulator(context).execute();

        expect(results.reverted).toBe(false);

        const returnData = results.output;
        expect(returnData).toEqual([value.toField()]);
      };

      it('address', async () => {
        const address = AztecAddress.fromField(new Fr(1));
        await testEnvGetter('address', address, 'avm_getAddress');
      });

      it('storageAddress', async () => {
        const storageAddress = AztecAddress.fromField(new Fr(1));
        await testEnvGetter('storageAddress', storageAddress, 'avm_getStorageAddress');
      });

      it('sender', async () => {
        const sender = AztecAddress.fromField(new Fr(1));
        await testEnvGetter('sender', sender, 'avm_getSender');
      });

      it('origin', async () => {
        const origin = AztecAddress.fromField(new Fr(1));
        await testEnvGetter('origin', origin, 'avm_getOrigin');
      });

      it('portal', async () => {
        const portal = EthAddress.fromField(new Fr(1));
        await testEnvGetter('portal', portal, 'avm_getPortal');
      });

      it('getFeePerL1Gas', async () => {
        const fee = new Fr(1);
        await testEnvGetter('feePerL1Gas', fee, 'avm_getFeePerL1Gas');
      });

      it('getFeePerL2Gas', async () => {
        const fee = new Fr(1);
        await testEnvGetter('feePerL2Gas', fee, 'avm_getFeePerL2Gas');
      });

      it('getFeePerDaGas', async () => {
        const fee = new Fr(1);
        await testEnvGetter('feePerDaGas', fee, 'avm_getFeePerDaGas');
      });

      it('chainId', async () => {
        const chainId = new Fr(1);
        await testEnvGetter('chainId', chainId, 'avm_getChainId', /*globalVar=*/ true);
      });

      it('version', async () => {
        const version = new Fr(1);
        await testEnvGetter('version', version, 'avm_getVersion', /*globalVar=*/ true);
      });

      it('blockNumber', async () => {
        const blockNumber = new Fr(1);
        await testEnvGetter('blockNumber', blockNumber, 'avm_getBlockNumber', /*globalVar=*/ true);
      });

      it('timestamp', async () => {
        const timestamp = new Fr(1);
        await testEnvGetter('timestamp', timestamp, 'avm_getTimestamp', /*globalVar=*/ true);
      });
    });

    describe('Test tree access functions from noir contract', () => {
      it(`Should execute contract function to emit note hash (should be traced)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        // Get contract function artifact
        const artifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_new_note_hash')!;

        // Decode bytecode into instructions
        const bytecode = Buffer.from(artifact.bytecode, 'base64');

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));

        const results = await new AvmSimulator(context).execute();

        expect(results.reverted).toBe(false);

        expect(context.persistableState.flush().newNoteHashes).toEqual([utxo]);
      });
      it(`Should execute contract function to emit nullifier (should be traced)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        // Get contract function artifact
        const artifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_new_nullifier')!;

        // Decode bytecode into instructions
        const bytecode = Buffer.from(artifact.bytecode, 'base64');

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));

        const results = await new AvmSimulator(context).execute();

        expect(results.reverted).toBe(false);

        expect(context.persistableState.flush().newNullifiers).toEqual([utxo]);
      });
      it(`Should execute contract function that checks if a nullifier existence (it does not)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        // Get contract function artifact
        const artifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_check_nullifier_exists')!;

        // Decode bytecode into instructions
        const bytecode = Buffer.from(artifact.bytecode, 'base64');

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));

        await new AvmSimulator(context).execute();
        const results = await new AvmSimulator(context).execute();
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([/*exists=false*/ new Fr(0)]);

        // Nullifier existence check should be in trace
        const sideEffects = context.persistableState.flush();
        expect(sideEffects.nullifierChecks.length).toEqual(1);
        expect(sideEffects.nullifierChecks[0].exists).toEqual(false);
      });
      it(`Should execute contract function that checks if a nullifier existence (it does)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        // Get contract function artifact
        const artifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_check_nullifier_exists')!;

        // Decode bytecode into instructions
        const bytecode = Buffer.from(artifact.bytecode, 'base64');

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));

        // nullifier exists!
        jest
          .spyOn(context.persistableState.hostStorage.commitmentsDb, 'getNullifierIndex')
          .mockReturnValue(Promise.resolve(BigInt(42)));

        await new AvmSimulator(context).execute();
        const results = await new AvmSimulator(context).execute();
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([/*exists=true*/ new Fr(1)]);

        // Nullifier existence check should be in trace
        const sideEffects = context.persistableState.flush();
        expect(sideEffects.nullifierChecks.length).toEqual(1);
        expect(sideEffects.nullifierChecks[0].exists).toEqual(true);
      });
      it(`Should execute contract function that checks emits a nullifier and checks its existence`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        // Get contract function artifact
        const artifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_emit_nullifier_and_check')!;

        // Decode bytecode into instructions
        const bytecode = Buffer.from(artifact.bytecode, 'base64');

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));

        await new AvmSimulator(context).execute();
        const results = await new AvmSimulator(context).execute();
        expect(results.reverted).toBe(false);

        // Nullifier existence check should be in trace
        const sideEffects = context.persistableState.flush();
        expect(sideEffects.newNullifiers).toEqual([utxo]);
        expect(sideEffects.nullifierChecks.length).toEqual(1);
        expect(sideEffects.nullifierChecks[0].exists).toEqual(true);
      });
      it(`Should execute contract function that emits same nullifier twice (should fail)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        // Get contract function artifact
        const artifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_nullifier_collision')!;

        // Decode bytecode into instructions
        const bytecode = Buffer.from(artifact.bytecode, 'base64');

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        jest
          .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));

        await new AvmSimulator(context).execute();
        const results = await new AvmSimulator(context).execute();
        expect(results.reverted).toBe(true);

        // Only the first nullifier should be in the trace, second one failed to add
        expect(context.persistableState.flush().newNullifiers).toEqual([utxo]);
      });
    });
  });
});
