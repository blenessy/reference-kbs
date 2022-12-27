# reference-kbs

A reference implementation of the [KBS](https://github.com/confidential-containers/kbs/).

## Attestation Server

While the [KBS](https://github.com/confidential-containers/kbs/) doesn't define the location of the component that does the attestation of the TEE measurements, allowing it to be a separate component from KBS, in this implementation that functionality is provided locally.

## Hacks by bleness

My use case involves running only 1 workload per attestation server and I want to
statically define the (secret) workload when the attestation server starts,
through an environment variable. This is why the diesel (sqlite3) dependency got removed.

## WORKLOAD security

Here is a valid `WORKLOAD`:

```json
{
    "workload_id": "sevtest",
    "tee_config": "{\"flags\":{\"bits\":63},\"minfw\":{\"major\":0,\"minor\":0}}",
    "passphrase": "mysecretpassphrase",
    "launch_measurement": "3c6f91219614a28d2e193e82dc2366d1a758a52c04607999b5b8ff9216304c97",
    "affinity": ["CEK EP384 E256 230f44f6705d3a0ab10edeac5aff6706713beaf3bec433d9d097b5f4c12cf5e3"]
}
```

The `passphrase` is the LUKS2 passphrase to your encrypted disk image. 
Obviously you need to keep your WORKLOAD secret :).

The `affinity` array contains CEKs (Chip Endorsement Keys) thar are
allowed to execute the remote attestation protocol. When empty all CEKs are allowed.
