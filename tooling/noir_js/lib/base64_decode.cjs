"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.base64Decode = void 0;
// Since this is a simple function, we can use feature detection to
// see if we are in the nodeJs environment or the browser environment.
function base64Decode(input) {
    if (typeof Buffer !== 'undefined') {
        // Node.js environment
        return Buffer.from(input, 'base64');
    }
    else if (typeof atob === 'function') {
        // Browser environment
        return Uint8Array.from(atob(input), (c) => c.charCodeAt(0));
    }
    else {
        throw new Error('No implementation found for base64 decoding.');
    }
}
exports.base64Decode = base64Decode;
