use std::fmt;
use std::fmt::Display;

use rand::Rng;

#[derive(Hash, Eq, PartialEq, Debug, PartialOrd, Ord, Clone, Copy)]
pub struct Pos(pub u64);

pub const START: Pos = Pos(0xfedcba9876543210);

impl Pos {
    pub fn hole_index(&self) -> usize {
        for i in 0..16 {
            if (self.0 >> (4 * i)) & 15 == 15 {
                return i;
            }
        }
        unreachable!()
    }

    pub fn swap(mut self, from: usize, to: usize) -> Pos {
        let to_mask = !(((!(self.0 >> (4 * to))) & 15) << (4 * from));
        self.0 &= to_mask;
        let from_mask = 15 << (4 * to);
        self.0 |= from_mask;
        self
    }

    pub fn manhattan(self, target: Pos) -> usize {
        let mut res: i8 = 0;

        let mut invers_target: u64 = 0;
        for i in 0..16 {
            invers_target |= i << (4 * ((target.0 >> (4 * i)) & 15));
        }

        for pos in 0..16 {
            let curr_val = (self.0 >> (4 * pos)) & 15;
            let pos_in_target = (invers_target >> (4 * curr_val)) & 15;
            res += ((pos & 3) as i8 - (pos_in_target & 3) as i8).abs()
                + ((pos >> 2) as i8 - (pos_in_target >> 2) as i8).abs();
        }

        (res as usize) / 2
    }

    pub fn from_string(input: String) -> Pos {
        let vec: Vec<u64> = input
            .split_whitespace()
            .map(|word| word.parse().unwrap())
            .map(|num: u64| if num == 0 { 15 } else { num - 1 })
            .collect();

        Self::from_permutation(vec)
    }

    pub fn from_permutation(input: Vec<u64>) -> Pos {
        let mut acc: Pos = Pos(0);
        for i in 0..16 {
            acc.0 |= input[i] << (4 * i);
        }
        acc
    }

    pub fn to_permutation(self) -> Vec<u64> {
        let mut res = Vec::new();
        for i in 0..16 {
            let val = (self.0 >> (4 * i)) & 15;
            res.push(if val == 15 { 0 } else { val + 1 });
        }
        res
    }

    pub fn apply(self, d: Dir) -> Pos {
        let hole_ind = self.hole_index();

        match d {
            Dir::Down => {
                if hole_ind >> 2 == 3 {
                    self
                } else {
                    self.swap(hole_ind, hole_ind + 4)
                }
            }
            Dir::Left => {
                if hole_ind & 3 == 0 {
                    self
                } else {
                    self.swap(hole_ind, hole_ind - 1)
                }
            }
            Dir::Up => {
                if hole_ind >> 2 == 0 {
                    self
                } else {
                    self.swap(hole_ind, hole_ind - 4)
                }
            }
            Dir::Right => {
                if hole_ind & 3 == 3 {
                    self
                } else {
                    self.swap(hole_ind, hole_ind + 1)
                }
            }
            _ => self,
        }
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let perm = self.to_permutation();
        for i in (0..16).step_by(4) {
            write!(
                f,
                "{:0>2}|{:0>2}|{:0>2}|{:0>2}\n",
                perm[i],
                perm[i + 1],
                perm[i + 2],
                perm[i + 3]
            )?;
            if i + 4 < 16 {
                write!(f, "--+--+--+--\n")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
    End,
}

impl Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dir::Up => write!(f, "Up"),
            Dir::Right => write!(f, "Righ"),
            Dir::Down => write!(f, "Down"),
            Dir::Left => write!(f, "Left"),
            Dir::End => write!(f, "End"),
        };
        Ok(())
    }
}

impl Dir {
    fn from_num(n: u32) -> Dir {
        match n % 4 {
            0 => Dir::Up,
            1 => Dir::Right,
            2 => Dir::Down,
            3 => Dir::Left,
            _ => unreachable!(),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Dir::Up => Dir::Right,
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Left,
            Dir::Left => Dir::End,
            Dir::End => Dir::End,
        }
    }

    pub fn random_path(n: usize) -> Vec<Dir> {
        let mut rng = rand::thread_rng();
        (0..n)
            .map(|_| rng.gen::<u32>())
            .map(Dir::from_num)
            .collect()
    }
}

pub struct Neighbors {
    center: Pos,
    curr_dir: Dir,
}

impl Neighbors {
    pub fn new(center: Pos, curr_dir: Dir) -> Neighbors {
        Neighbors { center, curr_dir }
    }

    pub fn get_dir(&self) -> Dir {
        match self.curr_dir {
            Dir::Up => Dir::End,
            Dir::Right => Dir::Up,
            Dir::Down => Dir::Right,
            Dir::Left => Dir::Down,
            Dir::End => Dir::Left,
        }
    }
}

impl Iterator for Neighbors {
    type Item = Pos;

    fn next(&mut self) -> Option<Pos> {
        if let Dir::End = self.curr_dir {
            None
        } else {
            let res = self.center.apply(self.curr_dir);
            self.curr_dir = self.curr_dir.next();

            if self.center == res {
                self.next()
            } else {
                Some(res)
            }
        }
    }
}

pub fn neighbors(pos: Pos) -> Neighbors {
    Neighbors::new(pos, Dir::Up)
}

pub fn maze() -> Pos {
    Dir::random_path(200).into_iter().fold(START, Pos::apply)
}

#[cfg(test)]
mod tests {
    use crate::position::{maze, neighbors, START};

    #[test]
    fn display_test() {
        println!("{}", START);
        println!("{}", maze());
    }

    #[test]
    fn neighbors_test() {
        for i in neighbors(START) {
            println!("{}", i);

            for j in neighbors(i) {
                println!("{}", j);
            }
        }
    }
}
