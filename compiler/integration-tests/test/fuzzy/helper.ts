export function getRandomString(length: number): string {
    const randomChars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += randomChars.charAt(Math.floor(Math.random() * randomChars.length));
    }
    return result;
}

export function getRandomNumber(length: number): number {
    const min = Math.pow(10, length - 1);
    const max = Math.pow(10, length) - 1;
    return Math.floor(Math.random() * (max - min + 1) + min);
}

export function getRandomUint8Array(length: number): Uint8Array {
    const result = new Uint8Array(length);
    for (let i = 0; i < length; i++) {
        result[i] = Math.floor(Math.random() * 256);
    }
    return result;
}

export function getRandomMap(length: number): Map<number, string> {
    const randomChars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    const resultMap = new Map<number, string>();

    for (let i = 0; i < length; i++) {
        const key = i;
        const valueLength = Math.floor(Math.random() * 10) + 1;
        let value = '';

        for (let j = 0; j < valueLength; j++) {
            value += randomChars.charAt(Math.floor(Math.random() * randomChars.length));
        }

        resultMap.set(key, value);
    }

    return resultMap;
}
