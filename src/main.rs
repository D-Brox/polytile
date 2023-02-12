use anyhow::Result;
use rayon::prelude::*;
use std::fs::File;
use std::io::BufReader;
mod tiles;
use tiles::{bit_masked_tiles,number2matrix};

fn u64_2str_matrix<'a>(matrix: &[Vec<u64>], mapping: &'a [&'a str]) -> Vec<Vec<&'a str>> {
    let mut str_matrix = vec![];
    for row in matrix {
        let mut str_row = vec![];
        for &value in row {
            str_row.push(mapping[value as usize]);
        }
        str_matrix.push(str_row);
    }
    str_matrix
}

fn vec2array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

fn main() -> Result<()> {
    let tilefile = BufReader::new(File::open("tiles.txt")?);
    let tiles = bit_masked_tiles(tilefile);
    let mask0 = 0;

    // Find valid solutions using the bit masks.
    let mask_solutions: Vec<[u64; 10]> = tiles
        .par_iter() // Now we just do Knuth's Algorithm X
        .flat_map(|&m1| {
            let mask1 = m1 | mask0;
            let filter_loop = |mask: u64, m: u64, uniq: &[u64], length: u64| {
                fn filter(mask: u64, m: u64, uniq: &[u64], masked: &mut Vec<u64>) {
                    masked.clear();
                    masked.extend(
                        uniq.iter()
                            .copied()
                            .take_while(|m1| *m1 < m) // No permutations.
                            .filter(|m1| m1 & mask == 0), // No intersection.
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
                        for m1 in uniq.iter().take_while(|&m1| *m1 > m) {
                            if m1 & mask == 0 {
                                curr.push(*m1);
                                filter_loop(0, curr, sol, 0, 0, &[0], 0);
                                curr.pop();
                            }
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

            let s = filter_loop(mask1, m1, &tiles, 10);
            let mut solutions = Vec::new();

            for i in s {
                let j: [u64; 10] = vec2array(i);
                solutions.push(j)
            }
            solutions
        })
        .collect();

    let mut num_grid = vec![vec![0; 10]; 5];
    for solution in mask_solutions {
        for s in solution {
            let mask = s & ((1u64 << 50) - 1);
            let kind = s >> 50;
            let mat = number2matrix(10, 5, mask);
            for (i, row) in mat.iter().enumerate() {
                for (j, &value) in row.iter().enumerate() {
                    if value {
                        num_grid[i][j] = kind;
                    }
                }
            }
            println!(" ");
        }
    }

    let mapping = &[" ", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
    let str_matrix = u64_2str_matrix(&num_grid, mapping);
    for line in str_matrix {
        println!("{:?}", line)
    }

    Ok(())
}
