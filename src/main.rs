use clap::Parser;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

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

    let batch_size = 1_000_000;
    let res = hashes(batch_size, args.zeros).take(args.target_count);
    // output without collecting so we can see the progress
    res.for_each(|(num, hash)| println!(r#"{num}, "{hash}""#));
}

/// Infinite iterator for hashes with `zeroes` trailng zeros along with values that produced them
/// `batch_size` controls how many values are submitted to the thread pool at a time
fn hashes(batch_size: u64, zeros: usize) -> impl Iterator<Item = (u64, String)> {
    // rayon does not implement IntoParallelIterator for RangeFrom<T>,
    // so we have to use bounded ranges
    // see https://github.com/rayon-rs/rayon/issues/520
    (1u64..).step_by(batch_size as usize).flat_map(move |x| {
        let data = x..(x.checked_add(batch_size).unwrap());
        data.into_par_iter()
            .map(|x| (x, sha256::digest(x.to_string())))
            .filter(|(_, hash)| hash.ends_with(&"0".repeat(zeros)))
            .collect::<Vec<(u64, String)>>()
    })
}

#[cfg(test)]
mod tests {
    use crate::hashes;

    #[test]
    fn get_two_hashes() {
        let res: Vec<_> = hashes(1000, 3).take(2).collect();
        assert_eq!(
            &res,
            &[
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
