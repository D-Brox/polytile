use std::fs::File;
use std::io::BufReader;
// use std::process::exit;
use anyhow::Result;
// use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use seq_macro::seq;
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
        println!("{line}");
    }
    println!();
}

fn main() -> Result<()> {
    let tilefile = BufReader::new(File::open("tiles.txt")?);
    let all_tiles = bit_masked_tiles(tilefile);
    let mask0 = (1 << 6) + (1 << 29) + (1 << 40);
    // let mask0 = 0;
    let tiles: Vec<u64> = all_tiles
        .iter()
        .filter(|t| (**t & mask0) == 0)
        .copied()
        .collect();
    let total_tiles = tiles.len();

    let first_tiles: Vec<u64> = tiles
        .iter()
        .take_while(|&&m| (m >> 50) == 1)
        .copied()
        .collect();

    // Find valid solutions using the bit masks.
    let total = first_tiles
        .par_iter()
        // .progress_count(first_tiles.len() as u64)
        .flat_map(|&m1| {
            // Now we just do Knuth's Algorithm X
            let filter = |mask: u64, m: u64, uniq: &[u64], masked: &mut Vec<u64>| {
                masked.clear();
                masked.extend(uniq.iter().copied().filter(|m1| {
                    let m2 = m.next_power_of_two();
                    (m1 & mask == 0) && (*m1 >= m2) && (*m1 < m2 << 1)
                })); // No intersections, and use next tile type
            };

            let mut solutions = Vec::new();
            seq!(I in 2..=9 {
                let mut masked~I = Vec::with_capacity(total_tiles);

            });
            let mask1 = mask0 | m1;
            filter(mask1, m1, &tiles, &mut masked2);
            for &m2 in &masked2 {
                let mask2: u64 = mask1 | m2;
                filter(mask2, m2, &tiles, &mut masked3);
                for &m3 in &masked3 {
                    let mask3: u64 = mask2 | m3;
                    filter(mask3, m3, &tiles, &mut masked4);
                    for &m4 in &masked4 {
                        let mask4: u64 = mask3 | m4;
                        filter(mask4, m4, &tiles, &mut masked5);
                        for &m5 in &masked5 {
                            let mask5: u64 = mask4 | m5;
                            filter(mask5, m5, &tiles, &mut masked6);
                            for &m6 in &masked6 {
                                let mask6: u64 = mask5 | m6;
                                filter(mask6, m6, &tiles, &mut masked7);
                                for &m7 in &masked7 {
                                    let mask7: u64 = mask6 | m7;
                                    filter(mask7, m7, &tiles, &mut masked8);
                                    for &m8 in &masked8 {
                                        let mask8: u64 = mask7 | m8;
                                        filter(mask8, m8, &tiles, &mut masked9);
                                        for &m9 in &masked9 {
                                            let mask9: u64 = mask8 | m9;
                                            for &m10 in tiles.iter().filter(|&m10| mask9 & m10 == 0)
                                            {
                                                print_sol(vec![
                                                    m1, m2, m3, m4, m5, m6, m7, m8, m9, m10,
                                                ]);
                                                // exit(0);
                                                solutions.push([
                                                    m1, m2, m3, m4, m5, m6, m7, m8, m9, m10,
                                                ]);
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
        .len();
    println!("{total}");
    Ok(())
}
