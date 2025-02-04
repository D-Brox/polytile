use graphalgs::petgraph::{Graph, Undirected};
use graphalgs::shortest_path::shortest_distances;
use itertools::Itertools;
use num_bigint::BigUint;
use pathfinding::grid::Grid;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::{BTreeSet, HashMap};
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
    rotations.dedup();
    rotations
}

fn submatrix2matrices(matrix: &[Vec<bool>], width: usize, height: usize) -> Vec<Vec<Vec<bool>>> {
    let w = matrix.len();
    let h = matrix[0].len();

    let mut result = vec![];
    if height < h || width < w {
        return result;
    }
    for (i, j) in (0..=(height - h)).cartesian_product(0..=(width - w)) {
        let mut temp = vec![vec![false; height]; width];
        for (k, l) in (0..h).cartesian_product(0..w) {
            temp[j + l][i + k] |= matrix[l][k];
        }
        result.push(temp);
    }
    result.dedup();
    result
}

fn matrix2number(matrix: &[Vec<bool>]) -> BigUint {
    let mut number = BigUint::ZERO;
    let width = matrix.len();
    let height = matrix[0].len();
    for (i, j) in (0..height).cartesian_product(0..width) {
        number.set_bit((i * width + j) as u64, matrix[j][i]);
    }

    number
}

pub fn u64_2matrix(width: usize, height: usize, number: u64) -> Vec<Vec<bool>> {
    let mut matrix = vec![vec![false; height]; width];
    for (i, j) in (0..height).cartesian_product(0..width) {
        if (number & (1 << (i * width + j))) != 0 {
            matrix[j][i] = true;
        }
    }
    matrix
}

pub fn number2matrix(width: usize, height: usize, number: &BigUint) -> Vec<Vec<bool>> {
    let mut matrix = vec![vec![false; height]; width];
    for (i, j) in (0..height).cartesian_product(0..width) {
        if (number & (BigUint::from(1u32) << (i * width + j))) != BigUint::ZERO {
            matrix[j][i] = true;
        }
    }
    matrix
}

pub fn number_of_tiles(width: usize, height: usize, number: &BigUint) -> u64 {
    (number >> (width * height)).count_ones()
}

pub fn min_rot(width: usize, height: usize, number: &BigUint) -> BigUint {
    let matrix = number2matrix(width, height, number);
    let bits = (number >> width * height) << height * width;
    rotations_and_mirrors(&matrix)
        .iter()
        .filter(|m| m.len() == matrix.len())
        .map(|m| &bits + matrix2number(m))
        .max()
        .unwrap()
}

pub fn number2grid(width: usize, height: usize, number: &BigUint) -> Grid {
    let mut grid = Grid::new(width, height);
    grid.fill();
    for (i, j) in (0..height).cartesian_product(0..width) {
        if (number & (BigUint::from(1u32) << (i * width + j))) != BigUint::ZERO {
            grid.remove_vertex((j, i));
        }
    }
    grid
}

pub fn longest_shortest_path(width: usize, height: usize, number: &BigUint) -> usize {
    let grid = number2grid(width, height, number);
    let mut graph = Graph::<(usize, usize), usize, Undirected>::new_undirected();
    let nodes = grid
        .iter()
        .map(|square| (square, graph.add_node(square)))
        .collect::<HashMap<_, _>>();
    for (a, b) in grid.edges() {
        graph.add_edge(nodes[&a], nodes[&b], 1);
    }
    let distances = nodes
        .par_iter()
        .flat_map(|(_, &node)| {
            let shortest_distances = shortest_distances(&graph, node);
            shortest_distances
                .iter()
                .filter(|&dist| *dist != f32::INFINITY)
                .map(|dist| *dist as usize)
                .collect::<BTreeSet<_>>()
        })
        .collect::<BTreeSet<_>>();
    *distances.last().unwrap_or(&0)
}

pub fn bit_masked_tiles(
    width: usize,
    height: usize,
    tilefile: BufReader<File>,
) -> (Vec<BigUint>, Vec<String>) {
    let mut tiles = vec![];
    let bit_masked = tilefile
        .lines()
        .enumerate()
        .flat_map(|(l, line)| -> Vec<BigUint> {
            let line = line.unwrap();
            let mut parts = line.split_whitespace();
            let w = parts.next().unwrap().parse::<usize>().unwrap();
            let h = parts.next().unwrap().parse::<usize>().unwrap();
            let b = u64::from_str_radix(parts.next().unwrap(), 2).unwrap();
            tiles.push(parts.next().unwrap().to_owned());
            let matrix = u64_2matrix(w, h, b);
            let matrices = rotations_and_mirrors(&matrix);
            matrices
                .iter()
                .unique()
                .flat_map(|matrix| submatrix2matrices(matrix, width, height))
                .map(|matrix| {
                    let number =
                        matrix2number(&matrix) + (BigUint::from(1u32) << (height * width + l));
                    number
                })
                .collect()
        })
        .sorted()
        .collect_vec();
    (bit_masked, tiles)
}
