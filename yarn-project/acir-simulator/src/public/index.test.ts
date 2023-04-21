import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { CallContext, FunctionData } from '@aztec/circuits.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { FunctionAbi } from '@aztec/noir-contracts';
import { PublicTokenContractAbi } from '@aztec/noir-contracts/examples';
import { MockProxy, mock } from 'jest-mock-extended';
import { default as memdown, type MemDown } from 'memdown';
import { encodeArguments } from '../abi_coder/encoder.js';
import { NoirPoint, computeSlotForMapping, toPublicKey } from '../utils.js';
import { PublicDB } from './db.js';
import { PublicExecution } from './execution.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('ACIR public execution simulator', () => {
  let bbWasm: BarretenbergWasm;
  let oracle: MockProxy<PublicDB>;

  beforeAll(async () => {
    bbWasm = await BarretenbergWasm.get();
  });

  beforeEach(() => {
    oracle = mock<PublicDB>();
  });

  describe('PublicToken contract', () => {
    let recipientPk: Buffer;
    let recipient: NoirPoint;

    beforeAll(() => {
      recipientPk = Buffer.from('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec', 'hex');

      const grumpkin = new Grumpkin(bbWasm);
      recipient = toPublicKey(recipientPk, grumpkin);
    });

    describe('mint', () => {
      it('should run the mint function', async () => {
        const contractAddress = AztecAddress.random();
        const abi = PublicTokenContractAbi.functions.find(f => f.name === 'mint')!;
        const functionData = new FunctionData(Buffer.alloc(4), false, false);
        const args = encodeArguments(abi, [140, recipient], false);

        const callContext = CallContext.from({
          msgSender: AztecAddress.random(),
          storageContractAddress: contractAddress,
          portalContractAddress: EthAddress.random(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
        });

        // Mock the old value for the recipient balance to be 20
        const previousBalance = new Fr(20n);
        oracle.storageRead.mockResolvedValue(previousBalance);

        const execution = new PublicExecution(oracle, abi, contractAddress, functionData, args, callContext);
        const result = await execution.run();

        const expectedBalance = new Fr(160n);
        expect(result.returnValues).toEqual([expectedBalance]);

        const storageSlot = computeSlotForMapping(new Fr(1n), recipient, bbWasm);
        expect(result.stateTransitions).toEqual([
          { storageSlot, oldValue: previousBalance, newValue: expectedBalance },
        ]);

        expect(result.stateReads).toEqual([]);
      });
    });

    describe('transfer', () => {
      let contractAddress: AztecAddress;
      let abi: FunctionAbi;
      let functionData: FunctionData;
      let args: Fr[];
      let sender: AztecAddress;
      let callContext: CallContext;
      let recipientStorageSlot: Fr;
      let senderStorageSlot: Fr;

      beforeEach(() => {
        contractAddress = AztecAddress.random();
        abi = PublicTokenContractAbi.functions.find(f => f.name === 'transfer')!;
        functionData = new FunctionData(Buffer.alloc(4), false, false);
        args = encodeArguments(abi, [140, recipient], false);
        sender = AztecAddress.random();

        callContext = CallContext.from({
          msgSender: sender,
          storageContractAddress: contractAddress,
          portalContractAddress: EthAddress.random(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
        });

        recipientStorageSlot = computeSlotForMapping(new Fr(1n), recipient, bbWasm);
        senderStorageSlot = computeSlotForMapping(new Fr(1n), Fr.fromBuffer(sender.toBuffer()), bbWasm);
      });

      const mockStore = (senderBalance: Fr, recipientBalance: Fr) => {
        // eslint-disable-next-line require-await
        oracle.storageRead.mockImplementation(async (_addr: AztecAddress, slot: Fr) => {
          if (slot.equals(recipientStorageSlot)) {
            return recipientBalance;
          } else if (slot.equals(senderStorageSlot)) {
            return senderBalance;
          } else {
            return Fr.ZERO;
          }
        });
      };

      it('should run the transfer function', async () => {
        const senderBalance = new Fr(200n);
        const recipientBalance = new Fr(20n);
        mockStore(senderBalance, recipientBalance);

        const execution = new PublicExecution(oracle, abi, contractAddress, functionData, args, callContext);
        const result = await execution.run();

        const expectedRecipientBalance = new Fr(160n);
        const expectedSenderBalance = new Fr(60n);

        expect(result.returnValues).toEqual([expectedRecipientBalance]);

        expect(result.stateTransitions).toEqual([
          { storageSlot: senderStorageSlot, oldValue: senderBalance, newValue: expectedSenderBalance },
          { storageSlot: recipientStorageSlot, oldValue: recipientBalance, newValue: expectedRecipientBalance },
        ]);

        expect(result.stateReads).toEqual([]);
      });

      // State reads and writes are implemented as built-ins, which at the moment Noir does not
      // now whether they have side-effects or not, so they get run even when their code path
      // is not picked by a conditional. Once that's fixed, we should re-enable this test.
      it.skip('should run the transfer function without enough sender balance', async () => {
        const senderBalance = new Fr(10n);
        const recipientBalance = new Fr(20n);
        mockStore(senderBalance, recipientBalance);

        const execution = new PublicExecution(oracle, abi, contractAddress, functionData, args, callContext);
        const result = await execution.run();

        expect(result.returnValues).toEqual([recipientBalance]);

        expect(result.stateReads).toEqual([
          { storageSlot: recipientStorageSlot, value: recipientBalance },
          { storageSlot: senderStorageSlot, value: senderBalance },
        ]);

        expect(result.stateTransitions).toEqual([]);
      });
    });
  });
});
