use std::fs::File;
use std::io::BufReader;
// use std::process::exit;
use anyhow::Result;
// use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
mod tiles;
use tiles::{bit_masked_tiles, number2matrix};

fn u64_2str_matrix(matrix: &[Vec<u64>], mapping: &[&str]) -> Vec<String> {
    let mut str_matrix = vec![];
    for row in matrix {
        let mut str_row = vec![];
        for &value in row {
            str_row.push(mapping[value as usize]);
        }
        str_matrix.push(str_row.join(""));
    }
    str_matrix
}

fn main() -> Result<()> {
    let tilefile = BufReader::new(File::open("tiles.txt")?);
    let all_tiles = bit_masked_tiles(tilefile);
    let mask0 = (1 << 6) + (1 << 29) + (1 << 40);
    // let mask0 = 0;
    let tiles: Vec<u64> = all_tiles
        .iter()
        .filter(|t| (**t & mask0) == 0)
        .cloned()
        .collect();

    let first_tiles: Vec<u64> = tiles
        .iter()
        .take_while(|&&m| (m >> 50) == 1)
        .cloned()
        .collect();

    // Find valid solutions using the bit masks.
    let total = first_tiles
        .par_iter()
        // .progress_count(first_tiles.len() as u64)
        .flat_map(|&m1| {
            // Now we just do Knuth's Algorithm X
            let mask1 = m1 | mask0;
            let filter_loop = |mask: u64, m: u64, uniq: &[u64], length: u64| -> Vec<Vec<u64>> {
                fn print_sol(sol: Vec<u64>) {
                    let mapping = &[" ", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
                    let mut num_grid = vec![vec![0; 10]; 5];
                    for s in sol {
                        let mask = s & ((1u64 << 50) - 1);
                        let kind = (u64::BITS - (s >> 50).leading_zeros()) as u64;
                        let mat = number2matrix(10, 5, mask);
                        for (i, row) in mat.iter().enumerate() {
                            for (j, &value) in row.iter().enumerate() {
                                if value {
                                    num_grid[i][j] = kind;
                                }
                            }
                        }
                    }
                    let str_matrix = u64_2str_matrix(&num_grid, mapping);
                    for line in str_matrix {
                        println!("{:?}", line)
                    }
                    println!();
                }
                fn filter(mask: u64, m: u64, uniq: &[u64], masked: &mut Vec<u64>) {
                    masked.clear();
                    masked.extend(
                        uniq.iter().copied().filter(|m1| {
                            let m2 = m.next_power_of_two();
                            (m1 & mask == 0) && (*m1 >= m2) && (*m1 < m2 << 1)
                        }), // No intersections, and use next tile type
                    );
                }
                fn filter_loop(
                    total: usize,
                    curr: &mut Vec<u64>,
                    sol: &mut Vec<Vec<u64>>,
                    mask: u64,
                    m: u64,
                    uniq: &[u64],
                    depth: u64,
                ) {
                    if depth == 0 {
                        sol.push(curr.to_vec());
                    } else if depth == 1 {
                        for &m1 in uniq.iter().filter(|&m1| m1 & mask == 0) {
                            curr.push(m1);
                            print_sol(curr.to_vec());
                            // exit(0); // Exit on first;
                            sol.push(curr.to_vec());
                            curr.pop();
                        }
                    } else {
                        let mut masked = Vec::with_capacity(total);
                        filter(mask, m, uniq, &mut masked);
                        for &m1 in &masked {
                            let mask1: u64 = mask | m1;
                            curr.push(m1);
                            filter_loop(total, curr, sol, mask1, m1, uniq, depth - 1);
                            curr.pop();
                        }
                    }
                }
                let mut s: Vec<Vec<u64>> = Vec::new();
                filter_loop(tiles.len(), &mut vec![m], &mut s, mask, m, uniq, length - 1);
                s
            };

            filter_loop(mask1, m1, &tiles, 10)
        })
        .collect::<Vec<_>>()
        .len();
    println!("{}", total);
    Ok(())
}
