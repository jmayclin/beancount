use beancount_parser::parse;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args().nth(1).ok_or("filename argument")?;
    let unparsed_file = std::fs::read_to_string(filename)?;

    let ledger = parse(&unparsed_file)?;
    dbg!(ledger);
    Ok(())
}

fn main() {
    if let Err(e) = run() { println!("Error: {}", e) }
}
