# Domain Transaction Producer

Domain Transaction Producer for Subspace EVM Network.

## Usage

```sh
$ cargo run -- --help
dtp 0.1.0
Domain Transaction Producer

USAGE:
    dtp [OPTIONS] --funding-amount <funding-amount> --initial-funded-account-private-key <initial-funded-account-private-key> --num-accounts <num-accounts> --rpc-url <rpc-url> --transaction-type <transaction-type>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --funding-amount <funding-amount>                                            Funding amount
    -k, --initial-funded-account-private-key <initial-funded-account-private-key>    Initial funded account private key
    -a, --num-accounts <num-accounts>                                                Number of accounts
    -b, --num-blocks <num-blocks>                                                    Number of blocks to run for
    -r, --rpc-url <rpc-url>                                                          Subspace EVM (Nova) RPC node URL
    -t, --transaction-type <transaction-type>                                        Transaction type: light or heavy
```

### Examples

> Activate the environment variables set in `.env` file. So, copy the `.env.example` file to `.env` and set the environment variables.

```sh
$ source .env
```

### Without setting any number of blocks as parameter

In short,

```sh

❯ cargo run -- -f 1000 -k $FUNDER_PRIVATE_KEY -a 3 -t light -r $SUBSPACE_EVM_RPC_URL
```

```sh
$ cargo run -- --funding-amount 1000 --initial-funded-account-private-key $FUNDER_PRIVATE_KEY --num-accounts 3 --transaction-type light --rpc-url $SUBSPACE_EVM_RPC_URL
```

### With number of blocks as parameter

In short,

```sh

❯ cargo run -- -f 1000 -k $FUNDER_PRIVATE_KEY -a 3 -t light -b 3 -r $SUBSPACE_EVM_RPC_URL
```

```sh
$ cargo run -- --funding-amount 1000 --initial-funded-account-private-key $FUNDER_PRIVATE_KEY --num-accounts 3 --transaction-type light --num-blocks 3  --rpc-url $SUBSPACE_EVM_RPC_URL
```
