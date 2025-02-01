use std::io::BufReader;
use std::sync::Arc;
use std::{fs::File, sync::Mutex};
use anyhow::Result;
use clap::Parser;
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use num_bigint::BigUint;
use pathfinding::directed::astar;
use pathfinding::grid::Grid;
use pathfinding::prelude::strongly_connected_components;
use rayon::prelude::*;
use seq_macro::seq;
mod tiles;
use tiles::{bit_masked_tiles, number2grid};

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
}

fn main() -> Result<()> {
    let args = Args::parse();
    let tilefile = BufReader::new(File::open(args.file)?);
    let (tiles,names): (Vec<BigUint>,Vec<String>) = bit_masked_tiles(args.width, args.height, tilefile);
    let mask0 = BigUint::ZERO;
    let total_tiles = tiles.len();
    let max_path: Arc<Mutex<usize>> = Arc::new(Mutex::new(args.known_max - 1));
    let max_tiles =
        |n: usize| args.max_tiles < n || args.height * args.width < n * 5 + args.known_max;
    let min_tiles = |n: usize| args.min_tiles <= n;
    let total = tiles
        .par_iter()
        .progress_count(tiles.len() as u64)
        .flat_map(|m1| {
            let filter =
                |mask: BigUint, m: BigUint, uniq: &[BigUint], masked: &mut Vec<BigUint>| -> bool {
                    masked.clear();
                    masked.extend(
                        uniq.iter()
                            .cloned()
                            .filter(|m1| (m1 & mask.clone() == BigUint::ZERO) && (*m1 > m)),
                    ); // No intersections, and use next tile type
                    masked.is_empty()
                };

            let mut local_max_path: usize = 0;
            let mut solutions: Vec<(Grid, usize)> = Vec::new();
            let max_path = Arc::clone(&max_path);
            let mut max_shortest_path = |flat_grid: BigUint| {
                let grid = number2grid(args.width, args.height, flat_grid.clone());
                let mut max_path_len = 0;
                let cliques = strongly_connected_components(&grid.iter().collect_vec(), |&p| {
                    grid.neighbours(p)
                });
                for group in cliques {
                    if group.len() < local_max_path {
                        continue;
                    }
                    for pair in group.iter().cloned().combinations(2) {
                        if let Some((_, path_len)) = astar::astar(
                            &pair[0],
                            |&p| {
                                grid.neighbours(p)
                                    .into_iter()
                                    .map(|p| (p, 1))
                                    .collect::<Vec<_>>()
                            },
                            |&p| grid.distance(p, pair[1]),
                            |&p| p == pair[1],
                        ) {
                            max_path_len = max_path_len.max(path_len);
                        };
                    }
                }
                let mut max_path = max_path.lock().unwrap();
                if local_max_path < *max_path {
                    solutions.clear();
                }
                if max_path_len >= *max_path {
                    println!("{}", max_path_len + 1);
                    println!("{grid:#?}");
                    let tiles_grid = number2grid(names.len(),1,flat_grid >> (args.height*args.width));
                    println!("{}",names.join(""));
                    println!("{tiles_grid:#?}\n");
                    solutions.push((grid, max_path_len));
                    local_max_path = max_path_len;
                    *max_path = max_path_len;
                }
            };

            seq!(I in 2..=12 {
                let mut masked~I = Vec::with_capacity(total_tiles);

            });
            let mask1 = mask0.clone() | m1.clone();
            if filter(mask1.clone(), m1.clone(), &tiles, &mut masked2) {
                if min_tiles(1) {
                    max_shortest_path(mask1.clone());
                }
                if max_tiles(1) {
                    return solutions;
                }
            };
            for m2 in &masked2 {
                let mask2: BigUint = mask1.clone() | m2.clone();
                if filter(mask2.clone(), m2.clone(), &tiles, &mut masked3) {
                    if min_tiles(2) {
                        max_shortest_path(mask2.clone());
                    }
                    if max_tiles(2) {
                        continue;
                    }
                };
                for m3 in &masked3 {
                    let mask3: BigUint = mask2.clone() | m3.clone();
                    if filter(mask3.clone(), m3.clone(), &tiles, &mut masked4) {
                        if min_tiles(3) {
                            max_shortest_path(mask3.clone());
                        }
                        if max_tiles(3) {
                            continue;
                        }
                    }
                    for m4 in &masked4 {
                        let mask4: BigUint = mask3.clone() | m4.clone();
                        if filter(mask4.clone(), m4.clone(), &tiles, &mut masked5) {
                            if min_tiles(4) {
                                max_shortest_path(mask4.clone());
                            }
                            if max_tiles(4) {
                                continue;
                            }
                        }
                        for m5 in &masked5 {
                            let mask5: BigUint = mask4.clone() | m5.clone();
                            if filter(mask5.clone(), m5.clone(), &tiles, &mut masked6) {
                                if min_tiles(5) {
                                    max_shortest_path(mask5.clone());
                                }
                                if max_tiles(5) {
                                    continue;
                                }
                            }
                            for m6 in &masked6 {
                                let mask6: BigUint = mask5.clone() | m6.clone();
                                if filter(mask6.clone(), m6.clone(), &tiles, &mut masked7) {
                                    if min_tiles(6) {
                                        max_shortest_path(mask6.clone());
                                    }
                                    if max_tiles(6) {
                                        continue;
                                    }
                                }
                                for m7 in &masked7 {
                                    let mask7: BigUint = mask6.clone() | m7.clone();
                                    if filter(mask7.clone(), m7.clone(), &tiles, &mut masked8) {
                                        if min_tiles(7) {
                                            max_shortest_path(mask7.clone());
                                        }
                                        if max_tiles(7) {
                                            continue;
                                        }
                                    }
                                    for m8 in &masked8 {
                                        let mask8: BigUint = mask7.clone() | m8.clone();
                                        if filter(mask8.clone(), m8.clone(), &tiles, &mut masked9) {
                                            if min_tiles(8) {
                                                max_shortest_path(mask8.clone());
                                            }
                                            if max_tiles(8) {
                                                continue;
                                            }
                                        }
                                        for m9 in &masked9 {
                                            let mask9: BigUint = mask8.clone() | m9.clone();
                                            if filter(
                                                mask9.clone(),
                                                m9.clone(),
                                                &tiles,
                                                &mut masked10,
                                            ) {
                                                if min_tiles(9) {
                                                    max_shortest_path(mask9.clone());
                                                }
                                                if max_tiles(9) {
                                                    continue;
                                                }
                                            }
                                            for m10 in &masked10 {
                                                let mask10: BigUint = mask9.clone() | m10.clone();
                                                if filter(
                                                    mask10.clone(),
                                                    m10.clone(),
                                                    &tiles,
                                                    &mut masked11,
                                                ) {
                                                    if min_tiles(10) {
                                                        max_shortest_path(mask10.clone());
                                                    }
                                                    if max_tiles(10) {
                                                        continue;
                                                    }
                                                }
                                                for m11 in &masked11 {
                                                    let mask11: BigUint =
                                                        mask10.clone() | m11.clone();
                                                    masked12 = tiles
                                                        .iter()
                                                        .filter(|&m12| {
                                                            mask11.clone() & m12 == BigUint::ZERO
                                                        })
                                                        .collect();
                                                    if masked12.len() == 0 {
                                                        max_shortest_path(mask11.clone());
                                                    }
                                                    for &m12 in &masked12 {
                                                        let mask12 = mask11.clone() | m12.clone();
                                                        max_shortest_path(mask12.clone());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            solutions
        })
        .collect::<Vec<_>>()
        .iter()
        .filter_map(|(g, l)| {
            if *l == *max_path.lock().unwrap() {
                Some(g)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .len();
    println!("Max path: {}\nTotal:{total}", *max_path.lock().unwrap() + 1);
    Ok(())
}
