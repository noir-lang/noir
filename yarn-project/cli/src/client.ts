import { type PXE, createPXEClient } from '@aztec/aztec.js';
import { type DebugLogger } from '@aztec/foundation/log';

import { gtr, ltr, satisfies, valid } from 'semver';

/**
 * Creates a PXE client with a given set of retries on non-server errors.
 * Checks that PXE matches the expected version, and warns if not.
 * @param rpcUrl - URL of the RPC server wrapping the PXE.
 * @param _logger - Debug logger to warn version incompatibilities.
 * @returns A PXE client.
 */
export function createCompatibleClient(rpcUrl: string, _logger: DebugLogger): Promise<PXE> {
  const pxe = createPXEClient(rpcUrl);
  return Promise.resolve(pxe);
}

/** Mismatch between server and client versions. */
class VersionMismatchError extends Error {}

/**
 * Checks that Private eXecution Environment (PXE) version matches the expected one by this CLI. Throws if not.
 * @param pxe - PXE client.
 * @param expectedVersionRange - Expected version by CLI.
 */
export async function checkServerVersion(pxe: PXE, expectedVersionRange: string) {
  const serverName = 'Aztec Node';
  const { nodeVersion } = await pxe.getNodeInfo();
  if (!nodeVersion) {
    throw new VersionMismatchError(`Couldn't determine ${serverName} version. You may run into issues.`);
  }
  if (!nodeVersion || !valid(nodeVersion)) {
    throw new VersionMismatchError(
      `Missing or invalid version identifier for ${serverName} (${nodeVersion ?? 'empty'}).`,
    );
  } else if (!satisfies(nodeVersion, expectedVersionRange)) {
    if (gtr(nodeVersion, expectedVersionRange)) {
      throw new VersionMismatchError(
        `${serverName} is running version ${nodeVersion} which is newer than the expected by this CLI (${expectedVersionRange}). Consider upgrading your CLI to a newer version.`,
      );
    } else if (ltr(nodeVersion, expectedVersionRange)) {
      throw new VersionMismatchError(
        `${serverName} is running version ${nodeVersion} which is older than the expected by this CLI (${expectedVersionRange}). Consider upgrading your ${serverName} to a newer version.`,
      );
    } else {
      throw new VersionMismatchError(
        `${serverName} is running version ${nodeVersion} which does not match the expected by this CLI (${expectedVersionRange}).`,
      );
    }
  }
}
