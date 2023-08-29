import { AztecRPC, NodeInfo } from '@aztec/types';

import { MockProxy, mock } from 'jest-mock-extended';

import { checkServerVersion } from './client.js';

describe('client', () => {
  describe('checkServerVersion', () => {
    let rpc: MockProxy<AztecRPC>;

    beforeEach(() => {
      rpc = mock<AztecRPC>();
    });

    it('checks versions match', async () => {
      rpc.getNodeInfo.mockResolvedValue({ client: 'rpc@0.1.0-alpha47' } as NodeInfo);
      await checkServerVersion(rpc, '0.1.0-alpha47');
    });

    it('reports mismatch on older rpc version', async () => {
      rpc.getNodeInfo.mockResolvedValue({ client: 'rpc@0.1.0-alpha47' } as NodeInfo);
      await expect(checkServerVersion(rpc, '0.1.0-alpha48')).rejects.toThrowError(
        /is older than the expected by this CLI/,
      );
    });

    it('reports mismatch on newer rpc version', async () => {
      rpc.getNodeInfo.mockResolvedValue({ client: 'rpc@0.1.0-alpha48' } as NodeInfo);
      await expect(checkServerVersion(rpc, '0.1.0-alpha47')).rejects.toThrowError(
        /is newer than the expected by this CLI/,
      );
    });
  });
});
