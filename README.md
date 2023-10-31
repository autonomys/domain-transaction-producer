# Domain Transaction Producer

A CLI tool to produce domain transactions for Subspace EVM Network.

## [DTP CLI](./dtp)

## [Subspace EVM Playground](./contracts/)

## [Subspace EVM Contracts Bindings](./bindings/)

### Generating Rust bindings to the contracts

Before following this, make sure you have the `forge` CLI installed.

Rust bindings to the contracts can be generated via `forge bind`, which requires
first building your contracts:

```sh
# Build the contracts
$ forge build --root ./contracts
# Generate the bindings to the contracts
$ forge bind --bindings-path ./bindings --root ./contracts --crate-name bindings
```

Any follow-on calls to `forge bind` will check that the generated bindings match
the ones under the build files. If you want to re-generate your bindings, pass
the `--overwrite` flag to your `forge bind` command.
