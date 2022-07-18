use std::{io::{self, BufRead}, fs::File};
use csim::{Cli, Instr::{*, self}, cache};
use clap::Parser;

fn main() {
    let args = Cli::parse();
    let mut manager = cache::CacheManager::init(&args);

    for ins in io::BufReader::new(
        File::open(args.trace_file).unwrap()
    ).lines() { 
        let ins = ins.unwrap();
        if ins.starts_with(' ') == false { continue }
        let raw = ins[1..].to_string();
        let ins = Instr::from_string(&raw);
        let result = match ins {
            Load(addr) => {
                manager.load(addr)
            },
            Store(addr) => {
                manager.store(addr)
            },
            Modify(addr) => {
                manager.load(addr);
                manager.store(addr)
            }
        };
        if args.verbose {
            println!("{} {}", raw, result);
        }
    }
    manager.get_stat().summary();
}
