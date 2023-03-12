import pbkdf2Lib from 'pbkdf2';

export function pbkdf2(password: Buffer, salt: Buffer, iterations: number, dklen: number): Promise<Buffer> {
  return new Promise<Buffer>((resolve, reject) => {
    pbkdf2Lib.pbkdf2(password, salt, iterations, dklen, 'sha256', (err, result) => {
      if (err) {
        reject(err);
      } else {
        resolve(result);
      }
    });
  });
}

export function pbkdf2Sync(password: Buffer, salt: Buffer, iterations: number, dklen: number) {
  return pbkdf2Lib.pbkdf2Sync(password, salt, iterations, dklen, 'sha256');
}
