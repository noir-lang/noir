import { type LogFn } from '@aztec/foundation/log';

import { createSecp256k1PeerId } from '@libp2p/peer-id-factory';

export async function generateP2PPrivateKey(log: LogFn) {
  const peerId = await createSecp256k1PeerId();
  const exportedPeerId = Buffer.from(peerId.privateKey!).toString('hex');
  log(`Private key: ${exportedPeerId}`);
  log(`Peer Id: ${peerId}`);
}
