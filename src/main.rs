use std::fs::File;
use std::io::BufReader;
use std::sync::RwLock;

use indicatif::ProgressBar;
use itertools::Itertools;
use rayon::prelude::*;

use anyhow::Result;
use clap::Parser;

mod tiles;
use num_bigint::BigUint;
use tiles::{bit_masked_tiles, longest_shortest_path, min_rot, number2grid, number_of_tiles};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    file: String,

    #[arg(long)]
    height: usize,

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
    let (tiles, tile_names): (Vec<BigUint>, Vec<String>) =
        bit_masked_tiles(args.width, args.height, tilefile);
    let mut first_tiles = tiles
        .iter()
        .map(|n| min_rot(args.width, args.height, n))
        .sorted()
        .collect::<Vec<_>>();
    first_tiles.dedup();
    let max_path: RwLock<usize> = RwLock::new(args.known_max - 1);
    let uniq = tiles.iter().collect_vec();
    let bar = ProgressBar::new(first_tiles.len() as u64);
    bar.inc(0);
    let solutions = first_tiles
        .par_iter()
        .flat_map(|m1| {
            // Now we just do Knuth's Algorithm X
            let filter_loop = |m: &BigUint, uniq: &[&BigUint]| {
                // Solution check
                fn max_shortest_path(
                    flat_grid: &BigUint,
                    sizes: (usize, usize),
                    max_path: &RwLock<usize>,
                    solutions: &mut Vec<(BigUint, usize)>,
                    tile_names: Option<&[String]>,
                ) {
                    let mask = min_rot(sizes.0, sizes.1, flat_grid);
                    let diameter =
                        longest_shortest_path(sizes.0, sizes.1, &mask, *(max_path.read().unwrap()));
                    if diameter >= *(max_path.read().unwrap()) {
                        {
                            let mut max_path = max_path.write().unwrap();
                            *max_path = diameter;
                        }
                        solutions.push((mask, diameter));
                        if let Some(tile_names) = tile_names {
                            println!("{:#?}", number2grid(sizes.0, sizes.1, flat_grid));
                            println!("{}", tile_names.join(""));
                            println!(
                                "{:#?} {}\n",
                                number2grid(
                                    tile_names.len(),
                                    1,
                                    &(flat_grid >> (sizes.0 * sizes.1))
                                ),
                                diameter + 1
                            );
                        }
                    }
                }

                // Filter bit-masked tiles
                fn filter<'a>(
                    mask: &BigUint,
                    uniq: &[&'a BigUint],
                    masked: &mut Vec<&'a BigUint>,
                ) -> bool {
                    masked.extend(
                        uniq.iter()
                            .filter(|&m1| (mask < *m1) & (mask & *m1 == BigUint::ZERO)),
                    ); // No intersections, and use next tile type
                    masked.is_empty()
                }

                // Nested for loop of arbitrary depth
                fn filter_loop<MinFn, MaxFn>(
                    mask: &BigUint,
                    uniq: &[&BigUint],
                    sizes: (usize, usize),
                    min_max: &(MinFn, MaxFn),
                    max_path: &RwLock<usize>,
                    solutions: &mut Vec<(BigUint, usize)>,
                    tile_names: Option<&[String]>,
                ) where
                    MinFn: Fn(&BigUint) -> bool,
                    MaxFn: Fn(&BigUint) -> bool,
                {
                    let mut masked = Vec::with_capacity(uniq.len());
                    // If no more tiles fit, check solution
                    if min_max.1(mask) || filter(mask, uniq, &mut masked) {
                        max_shortest_path(mask, sizes, max_path, solutions, tile_names);
                        return;
                    } else if min_max.0(mask) {
                        // Print if min has been reached
                        max_shortest_path(mask, sizes, max_path, solutions, tile_names);
                    }
                    // Check next possible tiles
                    for &m in &masked {
                        filter_loop(
                            &(mask | m),
                            &masked,
                            sizes,
                            min_max,
                            max_path,
                            solutions,
                            tile_names,
                        );
                    }
                }
                let mut solutions = Vec::new();
                filter_loop(
                    m,
                    uniq,
                    (args.width, args.height),
                    &(
                        |mask| {
                            number_of_tiles(args.width, args.height, mask) >= args.min_tiles as u64
                        },
                        |mask| {
                            number_of_tiles(args.width, args.height, mask) > args.max_tiles as u64
                        },
                    ),
                    &max_path,
                    &mut solutions,
                    if args.verbose {
                        Some(&tile_names)
                    } else {
                        None
                    },
                );
                solutions
            };

            let s = filter_loop(m1, &uniq);
            bar.inc(1);
            s
        })
        .collect::<Vec<_>>();

    println!("\n#### Solutions ####\n");
    let solutions = solutions
        .into_iter()
        .filter_map(|(g, l)| {
            if l == *max_path.read().unwrap() {
                println!("{:#?}", number2grid(args.width, args.height, &g));
                let tiles_grid =
                    number2grid(tile_names.len(), 1, &(g >> (args.height * args.width)));
                println!("{}", tile_names.join(""));
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
        *max_path.read().unwrap() + 1
    );
    Ok(())
}
