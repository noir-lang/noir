import { resolve } from 'dns/promises';

/**
 * Converts an address string to a multiaddr string.
 * Example usage:
 * const tcpAddr = '123.456.7.8:80' -> /ip4/123.456.7.8/tcp/80
 * const udpAddr = '[2001:db8::1]:8080' -> /ip6/2001:db8::1/udp/8080
 * @param address - The address string to convert. Has to be in the format <addr>:<port>.
 * @param protocol - The protocol to use in the multiaddr string.
 * @returns A multiaddr compliant string.
 */
export function convertToMultiaddr(address: string, protocol: 'tcp' | 'udp'): string {
  const [addr, port] = splitAddressPort(address, false);

  const multiaddrPrefix = addressToMultiAddressType(addr);
  if (multiaddrPrefix === 'dns') {
    throw new Error('Invalid address format. Expected an IPv4 or IPv6 address.');
  }

  return `/${multiaddrPrefix}/${addr}/${protocol}/${port}`;
}

/**
 * Splits an <address>:<port> string into its components.
 * @returns The ip6 or ip4 address & port separately
 */
export function splitAddressPort(address: string, allowEmptyAddress: boolean): [string, string] {
  let addr: string;
  let port: string;

  if (address.startsWith('[')) {
    // IPv6 address enclosed in square brackets
    const match = address.match(/^\[([^\]]+)\]:(\d+)$/);
    if (!match) {
      throw new Error(`Invalid IPv6 address format:${address}. Expected format: [<addr>]:<port>`);
    }
    [, addr, port] = match;
  } else {
    // IPv4 address
    [addr, port] = address.split(':');
    if ((!addr && !allowEmptyAddress) || !port) {
      throw new Error(`Invalid address format: ${address}. Expected format: <addr>:<port>`);
    }
  }

  return [addr, port];
}

/**
 * Queries the public IP address of the machine.
 */
export async function getPublicIp(): Promise<string> {
  const resp = await fetch('http://checkip.amazonaws.com/');
  const text = await resp.text();
  return text.trim();
}

export async function resolveAddressIfNecessary(address: string): Promise<string> {
  const [addr, port] = splitAddressPort(address, false);
  const multiaddrPrefix = addressToMultiAddressType(addr);
  if (multiaddrPrefix === 'dns') {
    const resolvedAddresses = await resolve(addr);
    if (resolvedAddresses.length === 0) {
      throw new Error(`Could not resolve address: ${addr}`);
    }
    return `${resolvedAddresses[0]}:${port}`;
  } else {
    return address;
  }
}

// Not public because it is not used outside of this file.
// Plus, it relies on `splitAddressPort` being called on the address first.
function addressToMultiAddressType(address: string): 'ip4' | 'ip6' | 'dns' {
  if (address.includes(':')) {
    return 'ip6';
  } else if (address.match(/^[\d.]+$/)) {
    return 'ip4';
  } else {
    return 'dns';
  }
}
