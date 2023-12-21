export async function getFile(file_path: string): Promise<ReadableStream<Uint8Array>> {
  const file_url = new URL(file_path, import.meta.url);
  const response = await fetch(file_url);

  if (!response.ok) throw new Error('Network response was not OK');

  return response.body as ReadableStream<Uint8Array>;
}
