export async function getPublicIp(): Promise<string> {
  const resp = await fetch('http://checkip.amazonaws.com/');
  const text = await resp.text();
  return text.trim();
}
