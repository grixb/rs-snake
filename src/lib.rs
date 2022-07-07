use std::iter::Iterator;

#[derive(Copy, Clone, Debug)]
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

pub trait Segment: Copy {
    fn dir(self) -> Direction;
    fn len(self) -> usize;    
}
impl<T: Copy> Segment for T where T: Into<(Direction, usize)> {
    fn dir(self) -> Direction {
        self.into().0
    }

    fn len(self) -> usize {
        self.into().1
    }
}


pub trait SegmentIter<'a, S: Segment + 'a>: Iterator<Item = &'a S> {}
impl<'a, T, S: Segment + 'a> SegmentIter<'a, S> for T where T: Iterator<Item = &'a S> {}

pub type Pos = (isize, isize);
trait NextPos {
    fn next_pos(self, dir: Direction) -> Pos;
}
impl<T> NextPos for T
where
    T: Into<Pos>,
{
    fn next_pos(self, dir: Direction) -> Pos {
        use Direction::*;
        let mut pos: Pos = self.into();
        match dir {
            Up => pos.1 += 1,
            Down => pos.1 -= 1,
            Left => pos.0 -= 1,
            Right => pos.0 += 1,
        };
        pos
    }
}

pub struct PosIter<I, S> {
    seg_itr: I,
    seg: Option<S>,
    pos: Pos,
    idx: usize,
    is_last: bool,
}

impl<'a, S: Segment + 'a, I: SegmentIter<'a, S>> Iterator for PosIter<I, S> {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.seg.is_none() {
                if let Some(nxt_seg) = self.seg_itr.next() {
                    self.seg = Some(*nxt_seg);
                    continue;
                } else {
                    if !self.is_last {
                        self.is_last = true;
                        break Some(self.pos);
                    } else {
                        break None;
                    }
                }
            } else {
                if self.idx + 1 >= self.seg.unwrap().len() {
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

impl<'a, S: Segment + 'a, I: SegmentIter<'a, S>> PosIter<I, S> {
    pub fn new(seg_iter: I, start: Pos) -> PosIter<I, S> {
        PosIter {
            seg_itr: seg_iter,
            seg: None,
            pos: start,
            idx: 0,
            is_last: false,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Direction, Pos, PosIter};

    #[test]
    fn pos_iter_test() {
        use Direction::*;
        let segs: &[(Direction, usize)] = &[
            (Right, 3),
            (Down, 2),
            (Right, 3),
            (Down, 4),
        ];

        let expected: &[Pos] = &[
            (4, -4),
            (3, -4),
            (2, -4),
            (2, -3),
            (1, -3),
            (0, -3),
            (0, -2),
            (0, -1),
            (0, 0),
        ];

        let itr = PosIter::new(segs.iter(), (4, -4));
        let actual: Vec<Pos> = itr.collect();

        assert_eq!(expected, actual);
    }
}
