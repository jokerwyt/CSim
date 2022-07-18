use core::panic;
use std::str::Chars;

use cache::CacheManager;
use clap::{Parser};
use Instr::*;

#[derive(Parser)]
pub struct Cli {
    #[clap(short = 's', default_value_t = 2)]
    pub set_bits: u32,
    
    #[clap(short = 'E', default_value_t = 2)]
    pub set_size: usize,
    
    #[clap(short = 'b', default_value_t = 3)]
    pub block_bits: u32,

    #[clap(short = 't', default_value = "../traces/trans.trace")]
    pub trace_file: String,

    /// verbose mode
    #[clap(short = 'v')]
    pub verbose: bool
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
pub enum Instr {
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

    pub fn from_string(raw: &String) -> Instr {
        let mut iter = raw.chars();
        let type_char = iter.next().unwrap();
        assert!(['L', 'S', 'M'].contains(&type_char));
        assert!(iter.next() == Some(' '));
        let address = parse_hex(&mut iter);

        Instr::new(type_char, address)
    }
}


pub mod cache{
    use std::{fs::File, io::Write, fmt::Display};

    use super::Cli;

    #[derive(Debug)]
    struct CacheLine {
        // valid:  bool,
        tag:    u64,
        time:   u32
        // block: Vec<u64>
    }

    type CacheSet = Vec<CacheLine>;

    pub struct CacheStat {
        pub hits: u32,
        pub misses: u32,
        pub evictions: u32 
    }

    pub struct CacheManager {
        set_bits: u32,
        set_size: usize,
        block_bits: u32,
        time_stamp: u32,
        verbose: bool,

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
        pub fn init(para: &Cli) -> CacheManager {
            let mut ret = CacheManager {
                set_bits:       para.set_bits,
                set_size:       para.set_size,
                block_bits:     para.block_bits,
                time_stamp:      0,
                verbose:        para.verbose,

                stat:           CacheStat { hits: 0, misses: 0, evictions: 0 },
                sets:           Vec::<CacheSet>::with_capacity(1 << para.set_bits),
            };
            for _ in 1..=(1 << para.set_bits) {
                ret.sets.push(vec![]);
            }
            ret
        }

        pub fn get_mem(&mut self, addr: u64) -> CacheResult {
            self.time_stamp += 1;

            let set_num: usize = ((addr >> self.block_bits) & ((1 << self.set_bits) - 1)).try_into().unwrap();
            let tag = addr >> (self.set_bits + self.block_bits);

            // It was removed to be consistent with csim-ref. You can turn it on.
            // if self.verbose {  
            //     println!("The next operation manipulates set {}", set_num);
            // }

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



static mut LOCAL_CACHE_MANAGER: Option<&mut CacheManager> = None;

/// run it when your function begin.
/// every time this function is called, it will reset all cache with new parameters.
/// attention! this will cause memory leak
/// so remove it when you want to detect memory leak.
#[no_mangle]
pub extern "C" fn _C_interface_init_cache_manager(set_bits: u32, set_size: u32, block_bits: u32, verbose: u32) {
    let args = Cli {
        set_bits,
        set_size: set_size.try_into().unwrap(),
        block_bits,
        trace_file: "".to_string(),
        verbose: match verbose {   // there is no bool in C...........
            0 => false,
            _ => true
        }
    };
    unsafe {
        LOCAL_CACHE_MANAGER = Some(Box::leak(
            Box::new(CacheManager::init(&args)))
        );
    }
}

fn get_manager() -> &'static mut CacheManager{
    unsafe {
        match &mut LOCAL_CACHE_MANAGER {
            Some(thing) => thing,
            None => panic!("manager was not initialized yet")
        }
    }
}

/// the return value shows the operation result.
#[no_mangle]
pub extern "C" fn _C_interface_access(addr: u64) -> i32 {

    match get_manager().get_mem(addr) {
        cache::CacheResult::Hit => 0,
        cache::CacheResult::MissWithoutEviction => 1,
        cache::CacheResult::MissAndEviction => 2
    }
}

#[no_mangle]
pub extern "C" fn _C_interface_get_miss() -> u32 {
    let stat = get_manager().get_stat();
    stat.misses
    // [stat.hits, stat.misses, stat.evictions]
}
