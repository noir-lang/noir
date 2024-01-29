import { NodeInfo } from '@aztec/aztec.js';
import { PXE } from '@aztec/circuit-types';

import { MockProxy, mock } from 'jest-mock-extended';

import { checkServerVersion } from './client.js';

describe('client', () => {
  describe('checkServerVersion', () => {
    let pxe: MockProxy<PXE>;

    beforeEach(() => {
      pxe = mock<PXE>();
    });

    it('checks versions match', async () => {
      pxe.getNodeInfo.mockResolvedValue({ nodeVersion: '0.1.0-alpha47' } as NodeInfo);
      await checkServerVersion(pxe, '0.1.0-alpha47');
    });

    it('reports mismatch on older pxe version', async () => {
      pxe.getNodeInfo.mockResolvedValue({ nodeVersion: '0.1.0-alpha47' } as NodeInfo);
      await expect(checkServerVersion(pxe, '0.1.0-alpha48')).rejects.toThrowError(
        /is older than the expected by this CLI/,
      );
    });

    it('reports mismatch on newer pxe version', async () => {
      pxe.getNodeInfo.mockResolvedValue({ nodeVersion: '0.1.0-alpha48' } as NodeInfo);
      await expect(checkServerVersion(pxe, '0.1.0-alpha47')).rejects.toThrowError(
        /is newer than the expected by this CLI/,
      );
    });
  });
});
