import { AcirSimulator } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { KernelProver } from '@aztec/kernel-prover';
import { MemoryDB } from '../database/index.js';
import { KeyStore, TestKeyStore } from '../key_store/index.js';
import { SimulatorOracle } from '../simulator_oracle/index.js';
import { AztecRPCServer } from './aztec_rpc_server.js';

export async function createAztecRPCServer(
  aztecNode: AztecNode,
  {
    keyStore,
    db,
    acirSimulator,
    kernelProver,
  }: {
    keyStore?: KeyStore;
    db?: MemoryDB;
    acirSimulator?: AcirSimulator;
    kernelProver?: KernelProver;
  } = {},
) {
  keyStore = keyStore || new TestKeyStore();
  db = db || new MemoryDB();
  acirSimulator = acirSimulator || new AcirSimulator(new SimulatorOracle(db, keyStore));
  kernelProver = kernelProver || new KernelProver();

  return await Promise.resolve(new AztecRPCServer(keyStore, acirSimulator, kernelProver, aztecNode, db));
}
