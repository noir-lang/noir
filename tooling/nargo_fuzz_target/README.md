# `nargo_fuzz_target`

To run:

```bash
cargo afl build
cargo afl fuzz -i inputs -o outputs ../../target/debug/nargo_fuzz_target
```

TODO
## Ramdisk

```bash
# make dir
❯ sudo mkdir /tmp/nargo_fuzz_ramdisk

# make accessible to other users
❯ sudo chmod 777 /tmp/nargo_fuzz_ramdisk

# now, the nargo binary is ~400MB
❯ ls -lah ../../target/debug/nargo_fuzz_target
-rwxrwxr-x 2 michael michael 398M May  1 11:32 ../../target/debug/nargo_fuzz_target

# so we round up to the next power of 2:
❯ sudo mount -t tmpfs -o size=512m nargo_fuzz_ramdisk /tmp/nargo_fuzz_ramdisk

# check for successful mount
❯ mount | tail -n 1
nargo_fuzz_ramdisk on /tmp/nargo_fuzz_ramdisk type tmpfs (rw,relatime,size=524288k,inode64)

# copy inputs/outputs to the ramdisk
❯ ..

# CLEANUP
# delete the ramdisk
sudo umount /tmp/nargo_fuzz_ramdisk/
```

