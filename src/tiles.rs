use itertools::Itertools;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn rotate(matrix: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let m = matrix.len();
    let n = matrix[0].len();
    let mut rotated = vec![vec![false; n]; m];
    for (i, j) in (0..n).cartesian_product(0..m) {
        rotated[j][n - i - 1] = matrix[i][j];
    }
    rotated
}

fn mirror(matrix: &[Vec<bool>]) -> Vec<Vec<bool>> {
    matrix
        .iter()
        .map(|row| row.iter().rev().cloned().collect())
        .collect()
}

fn rotations_and_mirrors(matrix: &[Vec<bool>]) -> Vec<Vec<Vec<bool>>> {
    let mut rotations = vec![];
    let mut rotated = matrix.to_vec();
    for _ in 0..4 {
        rotated = rotate(&rotated);
        rotations.push(rotated.to_vec());
    }
    let mirrors: Vec<Vec<Vec<bool>>> = rotations.iter().map(|matrix| mirror(matrix)).collect();
    rotations.extend(mirrors);
    rotations
}

fn matrix2number(matrix: &[Vec<bool>]) -> u16 {
    let mut number = 0u16;

    for (i, j) in (0..4).cartesian_product(0..4) {
        number |= (matrix[i][j] as u16) << (4 * i + j);
    }
    number
}


fn submatrix2matrices(matrix: &Vec<Vec<bool>>, a: usize, b: usize) -> Vec<Vec<Vec<bool>>> {
    let m = matrix.len();
    let n = matrix[0].len();

    let mut result = vec![];

    for (i, j) in (0..a - m + 1).cartesian_product(0..b - n + 1) {
        let mut temp = vec![vec![false; b]; a];
        for (k, l) in (0..m).cartesian_product(0..n) {
            temp[i + k][j + l] |= matrix[k][l];
        }
        result.push(temp);
    }

    result
}

pub fn number2matrix(m: usize, n: usize, number: u64) -> Vec<Vec<bool>> {
    let mut matrix = vec![vec![false; n]; m];

    for (i, j) in (0..m).cartesian_product(0..n) {
        if (number & (1 << (i * n + j))) != 0 {
            matrix[i][j] = true;
        }
    }
    matrix
}

pub fn bit_masked_tiles(tilefile: BufReader<File>) -> Vec<u64> {
    tilefile
        .lines()
        .enumerate()
        .flat_map(|(l, line)| -> Vec<u64> {
            let line = line.unwrap();
            let mut parts = line.split_whitespace();
            let m = parts.next().unwrap().parse::<usize>().unwrap();
            let n = parts.next().unwrap().parse::<usize>().unwrap();
            let b = u64::from_str_radix(parts.next().unwrap(), 2).unwrap();
            let matrix = number2matrix(m, n, b);
            let matrices = submatrix2matrices(&matrix, 10, 5);
            matrices
                .iter()
                .flat_map(|matrix| rotations_and_mirrors(matrix))
                .map(|matrix| (matrix2number(&matrix) as u64) | 1 << (50 + l))
                .collect()
        })
        .sorted()
        .dedup()
        .collect_vec()
}