# Signing hardware and PKCS#11 profile

**Date:** 2026-08-20
**Wayfinder ticket:** Select signing hardware and PKCS#11 profile
**Status:** Recommendation ready; target qualification still required

## Recommendation

Use seven independently initialized YubiKey 5C NFC tokens for the v1
offline signing profile:

| Role | Tokens | TUF threshold | PIV slot |
|---|---:|---:|---|
| Root custodians | 3 | 2 of 3 | 9c |
| Targets | 1 | 1 of 1 | 9c |
| Snapshot | 1 | 1 of 1 | 9c |
| Timestamp | 1 | 1 of 1 | 9c |
| Cosign release signer | 1 | separate signature | 9c |

Use ECDSA P-256 keys generated on each token, with unique PIN, PUK, and
management credentials. Require PIN and physical touch for every Root,
targets, snapshot, and Cosign signature. Timestamp may use a short cached
touch policy only if the release ceremony threat model and qualification
test explicitly approve it.

Use:

- YKCS11 (`libykcs11.so`) as the PKCS#11 v2.40 module.
- `securesystemslib[hsm]` `HSMSigner` for TUF signing.
- Cosign's native security-key/PIV path for the independent
  `release.json` blob signature.
- `pcscd` and all Python wheels/packages from the signed offline release
  tooling closure.

This profile satisfies the settled custody model without cloning or
exporting private keys. A lost Root token is replaced through a new
dual-threshold Root version signed by the two remaining custodians and
the new key; the threshold is never reduced.

This recommendation is conditional on the qualification checklist below.
The release factory remains blocked until every mandatory test passes.

## Primary-source findings

### YubiKey PIV and YKCS11

YubiKey PIV supports ECC secp256r1 in the standard PIV certificate slots.
Yubico documents slot 9c as the digital-signature slot and requires the
PIN immediately before each signature under its default policy:

- [YubiKey and PIV algorithm support](https://developers.yubico.com/PIV/Introduction/YubiKey_and_PIV.html)
- [PIV certificate slots](https://developers.yubico.com/PIV/Introduction/Certificate_slots.html)

Yubico documents on-device PIV key generation as non-exportable:

- [Generating keys using OpenSSL with YubiKey PIV](https://developers.yubico.com/PIV/Guides/Generating_keys_using_OpenSSL.html)

YKCS11 is Yubico's PKCS#11 v2.40 implementation. It maps object ID 2 to
PIV slot 9c, uses PC/SC on Linux, exposes PIV attestation certificates,
and supports raw ECDSA operations:

- [YKCS11 documentation](https://developers.yubico.com/yubico-piv-tool/YKCS11/)

PIN, PUK, management-key, reset, and key-administration behavior is
documented by Yubico:

- [PIV administrative access](https://developers.yubico.com/PIV/Introduction/Admin_access.html)

YubiKey PIV attestation can prove that a key was generated on a specific
device and records device/firmware and key-policy information:

- [PIV attestation](https://developers.yubico.com/PIV/Introduction/PIV_attestation.html)

YubiKey firmware is not field-upgradeable. Firmware remediation therefore
means replacing and rotating the affected token:

- [YubiKey 5 product documentation](https://www.yubico.com/products/yubikey-5-overview/)

### TUF signing

TUF explicitly supports multiple keys and threshold trust for high-value
roles:

- [TUF specification](https://theupdateframework.github.io/specification/latest/)

`securesystemslib` provides `HSMSigner` through its HSM extra and uses
PKCS#11 for hardware-backed signing:

- [securesystemslib documentation and source](https://github.com/secure-systems-lab/securesystemslib)
- [`HSMSigner` implementation](https://github.com/secure-systems-lab/securesystemslib/blob/main/securesystemslib/signer/_hsm_signer.py)

The implementation maps P-256 to `ecdsa-sha2-nistp256`, defaults to PIV
slot 9c/object ID 2, hashes with SHA-256, invokes `CKM_ECDSA`, and converts
the raw signature into the format expected by TUF.

### Cosign signing

Cosign documents native hardware-security-key operation, PIV slot
selection including `signature` (9c), ECDSA P-256 as the default signing
algorithm, offline blob signing, and disabling transparency-log upload:

- [Cosign sign command](https://github.com/sigstore/cosign/blob/main/doc/cosign_sign.md)
- [Cosign sign-blob command](https://github.com/sigstore/cosign/blob/main/doc/cosign_sign-blob.md)

The exact YubiKey initialization and pre-existing-key behavior must still
be tested against the pinned Cosign build. The documentation establishes
the supported interface, not the complete GZMO ceremony integration.

## Alternatives

### YubiKey 5 FIPS

Acceptable when a FIPS procurement requirement applies, provided the
exact purchased firmware is qualified for P-256 PIV signing, YKCS11,
TUF, and Cosign:

- [YubiKey FIPS series](https://www.yubico.com/products/yubikey-fips/)

### Nitrokey NetHSM 2

NetHSM 2 supports on-device P-256 generation, non-extractable private
keys, public-key export, PKCS#11 v2.40, and separate operator/admin users:

- [NetHSM PKCS#11 key operations](https://docs.nitrokey.com/nethsm/pkcs11-tool)
- [NetHSM PKCS#11 setup](https://docs.nitrokey.com/nethsm/pkcs11-setup)
- [NetHSM product information](https://www.nitrokey.com/products/nethsm)

It is not selected for v1 because it is a network-attached shared
appliance rather than an independently held portable custodian token.
Its encrypted backup model also needs a separate policy ruling under the
no-cloning requirement. Generic PKCS#11 TUF signing is plausible, but
Cosign's documented native security-key path is PIV-oriented; NetHSM
Cosign interoperability must be demonstrated before it can be an
alternative for that role.

### Nitrokey HSM 2 and enterprise HSMs

These are not approved alternatives in this research pass. Nitrokey HSM
2 requires a focused PKCS#11/TUF/Cosign qualification. Enterprise HSM
selection requires access to exact first-party mechanism, custody,
firmware, support, and PKCS#11 documentation for the proposed model.

## Ceremony records

For every initialized token, retain:

- vendor/model, serial number, and firmware;
- assigned role and custodian;
- PIV slot, algorithm, PIN/touch policy, and public key;
- TUF key ID or Cosign public-key ID;
- verified PIV attestation and its certificate chain;
- initialization/qualification tool and package digests;
- token receipt, inspection, initialization, rotation, loss, reset, and
  destruction events.

Never retain PIN, PUK, management key, private key, or reusable unlock
material in the release repository or ceremony record.

## Mandatory qualification before the first Root ceremony

1. Freeze exact token SKU and minimum firmware.
2. Freeze Ubuntu 24.04 packages, `pcscd`, YKCS11 module path/digest,
   Python wheels, `securesystemslib`, TUF implementation, and Cosign.
3. Generate P-256 in slot 9c on each target SKU and verify the private key
   is non-extractable.
4. Verify Yubico PIV attestation, serial, firmware, slot, and touch/PIN
   policy from the offline ceremony environment.
5. Run `HSMSigner.import_()` on two independently built ceremony machines
   and prove identical public key and TUF key ID.
6. Sign and verify representative Root, targets, snapshot, and timestamp
   metadata with the pinned TUF stack.
7. Sign and verify canonical `release.json` with the pinned Cosign build,
   PIV slot 9c, and no transparency/network access.
8. Prove that missing PIN or required touch fails closed.
9. Prove two-token Root threshold success and one-token failure.
10. Rehearse single-Root-token loss and sequential dual-threshold
    replacement without reducing the threshold.
11. Reset a test token and verify that its keys are gone.
12. Capture the entire ceremony with network disabled and verify no tool
    attempts external access.

Failure of any mandatory qualification test blocks the selected profile
and reopens hardware selection. It must not create a software-key
fallback for production signing.

## Decision summary

Select YubiKey 5C NFC plus YKCS11 as the v1 qualification candidate:
three independently held Root tokens and one separate token for each
delegated TUF role and the Cosign signer. The profile becomes approved
only after the pinned offline toolchain passes the complete qualification
matrix above.
