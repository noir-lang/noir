import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { AvmTestContractArtifact } from '@aztec/noir-contracts';

import { jest } from '@jest/globals';

import { TypeTag } from './avm_memory_types.js';
import { AvmSimulator } from './avm_simulator.js';
import { initContext, initExecutionEnvironment, initGlobalVariables } from './fixtures/index.js';
import { Add, CalldataCopy, Return } from './opcodes/index.js';
import { encodeToBytecode } from './serialization/bytecode_serialization.js';

describe('avm', () => {
  it('Should execute bytecode that performs basic addition', async () => {
    const calldata: Fr[] = [new Fr(1), new Fr(2)];

    // Construct bytecode
    const bytecode = encodeToBytecode([
      new CalldataCopy(/*indirect=*/ 0, /*cdOffset=*/ 0, /*copySize=*/ 2, /*dstOffset=*/ 0),
      new Add(/*indirect=*/ 0, TypeTag.FIELD, /*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Return(/*indirect=*/ 0, /*returnOffset=*/ 2, /*copySize=*/ 1),
    ]);

    const context = initContext({ env: initExecutionEnvironment({ calldata }) });
    jest.spyOn(context.worldState.hostStorage.contractsDb, 'getBytecode').mockReturnValue(Promise.resolve(bytecode));

    const results = await new AvmSimulator(context).execute();

    expect(results.reverted).toBe(false);

    const returnData = results.output;
    expect(returnData.length).toBe(1);
    expect(returnData).toEqual([new Fr(3)]);
  });

  describe('testing transpiled Noir contracts', () => {
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/4361): sync wire format w/transpiler.
    it('Should execute contract function that performs addition', async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];

      // Get contract function artifact
      const addArtifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_addArgsReturn')!;

      // Decode bytecode into instructions
      const bytecode = Buffer.from(addArtifact.bytecode, 'base64');

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      jest.spyOn(context.worldState.hostStorage.contractsDb, 'getBytecode').mockReturnValue(Promise.resolve(bytecode));

      const results = await new AvmSimulator(context).execute();

      expect(results.reverted).toBe(false);

      const returnData = results.output;
      expect(returnData.length).toBe(1);
      expect(returnData).toEqual([new Fr(3)]);
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
          .spyOn(context.worldState.hostStorage.contractsDb, 'getBytecode')
          .mockReturnValue(Promise.resolve(bytecode));
        // Execute

        const results = await new AvmSimulator(context).execute();

        expect(results.reverted).toBe(false);

        const returnData = results.output;
        expect(returnData.length).toBe(1);
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
  });
});
