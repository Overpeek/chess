use crate::{Board, BoardPos};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Side {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

//

impl Side {
    pub const fn other(self) -> Self {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

impl Piece {
    pub fn moves(
        self,
        board: &Board,
        pos: BoardPos,
        side: Side,
    ) -> impl Iterator<Item = BoardPos> + '_ {
        let x = pos.file as i32;
        let y = pos.rank as i32;

        let filter = |s: Option<BoardPos>| s;

        let piece_moves: Box<dyn Iterator<Item = BoardPos>> = match (self, side) {
            // pawn moves
            (Piece::Pawn, Side::Black) => {
                // eat to left
                let bottom_left = BoardPos::new(x - 1, y - 1);
                let bottom_left = if let Some((Side::White, _)) =
                    bottom_left.and_then(|pos| board.get_piece(&pos))
                {
                    Some(bottom_left)
                } else {
                    None
                };

                // eat to right
                let bottom_right = BoardPos::new(x + 1, y - 1);
                let bottom_right = if let Some((Side::White, _)) =
                    bottom_right.and_then(|pos| board.get_piece(&pos))
                {
                    Some(bottom_right)
                } else {
                    None
                };

                // first move can be 2
                let first_move = BoardPos::new(x, y - 2);
                let first_move = if pos.rank == 7
                    && first_move.and_then(|pos| board.get_piece(&pos)).is_none()
                {
                    Some(first_move)
                } else {
                    None
                };

                // any other move is 1
                let other_move = BoardPos::new(x, y - 1);
                let other_move = if other_move.and_then(|pos| board.get_piece(&pos)).is_none() {
                    Some(other_move)
                } else {
                    None
                };

                Box::new(
                    other_move
                        .into_iter()
                        .chain(first_move)
                        .chain(bottom_left)
                        .chain(bottom_right)
                        .collect::<Vec<_>>() // TODO: fix
                        .into_iter()
                        .filter_map(filter),
                )
            }
            (Piece::Pawn, Side::White) => {
                // eat to left
                let bottom_left = BoardPos::new(x - 1, y + 1);
                let bottom_left = if let Some((Side::Black, _)) =
                    bottom_left.and_then(|pos| board.get_piece(&pos))
                {
                    Some(bottom_left)
                } else {
                    None
                };

                // eat to right
                let bottom_right = BoardPos::new(x + 1, y + 1);
                let bottom_right = if let Some((Side::Black, _)) =
                    bottom_right.and_then(|pos| board.get_piece(&pos))
                {
                    Some(bottom_right)
                } else {
                    None
                };

                // first move can be 2
                let first_move = BoardPos::new(x, y + 2);
                let first_move = if pos.rank == 2
                    && first_move.and_then(|pos| board.get_piece(&pos)).is_none()
                {
                    Some(first_move)
                } else {
                    None
                };

                // any other move is 1
                let other_move = BoardPos::new(x, y + 1);
                let other_move = if other_move.and_then(|pos| board.get_piece(&pos)).is_none() {
                    Some(other_move)
                } else {
                    None
                };

                Box::new(
                    other_move
                        .into_iter()
                        .chain(first_move)
                        .chain(bottom_left)
                        .chain(bottom_right)
                        .collect::<Vec<_>>() // TODO: fix
                        .into_iter()
                        .filter_map(filter),
                )
            }

            // knight moves
            (Piece::Knight, _) => Box::new(
                vec![
                    BoardPos::new(x - 1, y + 2),
                    BoardPos::new(x + 1, y + 2),
                    BoardPos::new(x - 1, y - 2),
                    BoardPos::new(x + 1, y - 2),
                    BoardPos::new(x - 2, y - 1),
                    BoardPos::new(x - 2, y + 1),
                    BoardPos::new(x + 2, y - 1),
                    BoardPos::new(x + 2, y + 1),
                ]
                .into_iter()
                .filter_map(filter),
            ),

            // bishop moves
            (Piece::Bishop, _) => Box::new(Self::sliding_moves(board, pos, false, true)),

            // rook moves
            (Piece::Rook, _) => Box::new(Self::sliding_moves(board, pos, true, false)),

            // queen moves
            (Piece::Queen, _) => Box::new(Self::sliding_moves(board, pos, true, true)),

            // king moves
            (Piece::King, _) => Box::new(
                vec![
                    BoardPos::new(x - 1, y - 1),
                    BoardPos::new(x, y - 1),
                    BoardPos::new(x + 1, y - 1),
                    BoardPos::new(x - 1, y),
                    BoardPos::new(x + 1, y),
                    BoardPos::new(x - 1, y + 1),
                    BoardPos::new(x, y + 1),
                    BoardPos::new(x + 1, y + 1),
                ]
                .into_iter()
                .filter_map(filter),
            ),
        };

        piece_moves.filter(move |new_pos| {
            if let Some((s, _)) = board.get_piece(new_pos) {
                // do not allow eating own pieces
                s != side
            } else {
                true
            }
        })
    }

    pub fn sliding_moves(
        board: &Board,
        pos: BoardPos,
        level: bool,
        diagl: bool,
    ) -> impl Iterator<Item = BoardPos> + '_ {
        let x = pos.file as i32;
        let y = pos.rank as i32;

        let level = if level { 1 } else { 1000 };
        let diagl = if diagl { 1 } else { 1000 };

        let left = (level..)
            .map_while(move |i| BoardPos::new(x + i, y))
            .take_while_p(|pos| board.get_piece(pos).is_none());
        let right = (level..)
            .map_while(move |i| BoardPos::new(x - i, y))
            .take_while_p(|pos| board.get_piece(pos).is_none());
        let up = (level..)
            .map_while(move |i| BoardPos::new(x, y + i))
            .take_while_p(|pos| board.get_piece(pos).is_none());
        let down = (level..)
            .map_while(move |i| BoardPos::new(x, y - i))
            .take_while_p(|pos| board.get_piece(pos).is_none());
        let up_left = (diagl..)
            .map_while(move |i| BoardPos::new(x + i, y + i))
            .take_while_p(|pos| board.get_piece(pos).is_none());
        let up_right = (diagl..)
            .map_while(move |i| BoardPos::new(x - i, y + i))
            .take_while_p(|pos| board.get_piece(pos).is_none());
        let down_left = (diagl..)
            .map_while(move |i| BoardPos::new(x + i, y - i))
            .take_while_p(|pos| board.get_piece(pos).is_none());
        let down_right = (diagl..)
            .map_while(move |i| BoardPos::new(x - i, y - i))
            .take_while_p(|pos| board.get_piece(pos).is_none());

        left.chain(right)
            .chain(up)
            .chain(down)
            .chain(up_left)
            .chain(up_right)
            .chain(down_left)
            .chain(down_right)
    }
}

struct TakeWhilePlusOne<T, I, P>
where
    I: Iterator<Item = T>,
    P: FnMut(&T) -> bool,
{
    p: P,
    i: I,
    take: bool,
    _p: PhantomData<T>,
}

trait IntoTakeWhilePlusOne<T, P>
where
    Self: Iterator<Item = T> + Sized,
    P: FnMut(&T) -> bool,
{
    fn take_while_p(self, p: P) -> TakeWhilePlusOne<T, Self, P>;
}

impl<I, T, P> IntoTakeWhilePlusOne<T, P> for I
where
    I: Iterator<Item = T>,
    P: FnMut(&T) -> bool,
{
    fn take_while_p(self, p: P) -> TakeWhilePlusOne<T, Self, P> {
        TakeWhilePlusOne {
            p,
            i: self,
            take: true,
            _p: Default::default(),
        }
    }
}

impl<T, I, P> Iterator for TakeWhilePlusOne<T, I, P>
where
    I: Iterator<Item = T>,
    P: FnMut(&T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.take {
            None
        } else {
            let next = self.i.next()?;
            self.take = (self.p)(&next);
            Some(next)
        }
    }
}
