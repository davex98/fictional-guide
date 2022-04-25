use fictional_guide::account::AccountsRepository;
use fictional_guide::engine::Engine;
use fictional_guide::parser::Parser;
use fictional_guide::transaction::TransactionLedger;
use std::process;

fn main() {
    let mut args = std::env::args();
    let _prog_name = args.next().expect("USAGE: cargo run");

    let path = args.next().unwrap_or_else(|| {
        println!("provide file path");
        process::exit(1);
    });
    let transactions = Parser::parse(&path).unwrap_or_else(|err| {
        println!("could not parse input: {}", err);
        process::exit(1);
    });
    let mut account_repo = AccountsRepository::default();
    let mut tx_ledger = TransactionLedger::default();
    let mut engine = Engine::new(&mut tx_ledger, &mut account_repo);
    engine.process(&transactions);

    account_repo.display_all().unwrap_or_else(|err| {
        println!("could not display output: {}", err);
        process::exit(1);
    });
}
