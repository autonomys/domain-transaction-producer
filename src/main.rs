use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "dtp", about = "Domain Transaction Producer")]
struct Cli {
    /// Number of accounts
    #[structopt(short = "a", long)]
    num_accounts: u32,

    /// Transaction type: light or heavy
    #[structopt(short = "t", long)]
    transaction_type: String,

    /// Number of blocks to run for
    #[structopt(short = "b", long)]
    num_blocks: Option<u32>,

    /// Initial funded account private key
    #[structopt(short = "k", long)]
    initial_funded_account_private_key: String,

    /// Funding amount
    #[structopt(short = "f", long)]
    funding_amount: u64,

    /// Ethereum RPC node URL
    #[structopt(short = "r", long)]
    rpc_url: String,
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

fn main() {
    let opt = Cli::from_args();

    match opt.transaction_type.parse::<TransactionType>() {
        Ok(transaction_type) => match opt.num_blocks {
            Some(num_blocks) => {
                println!("Running {} transaction(s) of type {:?} for {} block(s) with {} account(s) and funding amount of {} wei using initial funded account with private key {} and Ethereum RPC node URL {}", 
                    opt.num_accounts, transaction_type, num_blocks, opt.num_accounts, opt.funding_amount, opt.initial_funded_account_private_key, opt.rpc_url);
            }
            None => {
                println!("Running {} transaction(s) of type {:?} indefinitely with {} account(s) and funding amount of {} wei using initial funded account with private key {} and Ethereum RPC node URL {}", 
                    opt.num_accounts, transaction_type, opt.num_accounts, opt.funding_amount, opt.initial_funded_account_private_key, opt.rpc_url);
            }
        },
        Err(e) => {
            println!("{}", e);
        }
    }
}
