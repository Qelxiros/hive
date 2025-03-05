use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    iter::once,
    ops::{Deref, DerefMut, Index, IndexMut, Not},
};

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Board {
    map: HashMap<Point, Piece>,
}

impl Deref for Board {
    type Target = HashMap<Point, Piece>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.map.iter().sorted().for_each(|entry| entry.hash(state));
    }
}

impl PartialOrd for Board {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Board {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.map.iter().sorted().cmp(other.map.iter().sorted())
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Piece {
    Queen(Player),
    Beetle(Player, Option<Box<Piece>>),
    Ant(Player),
    Grasshopper(Player),
    Spider(Player),
}

impl Piece {
    pub fn player(&self) -> Player {
        match self {
            Self::Queen(player) => *player,
            Self::Beetle(player, _) => *player,
            Self::Ant(player) => *player,
            Self::Grasshopper(player) => *player,
            Self::Spider(player) => *player,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pieces {
    p1: Vec<Piece>,
    p2: Vec<Piece>,
}

impl Pieces {
    pub fn new() -> Self {
        Self {
            p1: vec![
                [Piece::Queen(Player::P1); 1].as_slice(),
                [const { Piece::Beetle(Player::P1, None) }; 2].as_slice(),
                [const { Piece::Ant(Player::P1) }; 3].as_slice(),
                [const { Piece::Grasshopper(Player::P1) }; 3].as_slice(),
                [const { Piece::Spider(Player::P1) }; 2].as_slice(),
            ]
            .into_iter()
            .flatten()
            .cloned()
            .collect_vec(),
            p2: vec![
                [Piece::Queen(Player::P2); 1].as_slice(),
                [const { Piece::Beetle(Player::P2, None) }; 2].as_slice(),
                [const { Piece::Ant(Player::P2) }; 3].as_slice(),
                [const { Piece::Grasshopper(Player::P2) }; 3].as_slice(),
                [const { Piece::Spider(Player::P2) }; 2].as_slice(),
            ]
            .into_iter()
            .flatten()
            .cloned()
            .collect_vec(),
        }
    }

    pub fn remove(&mut self, player: Player, idx: usize) -> Piece {
        // FIXME: using swap_remove here breaks equality checks later
        // figure out if the extra O(n) here outweighs the alternative O(n log n) of sorting at
        // check time
        match player {
            Player::P1 => self.p1.remove(idx),
            Player::P2 => self.p2.remove(idx),
        }
    }
}

impl Default for Pieces {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct State {
    turn: usize,
    active: Player,
    p1_queen: Option<Point>,
    p2_queen: Option<Point>,
    unplaced: Pieces,
    board: Board,
}

impl State {
    pub fn new(
        turn: Option<usize>,
        active: Player,
        p1_queen: Option<Point>,
        p2_queen: Option<Point>,
        unplaced: Pieces,
        board: Board,
    ) -> Self {
        Self {
            turn: turn.unwrap_or(0),
            active,
            p1_queen,
            p2_queen,
            unplaced,
            board,
        }
    }

    pub fn next_turn(&self, queen: Option<Point>, unplaced: Option<Pieces>, board: Board) -> Self {
        Self {
            turn: if self.active == Player::P2 {
                self.turn + 1
            } else {
                self.turn
            },
            active: !self.active,
            p1_queen: if self.active == Player::P1 && queen.is_some() {
                queen
            } else {
                self.p1_queen
            },
            p2_queen: if self.active == Player::P2 && queen.is_some() {
                queen
            } else {
                self.p2_queen
            },
            unplaced: unplaced.unwrap_or_else(|| self.unplaced.clone()),
            board,
        }
    }

    pub fn placeable_points(&self) -> Vec<Point> {
        self.board
            .iter()
            .filter(|(_, piece)| piece.player() == self.active)
            .filter(|(point, _)| {
                point.neighbors().iter().all(|p| {
                    self.board
                        .get(p)
                        .is_none_or(|piece| piece.player() == self.active)
                })
            })
            .map(|(&p, _)| p)
            .collect_vec()
    }

    fn component_size(&self, point: Option<Point>) -> usize {
        let Some(point) = point else {
            return 0;
        };
        let mut q = VecDeque::new();
        let mut visited = HashSet::new();
        q.push_back(point);
        visited.insert(point);
        while let Some(point) = q.pop_front() {
            point
                .neighbors()
                .into_iter()
                .filter(|p| self.board.contains_key(p) && visited.insert(*p))
                .for_each(|p| q.push_back(p));
        }
        visited.len()
    }

    pub fn validate(&self) -> bool {
        self.component_size(self.board.keys().nth(0).cloned()) == self.board.len()
            && match (self.turn, self.active, self.p1_queen, self.p2_queen) {
                (5.., _, Some(_), Some(_)) => true,
                (4, Player::P2, None, _) | (5.., _, _, _) => false,
                _ => true,
            }
    }

    pub fn get_moves(&self) -> HashSet<State> {
        let mut v = HashSet::new();
        v.extend(
            (0..match self.active {
                Player::P1 => &self.unplaced.p1,
                Player::P2 => &self.unplaced.p2,
            }
            .len())
                .cartesian_product(match (self.board.len(), self.active) {
                    (0..=1, Player::P1) => vec![Point::new(0, 0, 0)],
                    (0..=1, Player::P2) => vec![Point::new(0, 0, 1)],
                    _ => self.placeable_points(),
                })
                .map(|(idx, point)| {
                    let mut b = self.board.clone();
                    let mut pieces = self.unplaced.clone();
                    b.insert(point, pieces.remove(self.active, idx));
                    self.next_turn(
                        if let Piece::Queen(_) = b[&point] {
                            Some(point)
                        } else {
                            None
                        },
                        Some(pieces),
                        b,
                    )
                }),
        );
        for (point, piece) in self
            .board
            .iter()
            .filter(|&(_, piece)| piece.player() == self.active)
        {
            v.extend(
                match piece {
                    Piece::Queen(_) => point
                        .movable_neighbors(&self.board)
                        .map(|p| {
                            let mut b = self.board.clone();
                            let piece = b.remove(point).unwrap();
                            b.insert(p, piece);
                            (b, Some(p))
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
                            (b, None)
                        })
                        .collect_vec(),
                    Piece::Ant(_) => {
                        fn ant_moves(
                            point: Point,
                            original_board: &Board,
                            hypothetical_board: &Board,
                            visited: &mut HashSet<Point>,
                        ) -> Vec<Board> {
                            point
                                .movable_neighbors(hypothetical_board)
                                .filter(|neighbor| {
                                    neighbor
                                        .neighbors()
                                        .into_iter()
                                        .any(|p| original_board.get(&p).is_some())
                                        && visited.insert(*neighbor)
                                })
                                // HACK: consume the iterator so that visited isn't borrowed mutably more
                                // than once
                                .collect_vec()
                                .into_iter()
                                .flat_map(|p| {
                                    let mut b = hypothetical_board.clone();
                                    let e = b.remove(&point).unwrap();
                                    b.insert(p, e);
                                    ant_moves(p, original_board, &b, visited)
                                        .into_iter()
                                        .chain(once(b))
                                        .collect_vec()
                                })
                                .collect_vec()
                        }

                        let mut cache = HashSet::new();
                        ant_moves(*point, &self.board, &self.board, &mut cache)
                            .into_iter()
                            .map(|b| (b, None))
                            .collect_vec()
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
                            (b, None)
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
                            .into_iter()
                            .map(|b| (b, None))
                            .collect_vec()
                    }
                }
                .into_iter()
                .map(|(b, queen)| self.next_turn(queen, None, b)),
            );
        }

        v.into_iter().filter(|s| s.validate()).collect()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            turn: 0,
            active: Player::P1,
            p1_queen: None,
            p2_queen: None,
            unplaced: Pieces::default(),
            board: Board::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_move() {
        let state = State::default();

        let moves = state.get_moves();

        assert_eq!(moves.len(), 5);
    }

    #[test]
    fn test_second_move() {
        for state in State::default().get_moves().into_iter() {
            let moves = state.get_moves();

            assert_eq!(moves.len(), 5);
        }
    }

    #[test]
    fn test_validate_no_queen() {
        let state = State {
            turn: 5,
            active: Player::P2,
            ..Default::default()
        };

        assert!(!state.validate());
    }

    #[test]
    fn test_validate_valid() {
        let state = State::default();

        assert!(state.validate());
    }
}
