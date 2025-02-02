use anyhow::Result;
use clap::Parser;
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use num_bigint::BigUint;
use pathfinding::directed::astar;
use pathfinding::prelude::strongly_connected_components;
use rayon::prelude::*;
use std::io::BufReader;
use std::sync::Arc;
use std::{fs::File, sync::Mutex};
mod tiles;
use tiles::{bit_masked_tiles, min_rot, number2grid};

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
    let mask0 = BigUint::ZERO;
    let max_path: Arc<Mutex<usize>> = Arc::new(Mutex::new(args.known_max - 1));
    let mut solutions = tiles
        // .iter()
        .par_iter()
        .progress_count(tiles.len() as u64)
        .flat_map(|m1| {
            let mut local_max_path: usize = args.known_max-1;
            let max_path = Arc::clone(&max_path);

            // Now we just do Knuth's Algorithm X
            let mut filter_loop = |mask: BigUint,
                                   m: BigUint,
                                   uniq: &[BigUint],
                                   length: usize|
             -> Vec<(BigUint, usize)> {
                fn max_shortest_path(
                    flat_grid: &BigUint,
                    width: usize,
                    height: usize,
                    names: &Vec<String>,
                    max_path: &Arc<Mutex<usize>>,
                    local_max_path: &mut usize,
                    solutions: &mut Vec<(BigUint, usize)>,
                    verbose: bool,
                ) {
                    let grid = number2grid(width, height, flat_grid.clone());
                    let mut max_path_len = 0;
                    let cliques = strongly_connected_components(&grid.iter().collect_vec(), |&p| {
                        grid.neighbours(p)
                    });
                    for group in cliques {
                        if group.len() < *local_max_path {
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
                    if local_max_path < &mut max_path {
                        solutions.clear();
                    }
                    if max_path_len >= *max_path {
                        solutions.push((min_rot(width, height, flat_grid.clone()), max_path_len));
                        *local_max_path = max_path_len;
                        *max_path = max_path_len;
                        if verbose {
                            let tiles_grid = number2grid(names.len(), 1, flat_grid.clone() >> (height * width));
                            println!("{:#?}", number2grid(width, height, flat_grid.clone()));
                            println!("{}", names.join(""));
                            println!("{tiles_grid:#?} {}\n",max_path_len+1);
                        }
                    }
                }

                fn filter(
                    mask: BigUint,
                    m: BigUint,
                    uniq: &[BigUint],
                    masked: &mut Vec<BigUint>,
                ) -> bool {
                    masked.clear();
                    masked.extend(
                        uniq.iter()
                            .cloned()
                            .filter(|m1| (m1 & mask.clone() == BigUint::ZERO) && (*m1 > m)),
                    ); // No intersections, and use next tile type
                    masked.is_empty()
                }
                fn filter_loop(
                    total: usize,
                    sol: &mut Vec<(BigUint, usize)>,
                    mask: &BigUint,
                    m: BigUint,
                    uniq: &[BigUint],
                    depth: usize,
                    width: usize,
                    height: usize,
                    max: usize,
                    min: usize,
                    known: usize,
                    names: &Vec<String>,
                    max_path: &Arc<Mutex<usize>>,
                    local_max_path: &mut usize,
                    verbose:bool,
                ) {
                    let max_tiles = |n: usize, w: usize, h: usize, m: usize, k: usize| {
                        m < n || h * w < n * 5 + k
                    };
                    let min_tiles = |n: usize, m: usize| m <= n;
                    if min_tiles(names.len() - depth, min) {
                        max_shortest_path(
                            &mask,
                            width,
                            height,
                            &names,
                            &max_path,
                            local_max_path,
                            sol,
                            verbose,
                        );
                    }
                    if max_tiles(names.len() - depth, width, height, max, known) {
                        return;
                    }
                    // println!("{}",names.len() - depth);
                    let mask1: BigUint = mask.clone() | m.clone();
                    let mut masked1 = Vec::with_capacity(total);
                    if filter(mask1.clone(), m.clone(), uniq, &mut masked1) {
                        max_shortest_path(
                            &mask1,
                            width,
                            height,
                            &names,
                            &max_path,
                            local_max_path,
                            sol,
                            verbose,
                        );
                        return;
                    }
                    for m1 in &masked1 {
                        filter_loop(
                            total,
                            sol,
                            &mask1,
                            m1.clone(),
                            uniq,
                            depth,
                            width,
                            height,
                            max,
                            min,
                            known,
                            &names,
                            &max_path,
                            local_max_path,
                            verbose,
                        );
                    }
                }
                let mut s: Vec<(BigUint, usize)> = Vec::new();
                filter_loop(
                    tiles.len(),
                    &mut s,
                    &mask,
                    m,
                    uniq,
                    length,
                    args.width,
                    args.height,
                    args.max_tiles,
                    args.min_tiles,
                    args.known_max,
                    &names,
                    &max_path,
                    &mut local_max_path,
                    args.verbose,
                );
                s
            };

            let solutions: Vec<(BigUint, usize)> =
                filter_loop(mask0.clone(), m1.clone(), &tiles, names.len());

            solutions
        })
        .filter_map(|(g, l)| {
            if l == *max_path.lock().unwrap() {
                Some((g, l))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    solutions.sort_by(|a,b|a.0.cmp(&b.0));
    solutions.dedup();
    let solutions = solutions.iter().map(|(g, l)| {
        println!("{}", l + 1);
        println!("{:#?}", number2grid(args.width, args.height, g.clone()));
        let tiles_grid = number2grid(names.len(), 1, g.clone() >> (args.height * args.width));
        println!("{}", names.join(""));
        println!("{tiles_grid:#?}\n");
    }).collect::<Vec<_>>();
    let total = solutions.len();
    println!("Max path: {}\nTotal:{total}", *max_path.lock().unwrap() + 1);
    Ok(())
}
