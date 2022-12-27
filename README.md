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
    "launch_measurement": "3c6f91219614a28d2e193e82dc2366d1a758a52c04607999b5b8ff9216304c97"
}
```

The `passphrase` is the LUKS2 passphrase to your encrypted disk image. 
Obviously you need to keep your WORKLOAD secret :).

### WORKLOAD Theft

If your TEE host gets hacked, and the following resouces get pinched:

1. Your LUKS2 encrypted disk image
2. Your `workload_id`
3. Your `libkrunfw-sev.so`

It is possible to for the hacker to start your workload on a different AMD SEV/SNP CPUs too if
your Attestation Server is open. 

A simple mitigation is too only allow remote attestation requests from the a specific IP.

In the future, I'm planning to pin each WORKLOAD to a Chip Endorsement Key (CEK) so that it 
cannot be moved to a different AMD SEV/SNP CPU without updating the WORKLOAD at the
Attestation Server.
