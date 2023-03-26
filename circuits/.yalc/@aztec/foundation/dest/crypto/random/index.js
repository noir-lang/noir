import isNode from 'detect-node';
import nodeCrypto from 'crypto';
// limit of Crypto.getRandomValues()
// https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
const MAX_BYTES = 65536;
const getWebCrypto = () => {
    if (typeof window !== 'undefined' && window.crypto)
        return window.crypto;
    if (typeof self !== 'undefined' && self.crypto)
        return self.crypto;
    return undefined;
};
export const randomBytes = (len) => {
    if (isNode) {
        return nodeCrypto.randomBytes(len);
    }
    const crypto = getWebCrypto();
    if (!crypto) {
        throw new Error('randomBytes UnsupportedEnvironment');
    }
    const buf = Buffer.allocUnsafe(len);
    if (len > MAX_BYTES) {
        // this is the max bytes crypto.getRandomValues
        // can do at once see https://developer.mozilla.org/en-US/docs/Web/API/window.crypto.getRandomValues
        for (let generated = 0; generated < len; generated += MAX_BYTES) {
            // buffer.slice automatically checks if the end is past the end of
            // the buffer so we don't have to here
            crypto.getRandomValues(buf.slice(generated, generated + MAX_BYTES));
        }
    }
    else {
        crypto.getRandomValues(buf);
    }
    return buf;
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvY3J5cHRvL3JhbmRvbS9pbmRleC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLE1BQU0sTUFBTSxhQUFhLENBQUM7QUFDakMsT0FBTyxVQUFVLE1BQU0sUUFBUSxDQUFDO0FBRWhDLG9DQUFvQztBQUNwQywwRUFBMEU7QUFDMUUsTUFBTSxTQUFTLEdBQUcsS0FBSyxDQUFDO0FBRXhCLE1BQU0sWUFBWSxHQUFHLEdBQUcsRUFBRTtJQUN4QixJQUFJLE9BQU8sTUFBTSxLQUFLLFdBQVcsSUFBSSxNQUFNLENBQUMsTUFBTTtRQUFFLE9BQU8sTUFBTSxDQUFDLE1BQU0sQ0FBQztJQUN6RSxJQUFJLE9BQU8sSUFBSSxLQUFLLFdBQVcsSUFBSSxJQUFJLENBQUMsTUFBTTtRQUFFLE9BQU8sSUFBSSxDQUFDLE1BQU0sQ0FBQztJQUNuRSxPQUFPLFNBQVMsQ0FBQztBQUNuQixDQUFDLENBQUM7QUFFRixNQUFNLENBQUMsTUFBTSxXQUFXLEdBQUcsQ0FBQyxHQUFXLEVBQUUsRUFBRTtJQUN6QyxJQUFJLE1BQU0sRUFBRTtRQUNWLE9BQU8sVUFBVSxDQUFDLFdBQVcsQ0FBQyxHQUFHLENBQVcsQ0FBQztLQUM5QztJQUVELE1BQU0sTUFBTSxHQUFHLFlBQVksRUFBRSxDQUFDO0lBQzlCLElBQUksQ0FBQyxNQUFNLEVBQUU7UUFDWCxNQUFNLElBQUksS0FBSyxDQUFDLG9DQUFvQyxDQUFDLENBQUM7S0FDdkQ7SUFFRCxNQUFNLEdBQUcsR0FBRyxNQUFNLENBQUMsV0FBVyxDQUFDLEdBQUcsQ0FBQyxDQUFDO0lBQ3BDLElBQUksR0FBRyxHQUFHLFNBQVMsRUFBRTtRQUNuQiwrQ0FBK0M7UUFDL0Msb0dBQW9HO1FBQ3BHLEtBQUssSUFBSSxTQUFTLEdBQUcsQ0FBQyxFQUFFLFNBQVMsR0FBRyxHQUFHLEVBQUUsU0FBUyxJQUFJLFNBQVMsRUFBRTtZQUMvRCxrRUFBa0U7WUFDbEUsc0NBQXNDO1lBQ3RDLE1BQU0sQ0FBQyxlQUFlLENBQUMsR0FBRyxDQUFDLEtBQUssQ0FBQyxTQUFTLEVBQUUsU0FBUyxHQUFHLFNBQVMsQ0FBQyxDQUFDLENBQUM7U0FDckU7S0FDRjtTQUFNO1FBQ0wsTUFBTSxDQUFDLGVBQWUsQ0FBQyxHQUFHLENBQUMsQ0FBQztLQUM3QjtJQUVELE9BQU8sR0FBRyxDQUFDO0FBQ2IsQ0FBQyxDQUFDIn0=