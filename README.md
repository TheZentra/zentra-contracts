# Zentra Protocol

This repository contains the smart contracts for an implementation of the Zentra Protocol. Zentra is a payment streaming protocol built on the Stellar Soroban Blockchain network, enabling seamless and continuous payment streams between parties.

## Documentation

To learn more about the Zentra Protocol and how to integrate it into your applications, visit the documentation:

- [Zentra Docs](https://docs.thezentra.com/)

## Audits

At present, no audits have been performed for the protocol. Outcomes will be provided upon completion of an audit.

## Getting Started

To build the contracts, follow these steps:

```bash
make
```

To run all unit tests and the integration test suite, use the following command:

```bash
make test
```

## Deployment

Running the make command generates both optimized and unoptimized versions of the WASM contracts. It is advisable to deploy the optimized version to the network for better performance.

You can find the contracts at the following path:

``` bash
target/wasm32-unknown-unknown/optimized
```

For assistance with deployment to the network, refer to the [Zentra Utils](https://github.com/the-zentra/zentra-utils) repository.

## Contributing

Notes for contributors:

- Ensure that the "overflow-checks" flag is not removed, as it maintains the safety of contract math operations.

## Community Links

Here are some community links related to Zentra:

- [Zentra Discord](https://discord.com)
