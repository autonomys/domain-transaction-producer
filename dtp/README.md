# Domain Transaction Producer

Domain Transaction Producer for Subspace EVM Network.

## Usage

> replace `cargo run --` with `dtp` by installing the binary using `cargo install --path -p dtp .`

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

In long,

```sh
$ cargo run -- --funding-amount 1000 --initial-funded-account-private-key $FUNDER_PRIVATE_KEY --num-accounts 3 --transaction-type light --rpc-url $SUBSPACE_EVM_RPC_URL
```

### With number of blocks as parameter

In short,

```sh
❯ cargo run -- -f 1000 -k $FUNDER_PRIVATE_KEY -a 3 -t light -b 3 -r $SUBSPACE_EVM_RPC_URL
```

In long,

```sh
$ cargo run -- --funding-amount 1000 --initial-funded-account-private-key $FUNDER_PRIVATE_KEY --num-accounts 3 --transaction-type light --num-blocks 3  --rpc-url $SUBSPACE_EVM_RPC_URL
```

### With accounts funded sufficiently so that they can send light transactions

> Here, light transactions mean calling `Counter:increment` function to increment the counter.

In long,

```sh
$ cargo run -p dtp -- \
--funding-amount 441000000000000 \
--initial-funded-account-private-key $FUNDER_PRIVATE_KEY \
--num-accounts 6 \
--rpc-url $SUBSPACE_EVM_RPC_URL \
--transaction-type light
```

Observations:

The light transactions signed by each account are awaited all at once using `join_all` function. This is the how some light txs are added in the same block indexed differently.

```sh
Number stored in 'Counter' before calls: 13

'0x2782…da06' increment number, which incurred a gas fee of '0.00007928100052854 TSSC', has a tx hash: '0x0cc998c362df131ace165321c6e9e8fa5daa41933929402f0e736bc3f962fbc4', indexed at #2 in block #52420.

'0xcb54…51c4' increment number, which incurred a gas fee of '0.00007928100052854 TSSC', has a tx hash: '0x995df338a4792e619a7c731fcb6d67a9f0a09a239144c317651e8fc521d0e8c6', indexed at #3 in block #52420.

'0x7a8c…a2d7' increment number, which incurred a gas fee of '0.00007928100052854 TSSC', has a tx hash: '0xf6e61f7678b6d58b727b8f0bf41f4e0ebc0449cdb7b075ad50e08e6bbc314da0', indexed at #0 in block #52420.

'0xbda5…789b' increment number, which incurred a gas fee of '0.00007928100052854 TSSC', has a tx hash: '0x457b65d0b8bac6e3c6c9d5d69ffdd1d6ae8e591decf7d763d96f86e644b99c20', indexed at #1 in block #52420.

'0x718a…f847' increment number, which incurred a gas fee of '0.00007928100052854 TSSC', has a tx hash: '0xe77f8e5ddfddc651c1b769305c73e0cf282121d6e489bb94533fb55ed61443a1', indexed at #4 in block #52420.

'0x1653…f1c4' increment number, which incurred a gas fee of '0.00007928100052854 TSSC', has a tx hash: '0xdd9e01509f3ce9ec148b76087222fd88c180e1f99d3b773badfe83ce9a77ae71', indexed at #0 in block #52421.

Number stored in 'Counter' after 6 calls: 19
```
