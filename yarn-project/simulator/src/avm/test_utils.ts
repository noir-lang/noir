import { Fr } from '@aztec/circuits.js';
import { type ContractInstanceWithAddress } from '@aztec/types/contracts';

import { type jest } from '@jest/globals';
import { mock } from 'jest-mock-extended';

import { type CommitmentsDB, type PublicContractsDB, type PublicStateDB } from '../public/db_interfaces.js';
import { type PublicSideEffectTraceInterface } from '../public/side_effect_trace_interface.js';
import { type HostStorage } from './journal/host_storage.js';

export function mockGetBytecode(hs: HostStorage, bytecode: Buffer) {
  (hs as jest.Mocked<HostStorage>).contractsDb.getBytecode.mockResolvedValue(bytecode);
}

export function mockTraceFork(trace: PublicSideEffectTraceInterface, nestedTrace?: PublicSideEffectTraceInterface) {
  (trace as jest.Mocked<PublicSideEffectTraceInterface>).fork.mockReturnValue(
    nestedTrace ?? mock<PublicSideEffectTraceInterface>(),
  );
}

export function mockStorageRead(hs: HostStorage, value: Fr) {
  (hs.publicStateDb as jest.Mocked<PublicStateDB>).storageRead.mockResolvedValue(value);
}

export function mockStorageReadWithMap(hs: HostStorage, mockedStorage: Map<bigint, Fr>) {
  (hs.publicStateDb as jest.Mocked<PublicStateDB>).storageRead.mockImplementation((_address, slot) =>
    Promise.resolve(mockedStorage.get(slot.toBigInt()) ?? Fr.ZERO),
  );
}

export function mockNoteHashExists(hs: HostStorage, leafIndex: Fr, _value?: Fr) {
  (hs.commitmentsDb as jest.Mocked<CommitmentsDB>).getCommitmentIndex.mockResolvedValue(leafIndex.toBigInt());
}

export function mockNullifierExists(hs: HostStorage, leafIndex: Fr, _value?: Fr) {
  (hs.commitmentsDb as jest.Mocked<CommitmentsDB>).getNullifierIndex.mockResolvedValue(leafIndex.toBigInt());
}

export function mockL1ToL2MessageExists(hs: HostStorage, leafIndex: Fr, value: Fr, valueAtOtherIndices?: Fr) {
  (hs.commitmentsDb as jest.Mocked<CommitmentsDB>).getL1ToL2LeafValue.mockImplementation((index: bigint) => {
    if (index == leafIndex.toBigInt()) {
      return Promise.resolve(value);
    } else {
      // any indices other than mockAtLeafIndex will return a different value
      // (or undefined if no value is specified for other indices)
      return Promise.resolve(valueAtOtherIndices!);
    }
  });
}

export function mockGetContractInstance(hs: HostStorage, contractInstance: ContractInstanceWithAddress) {
  (hs.contractsDb as jest.Mocked<PublicContractsDB>).getContractInstance.mockResolvedValue(contractInstance);
}
