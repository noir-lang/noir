# NOTE: This structure is a bit outdated now as the transcript is flat but still a useful reference.
Copied from https://github.com/AztecProtocol/ignition-verification/blob/master/Transcript_spec.md.
### Transcript files

Our setup produces 100,800,000 G1 points and 1 G2 point, over the BN254 curve.

This is split into multiple 'transcript' files. Each transcript file contains *5,040,000* G1 points. The first transcript file also contains 2 G2 point.

These points are not compressed.

### Data format

The transcript file contains raw binary data, where data elements are located by knowing their precise byte-position in the file. We write our data as follows:

For big-integer numbers (g1, g2 coordinates), we describe each 256-bit field element as a uint64_t[4] array. The first entry is the least significant word of the field element. Each 'word' is written in big-endian form.

For other integers (in the manifest section), variables are directly written in big-endian form.

Example Python decoder from Vitalik

```
def decode(chunk): return sum([int.from_bytes(chunk[i:i+8], 'big') * 256**i for i in range(0, 32, 8)])
```

### Structure of a transcript file

The transcript file contains 4 linear data sections

| section number | section size (bytes) | description |
| --- | --- | --- |
| 1 | 28 | A 'manifest' containing metadata |
| 2 | 322,560,000 | 5,040,000 uncompressed G1 points |
| 3 | 256 or 0 | (first transcript only) Two uncompressed G2 points |
| 4 | 64 | The 'checksum' - a BLAKE2B hash of the rest of the file's data |

### The G2 points

The first G2 point is `x.[2]` where `x` is the trusted setup toxic waste.
The second G2 point is `z.[2]`, where `z` is the toxic waste from the previous participant. It is used, in combination with the previous participant's transcript, to check that the current transcript was built off of the previous participant's transcript

### Naming scheme

The transcript input files are called 'transcript0.dat', 'transcript1.dat', ..., 'transcript19.dat'

The transcript output files are called 'transcript0_out.dat', ..., 'transcript19_out.dat'

### The manifest structure:
# NOTE: this is a historical document and the exact transcript structure is outdated
# Copied from https://github.com/AztecProtocol/ignition-verification/blob/master/Transcript_spec.md
The manifest is 28 bytes of data with the following structure

| byte index | description |
| --- | --- |
|0-3 | transcript number (starting from 0) |
|4-7 | total number of transcripts (should be 20) |
|8-11 | total number of G1 points in all transcripts (should be 100,000,000) |
|12-15 | total number of G2 points in all transcripts (should be 1) |
|16-19 | number of G1 points in this transcript (should be 5,000,000) |
|20-23 | number of G2 points in this transcript (2 for 1st transcript, 0 for the rest) |
|24-27 | 'start-from', the index of the 1st G1 point in this transcript |

Regarding start-from: the value will be 0 in transcript0.dat, 5,000,000 in transcript1.dat, 95,000,000 in transcript19.dat etc  

We have a bit of a continuity error, where for the first transcript, the 'local' number of G2 points is 2, when the 'total' number is 1. In the former, we're including the G2 element created by the participant, to verify transcripts. In the latter, we're referring to the total number of G2 elements in the structured reference string we're producing.

### G1 point structure

The first G1 point will be `x.[1]`, where `x` is the trusted setup toxic waste, and `[1]` is the bn254 G1 generator point (1, 2)

Structure is as follows: `x.[1]`, `x^{2}.[1]`, ..., `x^{100,800,000}.[1]`  

Each participant generates their own randomness `z` and exponentiates each point by `z^{i}`, where `i` is the G1 point index

### G2 point structure

The only G2 point is `x.[2]`, where `[2]` is the bn254 G2 generator point with coordinates:

```
{
    "x": {
        "c0": "10857046999023057135944570762232829481370756359578518086990519993285655852781",
        "c1": "11559732032986387107991004021392285783925812861821192530917403151452391805634"
    },
    "y": {
        "c0": "8495653923123431417604973247489272438418190587263600148770280649306958101930",
        "c1": "4082367875863433681332203403145435568316851327593401208105741076214120093531"
    }
}
```
