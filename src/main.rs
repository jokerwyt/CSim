use core::panic;
use std::{io::{self, BufRead}, fs::File, str::Chars};

use clap::{Parser};
use crate::Instr::*;

#[derive(Parser)]
pub struct Cli {
    #[clap(short = 's', default_value_t = 2)]
    set_bits: u32,
    
    #[clap(short = 'E', default_value_t = 2)]
    set_size: usize,
    
    #[clap(short = 'b', default_value_t = 3)]
    block_bits: u32,

    #[clap(short = 't', default_value = "../traces/trans.trace")]
    trace_file: String,

    /// verbose mode
    #[clap(short = 'v')]
    verbose: bool
}

/// parse a u64 hex number until a comma.
fn parse_hex(iter: &mut Chars) -> u64 {
    let mut ret = 0;
    for cur in iter {
        if cur == ',' {
            break;
        } else {
            ret = ret * 16 + (cur.to_digit(16).unwrap() as u64);
        }
    }
    ret
}

#[derive(Debug)]
enum Instr {
    Load(u64),
    Store(u64),
    Modify(u64)
}


impl Instr {
    fn new(c: char, addr: u64) -> Instr {
        match c {
            'L' => Load(addr),
            'S' => Store(addr),
            'M' => Modify(addr),
            _   => panic!("strange instruction type char.")
        }
    }

    fn from_string(raw: &String) -> Instr {
        let mut iter = raw.chars();
        let type_char = iter.next().unwrap();
        assert!(['L', 'S', 'M'].contains(&type_char));
        assert!(iter.next() == Some(' '));
        let address = parse_hex(&mut iter);

        Instr::new(type_char, address)
    }
}


mod cache{
    use std::{fs::File, io::Write, fmt::Display};

    #[derive(Debug)]
    struct CacheLine {
        // valid:  bool,
        tag:    u64,
        time:   u32
        // block: Vec<u64>
    }

    type CacheSet = Vec<CacheLine>;

    pub struct CacheStat {
        hits: u32,
        misses: u32,
        evictions: u32 
    }

    pub struct CacheManager {
        set_bits: u32,
        set_size: usize,
        block_bits: u32,
        time_stamp: u32,
        
        stat: CacheStat,
        sets: Vec<CacheSet>
    }

    #[derive(Debug)]
    pub enum CacheResult {
        Hit,
        MissWithoutEviction,
        MissAndEviction
    }

    impl Display for CacheResult {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", 
                match self {
                    Self::Hit => "hit",
                    Self::MissWithoutEviction => "miss",
                    Self::MissAndEviction => "miss eviction"
                }
            )
        }
    }

    impl CacheStat {
        pub fn summary(&self) {
            // {
            //     printf("hits:%d misses:%d evictions:%d\n", hits, misses, evictions);
            //     FILE* output_fp = fopen(".csim_results", "w");
            //     assert(output_fp);
            //     fprintf(output_fp, "%d %d %d\n", hits, misses, evictions);
            //     fclose(output_fp);
            // }
            println!("hits:{} misses:{} evictions:{}", self.hits, self.misses, self.evictions);
            if let Ok(mut file) = File::create(".csim_results") {
                if let Err(_) 
                    = file.write(format!("{} {} {}\n", self.hits, self.misses, self.evictions).as_bytes()) {
                    panic!("write error")
                }
            } else {
                panic!("create file error");
            }
        }
    }
    
    impl CacheManager {
        pub fn init(para: &crate::Cli) -> CacheManager {
            let mut ret = CacheManager {
                set_bits:       para.set_bits,
                set_size:       para.set_size,
                block_bits:     para.block_bits,
                time_stamp:      0,

                stat:           CacheStat { hits: 0, misses: 0, evictions: 0 },
                sets:           Vec::<CacheSet>::with_capacity(1 << para.set_bits),
            };
            for _ in 1..=(1 << para.set_bits) {
                ret.sets.push(vec![]);
            }
            ret
        }

        fn get_mem(&mut self, addr: u64) -> CacheResult {
            self.time_stamp += 1;

            let set_num: usize = ((addr >> self.block_bits) & ((1 << self.set_bits) - 1)).try_into().unwrap();
            let tag = addr >> (self.set_bits + self.block_bits);



            for line in &mut self.sets[set_num] {
                if line.tag == tag {
                    line.time = self.time_stamp;
                    self.stat.hits += 1;
                    return CacheResult::Hit;
                }
            }
            
            if self.sets[set_num].len() < self.set_size {
                self.sets[set_num].push(CacheLine{
                    tag,
                    time: self.time_stamp,
                });
                
                self.stat.misses += 1;
                return CacheResult::MissWithoutEviction
            } else {
                let earliest = self.sets[set_num].iter_mut()
                                        .min_by_key(|lines| lines.time)
                                        .unwrap();

                *earliest = CacheLine{
                    tag,
                    time: self.time_stamp
                };

                self.stat.misses += 1;
                self.stat.evictions += 1;
                return CacheResult::MissAndEviction
            }
        }

        pub fn load(&mut self, addr: u64) -> CacheResult {
            self.get_mem(addr)
        }
    
        pub fn store(&mut self, addr: u64) -> CacheResult {
            self.get_mem(addr)
        }
        
        pub fn get_stat(&self) -> &CacheStat {
            &self.stat
        }
    }
}


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
