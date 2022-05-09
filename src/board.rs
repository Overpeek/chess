use crate::piece::{Piece, Side};
use core::fmt;
use std::collections::HashMap;

//

/* #[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompressedBoardPiece {
    Empty { count: u8 },
    Piece { piece: Piece, side: Side },
    Row,
} */

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    // pieces: Vec<CompressedBoardPiece>,
    pieces: HashMap<BoardPos, (Side, Piece)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardPos {
    pub file: u8,
    pub rank: u8,
}

//

impl Board {
    pub fn starting() -> Self {
        Self::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap() // w KQkq - 0 1
    }

    pub fn parse_fen(fen: &str) -> Option<Self> {
        let mut pieces = HashMap::default();
        let mut pos = BoardPos::default();
        for c in fen.chars() {
            let c: char = c;
            let lc = c.to_lowercase().next().unwrap();
            let is_lower = c == lc;

            let side = if is_lower { Side::White } else { Side::Black };

            match lc {
                c @ '1'..='8' => pos.file += c as u8 - b'0',
                '/' => {
                    pos.rank += 1;
                    pos.file = 1;
                }
                _ => {
                    let piece = match lc {
                        'r' => Some(Piece::Rook),
                        'n' => Some(Piece::Knight),
                        'b' => Some(Piece::Bishop),
                        'q' => Some(Piece::Queen),
                        'k' => Some(Piece::King),
                        'p' => Some(Piece::Pawn),
                        _ => None,
                    };

                    piece.map(|piece| pieces.insert(pos, (side, piece)));
                    pos.file += 1;
                }
            };
        }

        Some(Self { pieces })
    }

    pub fn iter(&self) -> impl Iterator<Item = (Side, Piece, BoardPos)> + '_ {
        /* let mut pos = BoardPos::default();
        self.pieces.iter().filter_map(move |piece| match piece {
            CompressedBoardPiece::Empty { count } => {
                pos.file += count;
                None
            }
            CompressedBoardPiece::Row => {
                pos.file = 0;
                pos.rank += 1;
                None
            }
            CompressedBoardPiece::Piece { side, piece } => {
                let res = Some((*side, *piece, pos));
                pos.file += 1;
                res
            }
        }) */

        self.pieces
            .iter()
            .map(|(&pos, &(side, piece))| (side, piece, pos))
    }

    pub fn get_piece(&self, pos: &BoardPos) -> Option<(Side, Piece)> {
        /* self.iter()
        .find(|(_, _, p)| p == pos)
        .map(|(side, piece, _)| (side, piece))
        .unwrap() */

        self.pieces.get(pos).cloned()
    }

    pub fn set_piece(&mut self, side: Side, piece: Piece, pos: BoardPos) {
        /* self.iter().filter(|s|);

        self.iter()
            .find(|(_, _, p)| p == pos)
            .map(|(side, piece, _)| (side, piece))
            .unwrap() */
        self.pieces.insert(pos, (side, piece));
    }

    pub fn remove_piece(&mut self, pos: &BoardPos) -> Option<(Side, Piece)> {
        self.pieces.remove(pos)
    }
}

impl BoardPos {
    pub fn iter() -> impl Iterator<Item = BoardPos> {
        (1..=8)
            .flat_map(|y| (1..=8).map(move |x| (x, y)))
            .map(|(x, y)| Self::new(x, y).unwrap())
    }

    pub const fn new(x: i32, y: i32) -> Option<Self> {
        if x < 1 || y < 1 || x > 8 || y > 8 {
            None
        } else {
            Some(Self {
                file: x as u8,
                rank: y as u8,
            })
        }
    }

    pub const fn to_usize(self) -> usize {
        self.file as usize + self.rank as usize * 8 - 9
    }
}

impl Iterator for Board {
    type Item = (Side, Piece);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Default for BoardPos {
    fn default() -> Self {
        Self { file: 1, rank: 1 }
    }
}

impl fmt::Display for BoardPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", (b'a' + self.file - 1) as char, self.rank)
    }
}
