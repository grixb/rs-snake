use std::{iter::Iterator, collections::VecDeque, fmt::Display,};
use rand::prelude::*;
use rand_xorshift::XorShiftRng;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Self {
        use Direction::*;
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}

pub trait Segment {
    fn dir(&self) -> Direction;
    fn len(&self) -> usize;
}

pub type Seg = (Direction, usize);

impl Segment for (Direction, usize) {
    #[inline]
    fn dir(&self) -> Direction {
        self.0
    }

    #[inline]
    fn len(&self) -> usize {
        self.1
    }
}

pub trait SegmentIter<'a, S: Segment + 'a>: Iterator<Item = &'a S> {}
impl<'a, T, S: Segment + 'a> SegmentIter<'a, S> for T where T: Iterator<Item = &'a S> {}

pub trait Position {
    fn x(&self) -> isize;
    fn y(&self) -> isize;
}

// (x, y)
pub type Pos = (isize, isize);

impl Position for (isize, isize) {
    #[inline]
    fn x(&self) -> isize {
        self.0
    }

    #[inline]
    fn y(&self) -> isize {
        self.1
    }
}

trait NextPos<P: Position> {
    fn next_pos(self, dir: Direction) -> P;
}
impl NextPos<(isize, isize)> for (isize, isize) {
    fn next_pos(self, dir: Direction) -> (isize, isize) {
        use Direction::*;
        let mut pos = self.clone();
        match dir {
            Up => pos.1 += 1,
            Down => pos.1 -= 1,
            Left => pos.0 -= 1,
            Right => pos.0 += 1,
        };
        pos
    }
}

pub struct PosIter<'a, I, S, P> {
    seg_itr: I,
    seg: Option<&'a S>,
    pos: P,
    idx: usize,
}

impl<'a, S, I, P> Iterator for PosIter<'a, I, S, P>
where
    S: Segment + 'a,
    I: SegmentIter<'a, S>,
    P: Position + NextPos<P> + Copy,
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.seg.is_none() {
                if let Some(nxt_seg) = self.seg_itr.next() {
                    self.seg = Some(nxt_seg);
                    continue;
                } else { break None; }
            } else {
                if self.idx >= self.seg.unwrap().len() {
                    self.idx = 0;
                    self.seg = None;
                    continue;
                } else {
                    let actual_pos = self.pos;
                    self.pos = self.pos.next_pos(self.seg.unwrap().dir().opposite());
                    self.idx += 1;
                    break Some(actual_pos);
                }
            }
        }
    }
}

impl<'a, S, I, P> PosIter<'a, I, S, P>
where
    S: Segment + 'a,
    I: SegmentIter<'a, S>,
    P: Position,
{
    pub fn new(seg_iter: I, start: P) -> PosIter<'a, I, S, P> {
        PosIter {
            seg_itr: seg_iter,
            seg: None,
            pos: start,
            idx: 0,
        }
    }
}
pub trait PosIterBuilder<'a, S, I, P>
where
    S: Segment + 'a,
    I: SegmentIter<'a, S>,
    P: Position,
{
    fn iter_from_start(self, pos: P) -> PosIter<'a, I, S, P>;
}

impl<'a, T, S, I> PosIterBuilder<'a, S, I, Pos> for T 
where 
    S: Segment + 'a,
    I: SegmentIter<'a, S>,
    T: IntoIterator<Item = &'a S, IntoIter = I>,
{
    fn iter_from_start(self, pos: Pos) -> PosIter<'a, I, S, Pos> {
        PosIter::new(self.into_iter(), pos)
    }
}


#[cfg(test)]
mod pos_iter_test {
    use crate::{Direction, Pos, PosIterBuilder, Seg};

    #[test]
    fn test_pos_iter() {
        use Direction::*;
        let segs: &[Seg] = &[(Right, 2), (Down, 1), (Right, 2), (Down, 3)];

        let expected: &[Pos] = &[
            (4, -4),
            (3, -4),
            (2, -4),
            (2, -3),
            (1, -3),
            (0, -3),
            (0, -2),
            (0, -1),
        ];

        let actual: Vec<Pos> = segs.iter_from_start((4, -4)).collect();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_one_len_segment() {
        use Direction::*;
        let segs: &[Seg] = &[(Left, 1), (Up, 9)];

        let expected: &[Pos] = &[
            (-1,1),
            (0,1),
            (0,0),
            (0,-1),
            (0,-2),
            (0,-3),
            (0,-4),
            (0,-5),
            (0,-6),
            (0,-7)
        ];

        let actual: Vec<Pos> = segs.iter_from_start((-1,1)).collect();

        assert_eq!(expected, actual);
    }
}

// (col, row)
pub type Cell = (u16, u16);

pub trait Cellular: From<Cell> {
    fn col(&self) -> u16;
    fn row(&self) -> u16;
}

impl Cellular for Cell {
    #[inline]
    fn col(&self) -> u16 {
        self.0
    }

    #[inline]
    fn row(&self) -> u16 {
        self.1
    }
}

pub struct CellIter<I, C> {
    pos_itr: I,
    bound: C,
}

impl<I, P, C> Iterator for CellIter<I, C>
where
    P: Position,
    I: Iterator<Item = P>,
    C: Cellular,
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos_itr.next().map(|p| {
            (
                (self.bound.col() as isize / 2 + (p.x() * 2)).rem_euclid(self.bound.col() as isize)
                    as u16,
                (self.bound.row() as isize / 2 - p.y()).rem_euclid(self.bound.row() as isize)
                    as u16,
            )
                .into()
        })
    }
}

impl<I, P, C> CellIter<I, C>
where
    P: Position,
    I: Iterator<Item = P>,
    C: Cellular,
{
    pub fn new(pos_itr: I, bound: C) -> Self {
        Self { pos_itr, bound }
    }
}

impl<'a, S, I> PosIter<'a, I, S, Pos>
where
    S: Segment + 'a,
    I: SegmentIter<'a, S>,
{
    pub fn within_bound(self, bnd: Cell) -> CellIter<PosIter<'a, I, S, Pos>, Cell> {
        CellIter::new(self, bnd)
    }
} 

#[cfg(test)]
mod cell_iter_test {
    use crate::{Cell, PosIterBuilder, Direction, Seg};

    #[test]
    fn test_cell_iter() {
        use Direction::*;
        let segs: &[Seg] = &[(Up, 2), (Left, 3), (Up, 2), (Right, 6)];

        let expected: &[Cell] = &[
            (10, 5),
            (10, 6),
            (10, 7),
            (12, 7),
            (14, 7),
            (16, 7),
            (16, 8),
            (16, 9),
            (14, 9),
            (12, 9),
            (10, 9),
            (8, 9),
            (6, 9),
        ];

        let actual: Vec<Cell> = segs
            .iter_from_start((0, 0))
            .within_bound((20, 10))
            .collect();

        assert_eq!(expected, actual);
    }
}

pub struct Snake {
    head: Pos,
    segs: VecDeque<Seg>
}

impl Snake {

    pub fn new<P: Into<Pos>>(start: P, len: usize) -> Snake {
        Self { head: start.into(), segs: VecDeque::from([(Direction::Up, len)]) }
    }

    pub fn snaking(&mut self, new_dir: Option<Direction>) {
        if let (Some(new_dir), Some(top_seg)) = (new_dir, self.segs.front()) {
            if top_seg.dir() != new_dir && top_seg.dir() != new_dir.opposite() {
                self.segs.push_front((new_dir, 0));
            }
        }

        if let Some(top_seg) = self.segs.front_mut() {
            self.head = self.head.next_pos(top_seg.dir());
            top_seg.1 += 1;
        }
        if let Some(mut lst_seg) = self.segs.pop_back() {
            if lst_seg.len() > 1 {
                lst_seg.1 -= 1;
                self.segs.push_back(lst_seg);
            } else {
                drop(lst_seg);
            }
        }
    }

    pub fn is_collide(&self) -> bool {
        self.segs.iter_from_start(self.head)
        .skip(1)
        .any(|p| p == self.head)
    }

    pub fn grow(&mut self) {
        if let Some(lst_seg) = self.segs.back_mut() {
            lst_seg.1 += 1;
        }
    }

    pub fn formatter(&self, bound: Cell, elems: [char; 5]) -> SnakeFormatter<'_> {
        SnakeFormatter { snk: self, bnd: bound, lms: elems }
    }
}

pub struct SnakeFormatter<'a> { 
    snk: &'a Snake,
    bnd: Cell,
    lms: [char; 5]
}

impl<'a> Display for SnakeFormatter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let head_char = if let Some((head_dir, _)) = self.snk.segs.front() {
            self.lms[*head_dir as usize]
        } else {
            self.lms[4]
        };

        let mut cell_itr = self.snk.segs
            .iter_from_start(self.snk.head)
            .within_bound(self.bnd);

        if let Some(head_cell) = cell_itr.next() {
            write!(f,"\x1b[{};{}H{}", head_cell.row(), head_cell.col(), head_char)?;
            for body_cell in cell_itr {
                write!(f,"\x1b[{};{}H{}", body_cell.row(), body_cell.col(), self.lms[4])?;
            }
            Ok(())
        } else {
            write!(f,"")
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Food { 
    pos: Cell,
    bnd: Cell
}

impl Food {
    pub fn somewhere_within(bound: Cell) -> Self {
        let mut rng = XorShiftRng::from_entropy();
        Self { 
            bnd: bound,
            pos: (
                ((rng.next_u32() / 3 * 2 + 1)).rem_euclid(bound.col() as u32) as u16,
                (rng.next_u32()).rem_euclid(bound.row() as u32) as u16,
            ),
        }
    }

    pub fn is_eaten_by(&self, snake: &Snake) -> bool {
        snake.segs
        .iter_from_start(snake.head)
        .within_bound(self.bnd)
        .next()
        .map_or(false, |head_cell| head_cell == self.pos)
    }    
}

impl Display for Food {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"\x1b[{};{}H@", self.pos.row(), self.pos.col())
    }
}
