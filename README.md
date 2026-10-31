# SigmaOS-kernel

SigmaOS microkernel core: scheduler, VFS, syscall gate, and memory management.

## Development

This repository is the kernel-core component in the SigmaOS multi-repository
ecosystem. Keep public interfaces small, document safety invariants, and attach
test evidence to changes.

```bash
cargo check
./run_sigma_tests.sh
```

See the central [SigmaOS repository](https://github.com/AaryanSinghChauhan09/SigmaOS)
for integration status and system-wide development policy.

The included Secure Boot verification model has no cryptographic backend; it
reports signed inputs as `VerifierUnavailable` and must not be used as a trust
boundary.
