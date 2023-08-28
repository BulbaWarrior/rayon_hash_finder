use std::ops::Range;

use clap::Parser;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'N')]
    /// trailing zeros count
    zeros: usize,
    #[arg(short = 'F')]
    /// number of hash values to find
    target_count: usize,
}

fn main() {
    let args = Args::parse();
    let config = Miner {
        zeros: args.zeros,
        batch_size: 100_000, // experimentally found to work best on my machine
    };

    let res = config.into_iter().take(args.target_count);

    let hashes: Vec<String> = res
        .map(|(num, hash)| format!(r#"{num}, "{hash}""#))
        .collect();
    let out = hashes.join("\n");
    println!("{out}");
}

struct Miner {
    zeros: usize,
    batch_size: usize,
}

impl IntoIterator for Miner {
    type Item = (u64, String);

    type IntoIter = HashIter<std::vec::IntoIter<(u64, String)>>;

    fn into_iter(self) -> Self::IntoIter {
        // generates infinite Iterator of values matching prefix zeroes requirements along with
        // their digests, `batch_size` values are submitted to be calculated to the thread pool at a
        // time
        Self::IntoIter {
            current_iter: self.check_range(0..self.batch_size as u64).into_iter(),
            batch_size: self.batch_size,
            offset: self.batch_size as u64,
            config: self,
        }
    }
}

impl Miner {
    fn check(&self, val: u64) -> Option<String> {
        let hash = sha256::digest(val.to_string());
        if hash.ends_with(&"0".repeat(self.zeros)) {
            Some(hash)
        } else {
            None
        }
    }

    fn check_range(&self, range: Range<u64>) -> Vec<(u64, String)> {
        let data: Vec<u64> = range.collect();
        data.par_iter()
            .flat_map(|&x| Some((x, self.check(x)?)))
            .collect()
    }
}

struct HashIter<I: Iterator<Item = (u64, String)>> {
    config: Miner,
    current_iter: I,
    batch_size: usize,
    offset: u64,
}

impl Iterator for HashIter<std::vec::IntoIter<(u64, String)>> {
    type Item = (u64, String);

    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.current_iter.next() {
            Some(x) => Some(x),
            None => {
                self.current_iter = self
                    .config
                    .check_range(self.offset..(self.offset + self.batch_size as u64))
                    .into_iter();
                self.offset += self.batch_size as u64;
                self.next()
            }
        };
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::Miner;

    fn get_config() -> Miner {
        Miner {
            zeros: 3,
            batch_size: 100,
        }
    }

    #[test]
    fn check_success() {
        let config = get_config();
        let res = config.check(4163);
        assert_eq!(
            res,
            Some("95d4362bd3cd4315d0bbe38dfa5d7fb8f0aed5f1a31d98d510907279194e3000".into())
        );
    }

    #[test]
    fn check_failure() {
        let config = get_config();
        let res = config.check(4162);
        assert_eq!(res, None);
    }

    #[test]
    fn get_first_two() {
        let config = get_config();
        let res: Vec<_> = config.into_iter().take(2).collect();
        assert_eq!(
            res,
            [
                (
                    4163,
                    "95d4362bd3cd4315d0bbe38dfa5d7fb8f0aed5f1a31d98d510907279194e3000".into()
                ),
                (
                    11848,
                    "cb58074fd7620cd0ff471922fd9df8812f29f302904b15e389fc14570a66f000".into()
                )
            ]
        );
    }
}
