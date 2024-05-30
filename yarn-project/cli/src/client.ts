import { type PXE, createPXEClient } from '@aztec/aztec.js';
import { type DebugLogger } from '@aztec/foundation/log';
import { fileURLToPath } from '@aztec/foundation/url';

import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { gtr, ltr, satisfies, valid } from 'semver';

/**
 * Creates a PXE client with a given set of retries on non-server errors.
 * Checks that PXE matches the expected version, and warns if not.
 * @param rpcUrl - URL of the RPC server wrapping the PXE.
 * @param logger - Debug logger to warn version incompatibilities.
 * @returns A PXE client.
 */
export async function createCompatibleClient(rpcUrl: string, logger: DebugLogger) {
  const pxe = createPXEClient(rpcUrl);
  const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../package.json');
  const packageJsonContents = JSON.parse(readFileSync(packageJsonPath).toString());
  const expectedVersionRange = packageJsonContents.version;

  try {
    await checkServerVersion(pxe, expectedVersionRange);
  } catch (err) {
    if (err instanceof VersionMismatchError) {
      logger.warn(err.message);
    } else {
      throw err;
    }
  }

  return pxe;
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
