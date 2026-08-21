use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline]
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
    #[inline]
    pub fn as_usize(self) -> usize {
        self as usize
    }
    #[inline]
    pub fn as_index(self) -> usize {
        self as usize
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl PieceType {
    pub const ALL: [PieceType; 6] = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ];
    #[inline]
    pub fn as_usize(self) -> usize {
        self as usize
    }
    pub fn from_char(c: char) -> Option<PieceType> {
        match c.to_ascii_lowercase() {
            'p' => Some(PieceType::Pawn),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'r' => Some(PieceType::Rook),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            _ => None,
        }
    }
    pub fn to_char(self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }
    pub fn promo_char(self) -> char {
        match self {
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            _ => '?',
        }
    }
}

// Piece encoding: 0..11 + 12 empty
// WP=0, WN=1, WB=2, WR=3, WQ=4, WK=5, BP=6, BN=7, BB=8, BR=9, BQ=10, BK=11
pub const NO_PIECE: u8 = 12;

#[inline]
pub fn make_piece(color: Color, pt: PieceType) -> u8 {
    (color as u8) * 6 + (pt as u8)
}
#[inline]
pub fn piece_color(piece: u8) -> Color {
    if piece >= 6 {
        Color::Black
    } else {
        Color::White
    }
}
#[inline]
pub fn piece_type(piece: u8) -> PieceType {
    match piece % 6 {
        0 => PieceType::Pawn,
        1 => PieceType::Knight,
        2 => PieceType::Bishop,
        3 => PieceType::Rook,
        4 => PieceType::Queen,
        5 => PieceType::King,
        _ => unreachable!(),
    }
}
#[inline]
pub fn piece_type_of(piece: u8) -> PieceType {
    piece_type(piece)
}

// Squares: A1=0, B1=1 ... H1=7, A2=8 ... H8=63
pub type Square = u8;
pub const NO_SQUARE: u8 = 64;

pub const SQ_A1: u8 = 0;
pub const SQ_B1: u8 = 1;
pub const SQ_C1: u8 = 2;
pub const SQ_D1: u8 = 3;
pub const SQ_E1: u8 = 4;
pub const SQ_F1: u8 = 5;
pub const SQ_G1: u8 = 6;
pub const SQ_H1: u8 = 7;
pub const SQ_A8: u8 = 56;
pub const SQ_E8: u8 = 60;
pub const SQ_H8: u8 = 63;

#[inline]
pub fn square_file(sq: Square) -> u8 {
    sq % 8
}
#[inline]
pub fn square_rank(sq: Square) -> u8 {
    sq / 8
}
#[inline]
pub fn make_square(file: u8, rank: u8) -> Square {
    rank * 8 + file
}
#[inline]
pub fn square_name(sq: Square) -> String {
    let f = (b'a' + square_file(sq)) as char;
    let r = (b'1' + square_rank(sq)) as char;
    format!("{}{}", f, r)
}
pub fn square_from_name(s: &str) -> Option<Square> {
    if s.len() != 2 {
        return None;
    }
    let bytes = s.as_bytes();
    let f = bytes[0];
    let r = bytes[1];
    if !(b'a'..=b'h').contains(&f) || !(b'1'..=b'8').contains(&r) {
        return None;
    }
    Some(make_square(f - b'a', r - b'1'))
}

// Castling
pub const CASTLE_WK: u8 = 1;
pub const CASTLE_WQ: u8 = 2;
pub const CASTLE_BK: u8 = 4;
pub const CASTLE_BQ: u8 = 8;
pub const CASTLE_ALL: u8 = 15;

// Move encoding: bits 0-5 from, 6-11 to, 12-14 promo (0 none, 1 N,2 B,3 R,4 Q)
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move(pub u32);

impl Move {
    pub const NULL: Move = Move(0);

    #[inline]
    pub fn new(from: Square, to: Square, promo: Option<PieceType>) -> Self {
        let p = match promo {
            None => 0,
            Some(PieceType::Knight) => 1,
            Some(PieceType::Bishop) => 2,
            Some(PieceType::Rook) => 3,
            Some(PieceType::Queen) => 4,
            _ => 0,
        };
        Move(((from as u32) & 0x3F) | (((to as u32) & 0x3F) << 6) | ((p as u32) << 12))
    }

    #[inline]
    pub fn from_sq(self) -> Square {
        (self.0 & 0x3F) as Square
    }
    #[inline]
    pub fn to_sq(self) -> Square {
        ((self.0 >> 6) & 0x3F) as Square
    }
    #[inline]
    pub fn promotion(self) -> Option<PieceType> {
        match (self.0 >> 12) & 0x7 {
            0 => None,
            1 => Some(PieceType::Knight),
            2 => Some(PieceType::Bishop),
            3 => Some(PieceType::Rook),
            4 => Some(PieceType::Queen),
            _ => None,
        }
    }
    #[inline]
    pub fn is_null(self) -> bool {
        self.0 == 0
    }
    #[inline]
    pub fn is_promotion(self) -> bool {
        self.promotion().is_some()
    }

    pub fn from_uci(s: &str) -> Option<Move> {
        if s.len() < 4 || s.len() > 5 {
            return None;
        }
        let from = square_from_name(&s[0..2])?;
        let to = square_from_name(&s[2..4])?;
        let promo = if s.len() == 5 {
            match s.as_bytes()[4] as char {
                'q' | 'Q' => Some(PieceType::Queen),
                'r' | 'R' => Some(PieceType::Rook),
                'b' | 'B' => Some(PieceType::Bishop),
                'n' | 'N' => Some(PieceType::Knight),
                _ => return None,
            }
        } else {
            None
        };
        Some(Move::new(from, to, promo))
    }

    pub fn to_uci(self) -> String {
        if self.is_null() {
            return "0000".to_string();
        }
        let mut s = format!("{}{}", square_name(self.from_sq()), square_name(self.to_sq()));
        if let Some(p) = self.promotion() {
            s.push(p.promo_char());
        }
        s
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Move({})", self.to_uci())
    }
}
impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

// MoveList
#[derive(Clone)]
pub struct MoveList {
    pub moves: [Move; 256],
    pub len: usize,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: [Move::NULL; 256],
            len: 0,
        }
    }
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }
    #[inline]
    pub fn push(&mut self, m: Move) {
        debug_assert!(self.len < 256);
        self.moves[self.len] = m;
        self.len += 1;
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.len].iter()
    }
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

// Bitboard helpers
#[inline]
pub fn bb_square(sq: Square) -> u64 {
    1u64 << sq
}
#[inline]
pub fn pop_lsb(bb: &mut u64) -> Square {
    let sq = bb.trailing_zeros() as Square;
    *bb &= *bb - 1;
    sq
}
#[inline]
pub fn lsb(bb: u64) -> Square {
    bb.trailing_zeros() as Square
}
#[inline]
pub fn count_bits(bb: u64) -> u32 {
    bb.count_ones()
}
