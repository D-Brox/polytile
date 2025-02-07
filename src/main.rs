use anyhow::Result;
use clap::Parser;
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use num_bigint::BigUint;

use rayon::prelude::*;
use std::io::BufReader;
use std::sync::Arc;
use std::{fs::File, sync::Mutex};
mod tiles;
use tiles::{bit_masked_tiles, longest_shortest_path, min_rot, number2grid, number_of_tiles};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    file: String,

    /// Name of the person to greet
    #[arg(long)]
    height: usize,

    /// Number of times to greet
    #[arg(long)]
    width: usize,

    #[arg(long, default_value_t = 1)]
    known_max: usize,

    #[arg(long, default_value_t = 0)]
    min_tiles: usize,

    #[arg(long, default_value_t = 12)]
    max_tiles: usize,

    #[arg(long, default_value_t = false)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let tilefile = BufReader::new(File::open(args.file)?);
    let (tiles, names): (Vec<BigUint>, Vec<String>) =
        bit_masked_tiles(args.width, args.height, tilefile);
    let mut first_tiles = tiles
        .iter()
        .map(|n| min_rot(args.width, args.height, &n))
        .sorted()
        .collect::<Vec<_>>();
    first_tiles.dedup();
    let max_path: Arc<Mutex<usize>> = Arc::new(Mutex::new(args.known_max - 1));
    let solutions = first_tiles
        // .iter()
        .par_iter()
        .progress_count(first_tiles.len() as u64)
        .flat_map(|m1| {
            let max_path = Arc::clone(&max_path);

            // Now we just do Knuth's Algorithm X
            let filter_loop = |m: &BigUint, uniq: &[BigUint]| {
                fn max_shortest_path(
                    flat_grid: &BigUint,
                    width: usize,
                    height: usize,
                    names: &Vec<String>,
                    max_path: &Arc<Mutex<usize>>,
                    solutions: &mut Vec<(BigUint, usize)>,
                    verbose: bool,
                ) {
                    let mask = min_rot(width, height, &flat_grid);
                    let max_path_len = longest_shortest_path(width, height, &mask, {
                        let max_path = max_path.lock().unwrap();
                        *max_path
                    });
                    let mut max_path = max_path.lock().unwrap();
                    if max_path_len >= *max_path {
                        *max_path = max_path_len;
                        drop(max_path);
                        solutions.push((mask, max_path_len));
                        if verbose {
                            println!("{:#?}", number2grid(width, height, flat_grid));
                            println!("{}", names.join(""));
                            println!(
                                "{:#?} {}\n",
                                number2grid(names.len(), 1, &(flat_grid >> (height * width))),
                                max_path_len + 1
                            );
                        }
                    }
                }

                fn filter<'a>(
                    mask: &BigUint,
                    m: &BigUint,
                    uniq: &'a [BigUint],
                    masked: &mut Vec<&'a BigUint>,
                ) -> bool {
                    masked.clear();
                    masked.extend(
                        uniq.iter()
                            .filter(|&m1| (mask & m1 == BigUint::ZERO) && (m1 > m))
                    ); // No intersections, and use next tile type
                    masked.is_empty()
                }
                fn filter_loop<F, G>(
                    mask: &BigUint,
                    m: &BigUint,
                    uniq: &[BigUint],
                    width: usize,
                    height: usize,
                    max_tiles: &F,
                    min_tiles: &G,
                    names: &Vec<String>,
                    max_path: &Arc<Mutex<usize>>,
                    solutions: &mut Vec<(BigUint, usize)>,
                    verbose: bool,
                ) where
                    F: Fn(&BigUint) -> bool,
                    G: Fn(&BigUint) -> bool,
                {
                    if max_tiles(&mask) {
                        return;
                    }
                    if min_tiles(&mask) {
                        max_shortest_path(
                            &mask, width, height, &names, &max_path, solutions, verbose,
                        );
                    }
                    let mask1: BigUint = mask | m;
                    let mut masked1 = Vec::with_capacity(uniq.len());
                    if filter(&mask1, m, uniq, &mut masked1) {
                        max_shortest_path(
                            &mask1, width, height, &names, &max_path, solutions, verbose,
                        );
                        return;
                    }
                    for m1 in &masked1 {
                        filter_loop(
                            &mask1, m1, uniq, width, height, max_tiles, min_tiles, &names,
                            &max_path, solutions, verbose,
                        );
                    }
                }
                let mut solutions = Vec::new();
                filter_loop(
                    &BigUint::ZERO,
                    &m,
                    uniq,
                    args.width,
                    args.height,
                    &|mask| number_of_tiles(args.width, args.height, mask) > args.max_tiles as u64,
                    &|mask| number_of_tiles(args.width, args.height, mask) >= args.min_tiles as u64,
                    &names,
                    &max_path,
                    &mut solutions,
                    args.verbose,
                );
                solutions
            };

            filter_loop(&m1, &tiles)
        })
        .collect::<Vec<_>>();

    println!("\n#### Solutions ####\n");
    let solutions = solutions
        .into_iter()
        .filter_map(|(g, l)| {
            if l == *max_path.lock().unwrap() {
                println!("{:#?}", number2grid(args.width, args.height, &g));
                let tiles_grid = number2grid(names.len(), 1, &(g >> (args.height * args.width)));
                println!("{}", names.join(""));
                println!("{tiles_grid:#?}\n");
                Some(())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let total = solutions.len();
    println!(
        "Max path: {}\nUnique solutions:{total}",
        *max_path.lock().unwrap() + 1
    );
    Ok(())
}
