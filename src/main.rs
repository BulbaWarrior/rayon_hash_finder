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

    // rayon does not implement IntoParallelIterator for RangeFrom<T>,
    // so we have to use bounded ranges
    // see https://github.com/rayon-rs/rayon/issues/520
    let batch_size = 1_000_000;
    let res = (1u64..)
        .step_by(batch_size as usize)
        .flat_map(|x| {
            let data = x..(x.checked_add(batch_size).unwrap());
            data.into_par_iter()
                .map(|x| (x, sha256::digest(x.to_string())))
                .filter(|(_, hash)| hash.ends_with(&"0".repeat(args.zeros)))
                .collect::<Vec<(u64, String)>>()
        })
        .take(args.target_count);

    // output without collecting so we can see the progress
    res.for_each(|(num, hash)| println!(r#"{num}, "{hash}""#));
}
