# Domain Transaction Producer

Domain Transaction Producer for Subspace EVM Network

## Usage

```sh
$ domain-transaction-producer
dtp 0.1.0
Domain Transaction Producer

USAGE:
    domain-transaction-producer [OPTIONS] --funding-amount <funding-amount> --initial-funded-account-private-key <initial-funded-account-private-key> --num-accounts <num-accounts> --rpc-url <rpc-url> --transaction-type <transaction-type>

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
