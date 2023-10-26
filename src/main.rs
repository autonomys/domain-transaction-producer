extern crate structopt;
use structopt::StructOpt;

use std::str::FromStr;

#[derive(StructOpt, Debug)]
#[structopt(name = "dtp", about = "Domain Transaction Producer")]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug)]
enum TransactionType {
    LIGHT,
    HEAVY,
}

impl FromStr for TransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "LIGHT" => Ok(TransactionType::LIGHT),
            "HEAVY" => Ok(TransactionType::HEAVY),
            _ => Err(format!("{} is not a valid TransactionType", s)),
        }
    }
}

/// Funder struct
// TODO: might have to transfer the data to a local file or DB.
struct Funder {
    address: String,
}

/// RPC struct
// TODO: might have to transfer the data to a local file or DB.
struct Rpc {
    url: String,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(about = "Send transaction (light or heavy computation)")]
    SendTransaction {
        #[structopt(short, long, help = "Provide Transaction type")]
        types: TransactionType,
    },
    #[structopt(about = "Funder set")]
    SetFunder {
        #[structopt(short, long, help = "Provide funder address")]
        address: String,
    },
    #[structopt(about = "Fund address with amount")]
    Fund {
        #[structopt(short, long, help = "Provide receiver address")]
        receiver: String,
        #[structopt(short, long, help = "Provide amount to be funded")]
        // TODO: convert this to u256 type later
        amount: u128,
    },
    #[structopt(about = "Set RPC Url")]
    SetRpcUrl {
        // TODO: use dotenv to fetch the url from ".env" file as default.
        #[structopt(short, long, default_value = "", help = "Provide RPC Url")]
        url: String,
    },
    #[structopt(about = "View config details")]
    Info {
        #[structopt(short, long)]
        show: bool,
    },
}

fn main() {
    let opt = Cli::from_args();
    let mut funder = Funder {
        address: "0x123".to_string(),
    };

    // TODO: use dotenv to fetch the url from ".env" file as default.
    let mut rpc_url = Rpc {
        url: "https://domain-3.evm.gemini-3f.subspace.network/ws".to_string(),
    };

    match opt.cmd {
        Command::SendTransaction { types } => {
            println!("Hello, {:?}!", types);
            match types {
                TransactionType::LIGHT => {
                    todo!("write the ethers-rs code")
                }
                TransactionType::HEAVY => {
                    todo!("write the ethers-rs code")
                }
            }
        }
        Command::SetFunder { address } => {
            funder.address = address;
        }
        Command::Fund { receiver, amount } => {
            todo!("send TSSC using ethers-rs code")
        }
        Command::SetRpcUrl { url } => {
            rpc_url.url = url;
        }
        Command::Info { show } => {
            if show {
                println!("Funder address: {}", funder.address);
                println!("RPC Url: {}", rpc_url.url);
            }
        }
    }
}
