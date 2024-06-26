import { type ContractArtifact } from '@aztec/aztec.js';
import { type ExtendedNote, NoteStatus, type PXE, type TxHash } from '@aztec/circuit-types';
import { type AztecAddress, type Fr } from '@aztec/circuits.js';
import { siloNullifier } from '@aztec/circuits.js/hash';
import { type LogFn } from '@aztec/foundation/log';
import { toHumanReadable } from '@aztec/foundation/serialize';
import { ClassRegistererAddress } from '@aztec/protocol-contracts/class-registerer';
import { InstanceDeployerAddress } from '@aztec/protocol-contracts/instance-deployer';

export async function inspectBlock(pxe: PXE, blockNumber: number, log: LogFn, opts: { showTxs?: boolean } = {}) {
  const block = await pxe.getBlock(blockNumber);
  if (!block) {
    log(`No block found for block number ${blockNumber}`);
    return;
  }

  log(`Block ${blockNumber} (${block.hash().toString()})`);
  log(` Total fees: ${block.header.totalFees.toBigInt()}`);
  log(
    ` Fee per gas unit: DA=${block.header.globalVariables.gasFees.feePerDaGas.toBigInt()} L2=${block.header.globalVariables.gasFees.feePerL2Gas.toBigInt()}`,
  );
  log(` Coinbase: ${block.header.globalVariables.coinbase}`);
  log(` Fee recipient: ${block.header.globalVariables.feeRecipient}`);
  log(` Timestamp: ${new Date(block.header.globalVariables.timestamp.toNumber() * 500)}`);
  if (opts.showTxs) {
    log(``);
    const artifactMap = await getKnownArtifacts(pxe);
    for (const txHash of block.body.txEffects.map(tx => tx.txHash)) {
      await inspectTx(pxe, txHash, log, { includeBlockInfo: false, artifactMap });
    }
  } else {
    log(` Transactions: ${block.body.txEffects.length}`);
  }
}

export async function inspectTx(
  pxe: PXE,
  txHash: TxHash,
  log: LogFn,
  opts: { includeBlockInfo?: boolean; artifactMap?: ArtifactMap } = {},
) {
  const [receipt, effects, notes] = await Promise.all([
    pxe.getTxReceipt(txHash),
    pxe.getTxEffect(txHash),
    pxe.getIncomingNotes({ txHash, status: NoteStatus.ACTIVE_OR_NULLIFIED }),
  ]);

  if (!receipt || !effects) {
    log(`No receipt or effects found for transaction hash ${txHash.toString()}`);
    return;
  }

  const artifactMap = opts?.artifactMap ?? (await getKnownArtifacts(pxe));

  // Base tx data
  log(`Tx ${txHash.toString()}`);
  if (opts.includeBlockInfo) {
    log(` Block: ${receipt.blockNumber} (${receipt.blockHash?.toString('hex')})`);
  }
  log(` Status: ${receipt.status} (${effects.revertCode.getDescription()})`);
  if (receipt.error) {
    log(` Error: ${receipt.error}`);
  }
  if (receipt.transactionFee) {
    log(` Fee: ${receipt.transactionFee.toString()}`);
  }

  // Unencrypted logs
  const unencryptedLogs = effects.unencryptedLogs.unrollLogs();
  if (unencryptedLogs.length > 0) {
    log(' Logs:');
    for (const unencryptedLog of unencryptedLogs) {
      const data = toHumanReadable(unencryptedLog.data, 1000);
      log(`  ${toFriendlyAddress(unencryptedLog.contractAddress, artifactMap)}: ${data}`);
    }
  }

  // Public data writes
  const writes = effects.publicDataWrites;
  if (writes.length > 0) {
    log(' Public data writes:');
    for (const write of writes) {
      log(`  Leaf ${write.leafIndex.toString()} = ${write.newValue.toString()}`);
    }
  }

  // Created notes
  const noteEncryptedLogsCount = effects.noteEncryptedLogs.unrollLogs().length;
  if (noteEncryptedLogsCount > 0) {
    log(' Created notes:');
    const notVisibleNotes = noteEncryptedLogsCount - notes.length;
    if (notVisibleNotes > 0) {
      log(`  ${notVisibleNotes} notes not visible in the PXE`);
    }
    for (const note of notes) {
      inspectNote(note, artifactMap, log);
    }
  }

  // Nullifiers
  const nullifierCount = effects.nullifiers.length;
  const { deployNullifiers, initNullifiers, classNullifiers } = await getKnownNullifiers(pxe, artifactMap);
  if (nullifierCount > 0) {
    log(' Nullifiers:');
    for (const nullifier of effects.nullifiers) {
      const [note] = await pxe.getIncomingNotes({ siloedNullifier: nullifier });
      const deployed = deployNullifiers[nullifier.toString()];
      const initialized = initNullifiers[nullifier.toString()];
      const registered = classNullifiers[nullifier.toString()];
      if (nullifier.toBuffer().equals(txHash.toBuffer())) {
        log(`  Transaction hash nullifier ${nullifier.toShortString()}`);
      } else if (note) {
        inspectNote(note, artifactMap, log, `Nullifier ${nullifier.toShortString()} for note`);
      } else if (deployed) {
        log(
          `  Contract ${toFriendlyAddress(deployed, artifactMap)} deployed via nullifier ${nullifier.toShortString()}`,
        );
      } else if (initialized) {
        log(
          `  Contract ${toFriendlyAddress(
            initialized,
            artifactMap,
          )} initialized via nullifier ${nullifier.toShortString()}`,
        );
      } else if (registered) {
        log(`  Class ${registered} registered via nullifier ${nullifier.toShortString()}`);
      } else {
        log(`  Unknown nullifier ${nullifier.toString()}`);
      }
    }
  }

  // L2 to L1 messages
  if (effects.l2ToL1Msgs.length > 0) {
    log(` L2 to L1 messages:`);
    for (const msg of effects.l2ToL1Msgs) {
      log(`  ${msg.toString()}`);
    }
  }
}

function inspectNote(note: ExtendedNote, artifactMap: ArtifactMap, log: LogFn, text = 'Note') {
  const artifact = artifactMap[note.contractAddress.toString()];
  const contract = artifact?.name ?? note.contractAddress.toString();
  const type = artifact?.notes[note.noteTypeId.toString()]?.typ ?? note.noteTypeId.toField().toShortString();
  log(`  ${text} type ${type} at ${contract}`);
  log(`    Owner: ${toFriendlyAddress(note.owner, artifactMap)}`);
  for (const field of note.note.items) {
    log(`    ${field.toString()}`);
  }
}

function toFriendlyAddress(address: AztecAddress, artifactMap: ArtifactMap) {
  const artifact = artifactMap[address.toString()];
  if (!artifact) {
    return address.toString();
  }

  return `${artifact.name}<${address.toString()}>`;
}

async function getKnownNullifiers(pxe: PXE, artifactMap: ArtifactMap) {
  const knownContracts = await pxe.getContracts();
  const deployerAddress = InstanceDeployerAddress;
  const registererAddress = ClassRegistererAddress;
  const initNullifiers: Record<string, AztecAddress> = {};
  const deployNullifiers: Record<string, AztecAddress> = {};
  const classNullifiers: Record<string, string> = {};
  for (const contract of knownContracts) {
    initNullifiers[siloNullifier(contract, contract).toString()] = contract;
    deployNullifiers[siloNullifier(deployerAddress, contract).toString()] = contract;
  }
  for (const artifact of Object.values(artifactMap)) {
    classNullifiers[
      siloNullifier(registererAddress, artifact.classId).toString()
    ] = `${artifact.name}Class<${artifact.classId}>`;
  }
  return { initNullifiers, deployNullifiers, classNullifiers };
}

type ArtifactMap = Record<string, ContractArtifactWithClassId>;
type ContractArtifactWithClassId = ContractArtifact & { classId: Fr };
async function getKnownArtifacts(pxe: PXE): Promise<ArtifactMap> {
  const knownContractAddresses = await pxe.getContracts();
  const knownContracts = await Promise.all(knownContractAddresses.map(contract => pxe.getContractInstance(contract)));
  const classIds = [...new Set(knownContracts.map(contract => contract?.contractClassId))];
  const knownArtifacts = await Promise.all(
    classIds.map(classId =>
      classId ? pxe.getContractArtifact(classId).then(a => (a ? { ...a, classId } : undefined)) : undefined,
    ),
  );
  const map: Record<string, ContractArtifactWithClassId> = {};
  for (const instance of knownContracts) {
    if (instance) {
      const artifact = knownArtifacts.find(a => a?.classId.equals(instance.contractClassId));
      if (artifact) {
        map[instance.address.toString()] = artifact;
      }
    }
  }
  return map;
}
