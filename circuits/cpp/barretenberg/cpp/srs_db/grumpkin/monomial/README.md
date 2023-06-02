# Quick-and-dirty Grumpkin transcript

The Grumpkin transcript currently departs in structure from the BN254
transcript in that:
 - It does not contain a checksum
 - It does not contain any g2 points (indeed, there is no grumpkin::g2).
 - The transcript generation binary only produces a single transcript file.
   If more than 504000 points are desired at some point, it is likely we will 
   need to a small refactor so that more files are created.

A full-length transcript file containing 504000 points would have
BN254    transcript00.dat size: 322560412
Grumpkin transcript00.dat size: 322560028
322560028 - 322560412 = 384 =     256     +    128
                              ^^^^^^^^^^^   ^^^^^^^^
                              2 g2 points   checksum
