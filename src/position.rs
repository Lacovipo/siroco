use crate::types::*;

use std::sync::OnceLock;

// Zobrist
pub struct Zobrist {
    pub piece_keys: [[u64; 64]; 12],
    pub side_key: u64,
    pub castling_keys: [u64; 16],
    pub en_passant_keys: [u64; 8],
}

fn splitmix64(state: &mut u64) -> u64 {
    let mut z = *state;
    z = z.wrapping_add(0x9E3779B97F4A7C15);
    *state = z;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn init_zobrist() -> Zobrist {
    let mut seed: u64 = 0x123456789ABCDEF0;
    // Use splitmix to generate
    let mut piece_keys = [[0u64; 64]; 12];
    for p in 0..12 {
        for sq in 0..64 {
            piece_keys[p][sq] = splitmix64(&mut seed);
            // avoid zero
            if piece_keys[p][sq] == 0 {
                piece_keys[p][sq] = 1;
            }
        }
    }
    let side_key = splitmix64(&mut seed);
    let mut castling_keys = [0u64; 16];
    for i in 0..16 {
        castling_keys[i] = splitmix64(&mut seed);
    }
    let mut en_passant_keys = [0u64; 8];
    for i in 0..8 {
        en_passant_keys[i] = splitmix64(&mut seed);
    }
    Zobrist {
        piece_keys,
        side_key,
        castling_keys,
        en_passant_keys,
    }
}

static ZOBRIST: OnceLock<Zobrist> = OnceLock::new();

pub fn zobrist() -> &'static Zobrist {
    ZOBRIST.get_or_init(init_zobrist)
}

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub castling: u8,
    pub en_passant: u8,
    pub halfmove: u8,
    pub captured: u8,
    pub hash: u64,
}

pub struct Position {
    pub board: [u8; 64],
    pub bb: [[u64; 6]; 2],
    pub occupied: [u64; 2],
    pub occupied_all: u64,
    pub side_to_move: Color,
    pub castling: u8,
    pub en_passant: u8, // 64 = none
    pub halfmove: u8,
    pub fullmove: u16,
    pub hash: u64,
    pub history: Vec<State>,
    pub hash_history: Vec<u64>,
}

impl Position {
    pub fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("startpos fen")
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut board = [NO_PIECE; 64];
        let mut bb = [[0u64; 6]; 2];
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(format!("FEN too short: {}", fen));
        }
        let piece_part = parts[0];
        let side_part = parts[1];
        let castle_part = parts[2];
        let ep_part = parts[3];
        let halfmove: u8 = if parts.len() > 4 {
            parts[4].parse().map_err(|_| "halfmove")?
        } else {
            0
        };
        let fullmove: u16 = if parts.len() > 5 {
            parts[5].parse().map_err(|_| "fullmove")?
        } else {
            1
        };

        let mut rank: i8 = 7;
        let mut file: i8 = 0;
        for ch in piece_part.chars() {
            if ch == '/' {
                if file != 8 {
                    return Err("FEN rank not 8 files".to_string());
                }
                rank -= 1;
                file = 0;
                if rank < 0 {
                    return Err("too many ranks".to_string());
                }
            } else if ch.is_ascii_digit() {
                let empty = ch.to_digit(10).unwrap() as i8;
                if !(1..=8).contains(&empty) {
                    return Err("bad digit".to_string());
                }
                for _ in 0..empty {
                    let sq = make_square(file as u8, rank as u8);
                    board[sq as usize] = NO_PIECE;
                    file += 1;
                }
            } else {
                let color = if ch.is_ascii_uppercase() {
                    Color::White
                } else {
                    Color::Black
                };
                let pt = PieceType::from_char(ch).ok_or(format!("bad piece {}", ch))?;
                if file >= 8 || rank < 0 {
                    return Err("board overflow".to_string());
                }
                let sq = make_square(file as u8, rank as u8);
                let piece = make_piece(color, pt);
                board[sq as usize] = piece;
                bb[color as usize][pt as usize] |= 1u64 << sq;
                file += 1;
            }
        }
        if rank != 0 || file != 8 {
            // allow? ensure whole board filled
        }

        let side_to_move = match side_part {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("side".to_string()),
        };

        let mut castling: u8 = 0;
        if castle_part != "-" {
            for c in castle_part.chars() {
                match c {
                    'K' => castling |= CASTLE_WK,
                    'Q' => castling |= CASTLE_WQ,
                    'k' => castling |= CASTLE_BK,
                    'q' => castling |= CASTLE_BQ,
                    _ => return Err(format!("bad castling {}", c)),
                }
            }
        }

        let en_passant = if ep_part == "-" {
            NO_SQUARE
        } else {
            square_from_name(ep_part).ok_or("bad ep")?
        };

        // compute occupied
        let mut occupied = [0u64; 2];
        for c in 0..2 {
            let mut occ = 0u64;
            for pt in 0..6 {
                occ |= bb[c][pt];
            }
            occupied[c] = occ;
        }
        let occupied_all = occupied[0] | occupied[1];

        // compute hash
        let z = zobrist();
        let mut hash: u64 = 0;
        for sq in 0..64 {
            let p = board[sq];
            if p != NO_PIECE {
                hash ^= z.piece_keys[p as usize][sq];
            }
        }
        if side_to_move == Color::Black {
            hash ^= z.side_key;
        }
        hash ^= z.castling_keys[castling as usize];
        if en_passant != NO_SQUARE {
            hash ^= z.en_passant_keys[square_file(en_passant) as usize];
        }

        let mut pos = Position {
            board,
            bb,
            occupied,
            occupied_all,
            side_to_move,
            castling,
            en_passant,
            halfmove,
            fullmove,
            hash,
            history: Vec::with_capacity(256),
            hash_history: Vec::with_capacity(256),
        };
        // initial hash_history includes current hash for repetition detection
        pos.hash_history.push(hash);
        // recompute occupied to ensure consistency (already done)
        pos.recompute_occupied();
        debug_assert_eq!(pos.hash, pos.compute_hash());
        Ok(pos)
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let sq = make_square(file, rank);
                let p = self.board[sq as usize];
                if p == NO_PIECE {
                    empty += 1;
                } else {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }
                    let pt = piece_type(p);
                    let mut c = pt.to_char();
                    if piece_color(p) == Color::White {
                        c = c.to_ascii_uppercase();
                    }
                    fen.push(c);
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank != 0 {
                fen.push('/');
            }
        }
        fen.push(' ');
        fen.push(if self.side_to_move == Color::White { 'w' } else { 'b' });
        fen.push(' ');
        if self.castling == 0 {
            fen.push('-');
        } else {
            if self.castling & CASTLE_WK != 0 {
                fen.push('K');
            }
            if self.castling & CASTLE_WQ != 0 {
                fen.push('Q');
            }
            if self.castling & CASTLE_BK != 0 {
                fen.push('k');
            }
            if self.castling & CASTLE_BQ != 0 {
                fen.push('q');
            }
        }
        fen.push(' ');
        if self.en_passant == NO_SQUARE {
            fen.push('-');
        } else {
            fen.push_str(&square_name(self.en_passant));
        }
        fen.push(' ');
        fen.push_str(&self.halfmove.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove.to_string());
        fen
    }

    #[inline]
    pub fn piece_at(&self, sq: Square) -> u8 {
        self.board[sq as usize]
    }

    #[inline]
    pub fn is_empty(&self, sq: Square) -> bool {
        self.board[sq as usize] == NO_PIECE
    }

    fn recompute_occupied(&mut self) {
        for c in 0..2 {
            let mut occ = 0u64;
            for pt in 0..6 {
                occ |= self.bb[c][pt];
            }
            self.occupied[c] = occ;
        }
        self.occupied_all = self.occupied[0] | self.occupied[1];
    }

    pub fn compute_hash(&self) -> u64 {
        let z = zobrist();
        let mut h = 0u64;
        for sq in 0..64 {
            let p = self.board[sq];
            if p != NO_PIECE {
                h ^= z.piece_keys[p as usize][sq];
            }
        }
        if self.side_to_move == Color::Black {
            h ^= z.side_key;
        }
        h ^= z.castling_keys[self.castling as usize];
        if self.en_passant != NO_SQUARE {
            h ^= z.en_passant_keys[square_file(self.en_passant) as usize];
        }
        h
    }

    pub fn king_square(&self, color: Color) -> Square {
        let bb = self.bb[color as usize][PieceType::King as usize];
        debug_assert!(bb != 0);
        lsb(bb)
    }

    // incremental make_move with full logic
    pub fn make_move(&mut self, mv: Move) {
        let from = mv.from_sq();
        let to = mv.to_sq();
        let promo = mv.promotion();
        let us = self.side_to_move;
        let them = us.opposite();
        let us_idx = us as usize;
        let them_idx = them as usize;

        let piece = self.board[from as usize];
        debug_assert!(piece != NO_PIECE);
        debug_assert!(piece_color(piece) == us);
        let pt = piece_type(piece);

        let captured = if self.board[to as usize] != NO_PIECE {
            self.board[to as usize]
        } else if pt == PieceType::Pawn && to == self.en_passant {
            // en passant capture
            let cap_sq = if us == Color::White {
                to - 8
            } else {
                to + 8
            };
            self.board[cap_sq as usize]
        } else {
            NO_PIECE
        };

        let is_capture = captured != NO_PIECE;
        let is_pawn_move = pt == PieceType::Pawn;
        let is_king_move = pt == PieceType::King;

        // save state
        let state = State {
            castling: self.castling,
            en_passant: self.en_passant,
            halfmove: self.halfmove,
            captured,
            hash: self.hash,
        };
        self.history.push(state);

        // Hash updates: remove old ep and castling
        let z = zobrist();
        // side will toggle at end, but handle ep/castling removal now
        if self.en_passant != NO_SQUARE {
            self.hash ^= z.en_passant_keys[square_file(self.en_passant) as usize];
        }
        self.hash ^= z.castling_keys[self.castling as usize];

        // Remove piece from from
        self.hash ^= z.piece_keys[piece as usize][from as usize];
        self.board[from as usize] = NO_PIECE;
        self.bb[us_idx][pt as usize] &= !(1u64 << from);

        // Remove captured
        if is_capture {
            let cap_sq = if pt == PieceType::Pawn && to == state.en_passant && self.board[to as usize]==NO_PIECE {
                // en passant case where captured was not on to
                if us == Color::White { to - 8 } else { to + 8 }
            } else {
                to
            };
            // captured piece already identified
            self.hash ^= z.piece_keys[captured as usize][cap_sq as usize];
            self.board[cap_sq as usize] = NO_PIECE;
            let cap_pt = piece_type(captured);
            let cap_color = piece_color(captured);
            self.bb[cap_color as usize][cap_pt as usize] &= !(1u64 << cap_sq);
            // if we captured en passant pawn, to remains empty before placing mover, already cleared
            // if normal capture, to already cleared by above? board[to] was captured but we need to ensure to cleared before placing mover; in en passant case board[to] was empty, cap_sq separate.
            // For normal capture, board[to] == captured, we already cleared board[cap_sq] which is to, so okay.
        }

        // Handle castling rook movement
        let is_castle = is_king_move && (square_file(from) as i8 - square_file(to) as i8).abs() == 2;
        if is_castle {
            let (rook_from, rook_to) = match (us, to) {
                (Color::White, 6) => (7, 5),  // e1->g1, h1->f1
                (Color::White, 2) => (0, 3),  // e1->c1, a1->d1
                (Color::Black, 62) => (63, 61),
                (Color::Black, 58) => (56, 59),
                _ => panic!("bad castle"),
            };
            let rook = make_piece(us, PieceType::Rook);
            // remove rook from
            self.hash ^= z.piece_keys[rook as usize][rook_from as usize];
            self.board[rook_from as usize] = NO_PIECE;
            self.bb[us_idx][PieceType::Rook as usize] &= !(1u64 << rook_from);
            // add rook to
            self.hash ^= z.piece_keys[rook as usize][rook_to as usize];
            self.board[rook_to as usize] = rook;
            self.bb[us_idx][PieceType::Rook as usize] |= 1u64 << rook_to;
        }

        // Place moving piece at to (with promotion)
        let placed_piece = if let Some(promo_pt) = promo {
            make_piece(us, promo_pt)
        } else {
            piece
        };
        // if is_capture and not en passant, board[to] already cleared; if en passant, board[to] was empty
        // For castle, to was empty (king dest) but rook moved separately.
        self.hash ^= z.piece_keys[placed_piece as usize][to as usize];
        self.board[to as usize] = placed_piece;
        let placed_pt = piece_type(placed_piece);
        self.bb[us_idx][placed_pt as usize] |= 1u64 << to;
        // if promotion, we removed pawn already from from and added new piece at to; no extra handling.
        // But we need to ensure pawn bitboard not have extra: we already removed pawn from from, and added queen etc. For promotion pawn piece type is pawn, but placed_pt is queen, so bb pawn reduced, queen increased.
        // However for non-promo pawn moves, placed_pt == Pawn, so pawn bit moves.

        // Update castling rights
        let mut new_castling = self.castling;
        if is_king_move {
            if us == Color::White {
                new_castling &= !(CASTLE_WK | CASTLE_WQ);
            } else {
                new_castling &= !(CASTLE_BK | CASTLE_BQ);
            }
        }
        // rook moving from original squares
        match from {
            0 => new_castling &= !CASTLE_WQ,
            7 => new_castling &= !CASTLE_WK,
            56 => new_castling &= !CASTLE_BQ,
            63 => new_castling &= !CASTLE_BK,
            _ => {}
        }
        // rook captured on original squares
        // Note captured square for en passant is cap_sq, for normal is to
        let cap_sq_for_castle = if is_capture && pt == PieceType::Pawn && to == state.en_passant {
            if us == Color::White { to - 8 } else { to + 8 }
        } else {
            to
        };
        if is_capture {
            match cap_sq_for_castle {
                0 => new_castling &= !CASTLE_WQ,
                7 => new_castling &= !CASTLE_WK,
                56 => new_castling &= !CASTLE_BQ,
                63 => new_castling &= !CASTLE_BK,
                _ => {}
            }
        }
        self.castling = new_castling;
        self.hash ^= z.castling_keys[self.castling as usize];

        // Update en passant
        let mut new_ep = NO_SQUARE;
        if is_pawn_move && (to as i16 - from as i16).abs() == 16 {
            // double push
            new_ep = if us == Color::White {
                from + 8
            } else {
                from - 8
            };
        }
        self.en_passant = new_ep;
        if self.en_passant != NO_SQUARE {
            self.hash ^= z.en_passant_keys[square_file(self.en_passant) as usize];
        }

        // Update halfmove
        if is_pawn_move || is_capture {
            self.halfmove = 0;
        } else {
            self.halfmove = self.halfmove.wrapping_add(1);
        }

        // Update fullmove
        if us == Color::Black {
            self.fullmove += 1;
        }

        // Toggle side
        self.side_to_move = them;
        self.hash ^= z.side_key;

        // Recompute occupied
        self.recompute_occupied();

        // push new hash for repetition tracking
        self.hash_history.push(self.hash);

        debug_assert_eq!(self.hash, self.compute_hash(), "hash mismatch after make {}: fen {}", mv.to_uci(), self.to_fen());
        // verify occupancy consistency
        debug_assert!(self.occupied_all == (self.occupied[0] | self.occupied[1]));
    }

    pub fn unmake_move(&mut self, mv: Move) {
        let state = self.history.pop().expect("no history");
        let from = mv.from_sq();
        let to = mv.to_sq();
        let promo = mv.promotion();

        // side that moved is opposite of current
        let them = self.side_to_move;
        let us = them.opposite();
        let us_idx = us as usize;
        let them_idx = them as usize;

        // current board has placed piece at to
        let placed_piece = self.board[to as usize];
        debug_assert!(placed_piece != NO_PIECE);

        // handle castling rook reverse
        let is_king_move = piece_type(placed_piece) == PieceType::King || (promo.is_none() && {
            // if promotion, piece at to is queen etc not king, so not castle
            false
        });
        // Better determine castle by checking if we moved king 2 squares: from/to diff file 2 and piece originally was king
        // Since after promotion placed_piece is not king, castle cannot be promotion. So we can detect castle by:
        // if original moving piece was king and abs file diff 2
        // To know original piece, check if placed_piece is king or if not promotion but still king
        // Actually we can deduce: if promo is Some, not castle.
        let was_castle = promo.is_none() && piece_type(placed_piece) == PieceType::King
            && (square_file(from) as i8 - square_file(to) as i8).abs() == 2;

        if was_castle {
            let (rook_from, rook_to) = match (us, to) {
                (Color::White, 6) => (7, 5),
                (Color::White, 2) => (0, 3),
                (Color::Black, 62) => (63, 61),
                (Color::Black, 58) => (56, 59),
                _ => panic!("bad castle unmake"),
            };
            let rook = make_piece(us, PieceType::Rook);
            // remove rook from to
            self.board[rook_to as usize] = NO_PIECE;
            self.bb[us_idx][PieceType::Rook as usize] &= !(1u64 << rook_to);
            // place rook back
            self.board[rook_from as usize] = rook;
            self.bb[us_idx][PieceType::Rook as usize] |= 1u64 << rook_from;
        }

        // remove placed piece from to
        let placed_pt = piece_type(placed_piece);
        self.bb[us_idx][placed_pt as usize] &= !(1u64 << to);
        self.board[to as usize] = NO_PIECE;

        // restore moving piece at from
        let original_piece = if promo.is_some() {
            make_piece(us, PieceType::Pawn)
        } else {
            placed_piece
        };
        let orig_pt = piece_type(original_piece);
        self.board[from as usize] = original_piece;
        self.bb[us_idx][orig_pt as usize] |= 1u64 << from;

        // restore captured
        if state.captured != NO_PIECE {
            let cap_sq = if promo.is_some() {
                // promotion capture case: captured was on to
                to
            } else if orig_pt == PieceType::Pawn && to == state.en_passant {
                // en passant capture: captured pawn not on to
                if us == Color::White { to - 8 } else { to + 8 }
            } else {
                to
            };
            // en passant: to was empty, captured pawn at cap_sq
            // normal capture: to will be overwritten with captured after removing mover, but we already cleared to, so we can place captured there
            self.board[cap_sq as usize] = state.captured;
            let cap_pt = piece_type(state.captured);
            let cap_color = piece_color(state.captured);
            self.bb[cap_color as usize][cap_pt as usize] |= 1u64 << cap_sq;
            // if capture was en passant, to remains empty (already), else to now holds captured (but we already cleared to after removing mover, so now we place captured at to which is correct for normal capture)
            // But for normal capture we set board[to]=captured? Wait we just set board[cap_sq]=captured where cap_sq == to, so board[to] becomes captured, not empty. However we previously set board[to]=NO_PIECE after removing mover. Now we overwrote it with captured. That's correct for normal capture? No, after unmake, board[to] should be empty (since piece moved back to from), and if there was a capture, the captured piece should be restored at to. Actually after undoing a normal capture, the destination square should contain the captured piece, not be empty. But we are unmaking to restore position before move, which had piece at from and captured piece at to. After unmake, we restored piece at from, and we need to restore captured piece at to. So yes board[to] should become captured.
            // For en passant, board[to] should remain empty (pawn moved to ep square, captured pawn at cap_sq behind), so after unmake, board[to] stays empty and cap_sq gets pawn.
            // Our code above does: for en passant, cap_sq != to, so we set board[cap_sq]=captured and board[to] remains NO_PIECE (as we left). Correct.
            // For normal capture, cap_sq == to, so we set board[to]=captured, correct.
        } else {
            // no capture, board[to] remains empty (already cleared)
        }

        // restore other state
        self.castling = state.castling;
        self.en_passant = state.en_passant;
        self.halfmove = state.halfmove;
        if us == Color::Black {
            // we incremented fullmove when us was black, so decrement
            self.fullmove -= 1;
        }
        self.side_to_move = us;
        self.hash = state.hash;

        // hash_history: pop current hash, as we revert
        self.hash_history.pop();
        // recompute occupied
        self.recompute_occupied();

        debug_assert_eq!(self.hash, self.compute_hash(), "hash mismatch after unmake {}", mv.to_uci());
    }

    // draw detection
    pub fn is_draw(&self) -> bool {
        if self.halfmove >= 100 {
            return true;
        }
        if self.is_insufficient_material() {
            return true;
        }
        if self.is_threefold() {
            return true;
        }
        false
    }

    pub fn is_insufficient_material(&self) -> bool {
        // K vs K, K vs KB, K vs KN, KB vs KB same color bishops?
        // Simplified: if no pawns, rooks, queens and limited minors
        let pawns = self.bb[0][PieceType::Pawn as usize] | self.bb[1][PieceType::Pawn as usize];
        if pawns != 0 {
            return false;
        }
        let rooks = self.bb[0][PieceType::Rook as usize] | self.bb[1][PieceType::Rook as usize];
        let queens = self.bb[0][PieceType::Queen as usize] | self.bb[1][PieceType::Queen as usize];
        if rooks != 0 || queens != 0 {
            return false;
        }
        let bishops_w = self.bb[0][PieceType::Bishop as usize];
        let bishops_b = self.bb[1][PieceType::Bishop as usize];
        let knights_w = self.bb[0][PieceType::Knight as usize];
        let knights_b = self.bb[1][PieceType::Knight as usize];
        let bishops = bishops_w | bishops_b;
        let knights = knights_w | knights_b;
        let num_bishops = bishops.count_ones();
        let num_knights = knights.count_ones();

        // K vs K
        if num_bishops == 0 && num_knights == 0 {
            return true;
        }
        // K + single minor vs K
        if num_bishops + num_knights == 1 {
            return true;
        }
        // KB vs KB with bishops same color? Need to check square color
        if num_bishops == 2 && num_knights == 0 {
            // if both bishops on same color, insufficient
            // Check if bishops are on same color squares
            // Find squares
            let mut bb = bishops;
            let mut colors = 0;
            let mut count = 0;
            while bb != 0 {
                let sq = pop_lsb(&mut bb);
                let col = (square_file(sq) + square_rank(sq)) % 2;
                if count == 0 {
                    colors = col;
                } else if col != colors {
                    return false; // opposite color, not insufficient
                }
                count += 1;
            }
            // same color
            return true;
        }
        false
    }

    pub fn is_threefold(&self) -> bool {
        let mut count = 0;
        for &h in &self.hash_history {
            if h == self.hash {
                count += 1;
                if count >= 3 {
                    return true;
                }
            }
        }
        // also count current?
        false
    }

    pub fn is_repetition(&self) -> bool {
        // for search: check if current hash appears at least once in history with same side to move? hash includes side, so just count
        let mut reps = 0;
        for &h in self.hash_history.iter().rev().skip(1) {
            if h == self.hash {
                reps += 1;
                if reps >= 1 {
                    // For search we consider draw if at least 1 repetition and we can claim? Usually 2-fold is draw-ish but UCI should not claim? We'll treat 2nd occurrence as draw for search.
                    // But to avoid false draws early, require 2 occurrences total (current + one). So if reps >=1 means 2-fold.
                    return true;
                }
            }
            // stop early if halfmove reset? Not needed.
        }
        false
    }
}
