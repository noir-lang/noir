export const randomBytes = (len: number) => {
  const getWebCrypto = () => {
    if (typeof window !== 'undefined' && window.crypto) return window.crypto;
    if (typeof self !== 'undefined' && self.crypto) return self.crypto;
    return undefined;
  };

  const crypto = getWebCrypto();
  if (!crypto) {
    throw new Error('randomBytes UnsupportedEnvironment');
  }

  const buf = new Uint8Array(len);

  // limit of Crypto.getRandomValues()
  // https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
  const MAX_BYTES = 65536;

  if (len > MAX_BYTES) {
    // this is the max bytes crypto.getRandomValues
    // can do at once see https://developer.mozilla.org/en-US/docs/Web/API/window.crypto.getRandomValues
    for (let generated = 0; generated < len; generated += MAX_BYTES) {
      // buffer.slice automatically checks if the end is past the end of
      // the buffer so we don't have to here
      crypto.getRandomValues(buf.subarray(generated, generated + MAX_BYTES));
    }
  } else {
    crypto.getRandomValues(buf);
  }

  return buf;
};
