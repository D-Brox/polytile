use itertools::Itertools;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn rotate(matrix: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let m = matrix[0].len();
    let n = matrix.len();
    let mut rotated = vec![vec![false; n]; m];
    for (i, j) in (0..n).cartesian_product(0..m) {
        rotated[j][(n - i - 1) % n] = matrix[i][j];
    }
    rotated
}

fn mirror(matrix: &[Vec<bool>]) -> Vec<Vec<bool>> {
    matrix
        .iter()
        .map(|row| row.iter().rev().copied().collect())
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

fn submatrix2matrices(matrix: &[Vec<bool>], a: usize, b: usize) -> Vec<Vec<Vec<bool>>> {
    let m = matrix.len();
    let n = matrix[0].len();

    let mut result = vec![];

    for (i, j) in (0..=(a - m)).cartesian_product(0..=(b - n)) {
        let mut temp = vec![vec![false; b]; a];
        for (k, l) in (0..m).cartesian_product(0..n) {
            temp[i + k][j + l] |= matrix[k][l];
        }
        result.push(temp);
    }
    result
}

fn matrix2number(matrix: &[Vec<bool>]) -> u64 {
    let mut number = 0u64;
    let m = matrix.len();
    let n = matrix[0].len();

    for (i, j) in (0..m).cartesian_product(0..n) {
        number <<= 1;
        number |= matrix[i][j] as u64;
    }
    number
}

pub fn number2matrix(m: usize, n: usize, number: u64) -> Vec<Vec<bool>> {
    let mut matrix = vec![vec![false; m]; n];
    for (i, j) in (0..m).cartesian_product(0..n) {
        if (number & (1 << (i * n + j))) != 0 {
            matrix[j][i] = true;
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
            let n = parts.next().unwrap().parse::<usize>().unwrap();
            let m = parts.next().unwrap().parse::<usize>().unwrap();
            let b = u64::from_str_radix(parts.next().unwrap(), 2).unwrap();
            let matrix = number2matrix(m, n, b);
            let matrices = rotations_and_mirrors(&matrix);
            matrices
                .iter()
                .unique()
                .flat_map(|matrix| submatrix2matrices(matrix, 10, 5))
                .map(|matrix| matrix2number(&matrix) + (1u64 << (50 + l)))
                .collect()
        })
        .sorted()
        .collect_vec()
}
