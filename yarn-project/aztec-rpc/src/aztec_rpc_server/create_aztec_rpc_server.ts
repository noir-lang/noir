import { AcirSimulator } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { KernelProver } from '@aztec/kernel-prover';
import { MemoryDB } from '../database/index.js';
import { KeyStore, TestKeyStore } from '../key_store/index.js';
import { Synchroniser } from '../synchroniser/index.js';
import { AztecRPCServer } from './aztec_rpc_server.js';

export async function createAztecRPCServer(
  aztecNode: AztecNode,
  {
    keyStore,
    db,
    synchroniser,
    acirSimulator,
    kernelProver,
  }: {
    keyStore?: KeyStore;
    db?: MemoryDB;
    synchroniser?: Synchroniser;
    acirSimulator?: AcirSimulator;
    kernelProver?: KernelProver;
  } = {},
) {
  keyStore = keyStore || new TestKeyStore();
  db = db || new MemoryDB();
  synchroniser = synchroniser || new Synchroniser(aztecNode, db);
  acirSimulator = acirSimulator || new AcirSimulator();
  kernelProver = kernelProver || new KernelProver();

  return await Promise.resolve(new AztecRPCServer(keyStore, synchroniser, acirSimulator, kernelProver, aztecNode, db));
}
