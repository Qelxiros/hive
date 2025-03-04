use std::{
    collections::{HashMap, HashSet},
    iter::once,
    ops::{Index, IndexMut, Not},
};

use itertools::Itertools;

type Board = HashMap<Point, Piece>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    x: isize,
    y: isize,
    z: isize,
}

impl Index<usize> for Point {
    type Output = isize;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Point has 3 values but the index was {index}"),
        }
    }
}

impl IndexMut<usize> for Point {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Point has 3 values but the index was {index}"),
        }
    }
}

// TODO: every function that mutates a Point must canonicalize the result
impl Point {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Self { x, y, z }.canonicalize()
    }

    pub fn canonicalize(&self) -> Self {
        let mut new = *self;
        while new.y > 0 && new.z < 0 {
            new.y -= 1;
            new.z += 1;
            new.x += 1;
        }
        new
    }

    pub fn neighbors(&self) -> Vec<Self> {
        vec![-1, 1]
            .into_iter()
            .cartesian_product(0..3)
            .map(|(dir, axis)| {
                let mut new = *self;
                new[axis] += dir;
                new.canonicalize()
            })
            .collect()
    }

    pub fn movable_neighbors(&self, board: &Board) -> impl Iterator<Item = Point> {
        self.neighbors()
            .into_iter()
            .map(|p| (p, board.get(&p).is_none()))
            .circular_tuple_windows()
            .filter(|((_, a), (_, b))| *a && *b)
            .flat_map(|((p1, _), (p2, _))| vec![p1, p2])
            .unique()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Player {
    P1,
    P2,
}

impl Not for Player {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::P1 => Self::P2,
            Self::P2 => Self::P1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Piece {
    Queen(Player),
    Beetle(Player, Option<Box<Piece>>),
    Ant(Player),
    Grasshopper(Player),
    Spider(Player),
}

pub struct State {
    turn: Player,
    board: Board,
}

impl State {
    pub fn new(turn: Player, board: Board) -> Self {
        Self { turn, board }
    }

    pub fn get_moves(&self) -> Vec<State> {
        let mut v = Vec::new();
        for (point, piece) in self.board.iter() {
            v.extend(
                match piece {
                    Piece::Queen(_) => point
                        .movable_neighbors(&self.board)
                        .map(|p| {
                            let mut b = self.board.clone();
                            let piece = b.remove(point).unwrap();
                            b.insert(p, piece);
                            b
                        })
                        .collect_vec(),
                    Piece::Beetle(player, under) => point
                        .neighbors()
                        .into_iter()
                        .map(|p| {
                            let mut b = self.board.clone();
                            let piece = b.remove(point).map(Box::new);
                            b.insert(p, Piece::Beetle(*player, piece));
                            if let Some(u) = under {
                                b.insert(*point, *u.clone());
                            }
                            b
                        })
                        .collect_vec(),
                    Piece::Ant(_) => {
                        fn ant_moves(
                            point: Point,
                            board: &Board,
                            visited: &mut HashSet<Point>,
                        ) -> Vec<Board> {
                            point
                                .movable_neighbors(board)
                                .filter(|p| {
                                    p.neighbors()
                                        .into_iter()
                                        .any(|p| board.get(&p).is_some() && visited.insert(p))
                                })
                                // HACK: consume the iterator so that visited isn't borrowed mutably more
                                // than once
                                .collect_vec()
                                .into_iter()
                                .flat_map(|p| {
                                    let mut b = board.clone();
                                    let e = b.remove(&point).unwrap();
                                    b.insert(p, e);
                                    p.movable_neighbors(&b)
                                        .flat_map(|neighbor| ant_moves(neighbor, &b, visited))
                                        .collect_vec()
                                        .into_iter()
                                        .chain(once(b))
                                        .collect_vec()
                                })
                                .collect_vec()
                        }

                        let mut cache = HashSet::new();
                        ant_moves(*point, &self.board, &mut cache)
                    }
                    Piece::Grasshopper(_) => vec![-1, 1]
                        .into_iter()
                        .cartesian_product(0..3)
                        .map(|(dir, axis)| {
                            let mut p = *point;
                            while self.board.contains_key(&p) {
                                p[axis] += dir;
                            }
                            let mut b = self.board.clone();
                            let piece = b.remove(point).unwrap();
                            b.insert(p, piece);
                            b
                        })
                        .collect_vec(),
                    Piece::Spider(_) => {
                        fn spider_moves(
                            point: Point,
                            board: &Board,
                            visited: &mut HashSet<Point>,
                            moves_remaining: usize,
                        ) -> Vec<Board> {
                            if moves_remaining > 0 {
                                point
                                    .movable_neighbors(board)
                                    .filter(|p| {
                                        p.neighbors()
                                            .into_iter()
                                            .any(|p| board.get(&p).is_some() && visited.insert(p))
                                    })
                                    // HACK: consume the iterator so that visited isn't borrowed mutably more
                                    // than once
                                    .collect_vec()
                                    .into_iter()
                                    .flat_map(|p| {
                                        let mut b = board.clone();
                                        let e = b.remove(&point).unwrap();
                                        b.insert(p, e);
                                        p.movable_neighbors(&b)
                                            .flat_map(|neighbor| {
                                                spider_moves(
                                                    neighbor,
                                                    &b,
                                                    visited,
                                                    moves_remaining - 1,
                                                )
                                            })
                                            .collect_vec()
                                            .into_iter()
                                            .chain(once(b))
                                            .collect_vec()
                                    })
                                    .collect_vec()
                            } else {
                                Vec::new()
                            }
                        }

                        let mut cache = HashSet::new();
                        spider_moves(*point, &self.board, &mut cache, 3)
                    }
                }
                .into_iter()
                .map(|b| State::new(!self.turn, b)),
            );
        }

        v
    }
}
