use crate::position::Position;
use crate::types::*;

use std::sync::OnceLock;

// Precomputed attacks
struct Attacks {
    knight: [u64; 64],
    king: [u64; 64],
    pawn: [[u64; 64]; 2],
}

fn init_attacks() -> Attacks {
    let mut knight = [0u64; 64];
    let mut king = [0u64; 64];
    let mut pawn = [[0u64; 64]; 2];

    for sq in 0..64u8 {
        let r = square_rank(sq) as i8;
        let f = square_file(sq) as i8;

        // knight
        let mut bb = 0u64;
        for (dr, df) in [(2, 1), (1, 2), (-1, 2), (-2, 1), (-2, -1), (-1, -2), (1, -2), (2, -1)] {
            let nr = r + dr;
            let nf = f + df;
            if nr >= 0 && nr < 8 && nf >= 0 && nf < 8 {
                bb |= 1u64 << make_square(nf as u8, nr as u8);
            }
        }
        knight[sq as usize] = bb;

        // king
        bb = 0;
        for dr in -1..=1 {
            for df in -1..=1 {
                if dr == 0 && df == 0 {
                    continue;
                }
                let nr = r + dr;
                let nf = f + df;
                if nr >= 0 && nr < 8 && nf >= 0 && nf < 8 {
                    bb |= 1u64 << make_square(nf as u8, nr as u8);
                }
            }
        }
        king[sq as usize] = bb;

        // pawn attacks
        // White pawns attack north
        let mut w = 0u64;
        let mut b = 0u64;
        if r < 7 {
            if f > 0 {
                w |= 1u64 << make_square((f - 1) as u8, (r + 1) as u8);
            }
            if f < 7 {
                w |= 1u64 << make_square((f + 1) as u8, (r + 1) as u8);
            }
        }
        if r > 0 {
            if f > 0 {
                b |= 1u64 << make_square((f - 1) as u8, (r - 1) as u8);
            }
            if f < 7 {
                b |= 1u64 << make_square((f + 1) as u8, (r - 1) as u8);
            }
        }
        pawn[Color::White as usize][sq as usize] = w;
        pawn[Color::Black as usize][sq as usize] = b;
    }

    Attacks { knight, king, pawn }
}

static ATTACKS: OnceLock<Attacks> = OnceLock::new();

fn attacks() -> &'static Attacks {
    ATTACKS.get_or_init(init_attacks)
}

#[inline]
pub fn knight_attacks(sq: Square) -> u64 {
    attacks().knight[sq as usize]
}
#[inline]
pub fn king_attacks(sq: Square) -> u64 {
    attacks().king[sq as usize]
}
#[inline]
pub fn pawn_attacks(color: Color, sq: Square) -> u64 {
    attacks().pawn[color as usize][sq as usize]
}

// Slider attacks on the fly
#[inline]
pub fn bishop_attacks(sq: Square, occ: u64) -> u64 {
    let mut attacks = 0u64;
    let r = square_rank(sq) as i8;
    let f = square_file(sq) as i8;
    for (dr, df) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut nr = r + dr;
        let mut nf = f + df;
        while nr >= 0 && nr < 8 && nf >= 0 && nf < 8 {
            let nsq = make_square(nf as u8, nr as u8);
            attacks |= 1u64 << nsq;
            if occ & (1u64 << nsq) != 0 {
                break;
            }
            nr += dr;
            nf += df;
        }
    }
    attacks
}

#[inline]
pub fn rook_attacks(sq: Square, occ: u64) -> u64 {
    let mut attacks = 0u64;
    let r = square_rank(sq) as i8;
    let f = square_file(sq) as i8;
    for (dr, df) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let mut nr = r + dr;
        let mut nf = f + df;
        while nr >= 0 && nr < 8 && nf >= 0 && nf < 8 {
            let nsq = make_square(nf as u8, nr as u8);
            attacks |= 1u64 << nsq;
            if occ & (1u64 << nsq) != 0 {
                break;
            }
            nr += dr;
            nf += df;
        }
    }
    attacks
}

#[inline]
pub fn queen_attacks(sq: Square, occ: u64) -> u64 {
    bishop_attacks(sq, occ) | rook_attacks(sq, occ)
}

pub fn is_square_attacked(pos: &Position, sq: Square, attacker: Color) -> bool {
    let occ = pos.occupied_all;
    let att_idx = attacker as usize;
    // pawn
    let pawns = pos.bb[att_idx][PieceType::Pawn as usize];
    // For pawn attack, we need to check if sq is attacked by pawn from attacker perspective.
    // Pawn attacks are reverse: if attacker is white, pawns attack north, so to see if sq is attacked, we look at pawn positions that can attack sq.
    // Our pawn_attacks[color][sq] gives squares attacked by pawn on sq. So to see if sq is attacked, we can check pawns that attack sq: pawns & pawn_attacks_opposite.
    // Simpler: compute pawn attackers: for white attacker, pawns that could attack sq are on rank-1.
    // We can just test: pawns & (pawn_attacks of opposite color from sq) ??? Trick: pawn_attacks[attacker][sq] doesn't give attacker positions; need reverse.
    // Actually pawn_attacks[White][sq] gives squares a white pawn on sq attacks. So to find if sq is attacked by white, we need to find white pawns on squares that attack sq: that's equivalent to black pawn attacks from sq intersecting white pawns.
    // So use: pawn_attacks(attacker.opposite(), sq) & pawns !=0
    let pawn_attackers = pawn_attacks(attacker.opposite(), sq);
    if pawns & pawn_attackers != 0 {
        return true;
    }

    if knight_attacks(sq) & pos.bb[att_idx][PieceType::Knight as usize] != 0 {
        return true;
    }
    if king_attacks(sq) & pos.bb[att_idx][PieceType::King as usize] != 0 {
        return true;
    }

    let bishops_queens = pos.bb[att_idx][PieceType::Bishop as usize] | pos.bb[att_idx][PieceType::Queen as usize];
    if bishop_attacks(sq, occ) & bishops_queens != 0 {
        return true;
    }
    let rooks_queens = pos.bb[att_idx][PieceType::Rook as usize] | pos.bb[att_idx][PieceType::Queen as usize];
    if rook_attacks(sq, occ) & rooks_queens != 0 {
        return true;
    }
    false
}

#[inline]
fn add_moves(list: &mut MoveList, from: Square, to_bb: u64) {
    let mut bb = to_bb;
    while bb != 0 {
        let to = pop_lsb(&mut bb);
        list.push(Move::new(from, to, None));
    }
}

// Generate pseudo-legal moves (may leave king in check)
pub fn generate_pseudo_legal(pos: &Position, list: &mut MoveList) {
    list.clear();
    let us = pos.side_to_move;
    let them = us.opposite();
    let us_idx = us as usize;
    let them_idx = them as usize;
    let occ = pos.occupied_all;
    let own_occ = pos.occupied[us_idx];
    let opp_occ = pos.occupied[them_idx];
    let empty = !occ;

    // Pawns
    let pawns = pos.bb[us_idx][PieceType::Pawn as usize];
    let mut pawns_bb = pawns;
    while pawns_bb != 0 {
        let from = pop_lsb(&mut pawns_bb);
        let rank = square_rank(from);
        let file = square_file(from);

        if us == Color::White {
            // single push
            if rank < 7 {
                let to = from + 8;
                if pos.board[to as usize] == NO_PIECE {
                    if rank == 6 {
                        // promotion
                        for promo in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                            list.push(Move::new(from, to, Some(promo)));
                        }
                    } else {
                        list.push(Move::new(from, to, None));
                        // double push from rank 1
                        if rank == 1 {
                            let to2 = from + 16;
                            if pos.board[to2 as usize] == NO_PIECE {
                                list.push(Move::new(from, to2, None));
                            }
                        }
                    }
                }
                // captures
                if file > 0 {
                    let to = from + 7;
                    if rank < 7 {
                        if opp_occ & (1u64 << to) != 0 {
                            if rank == 6 {
                                for promo in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                                    list.push(Move::new(from, to, Some(promo)));
                                }
                            } else {
                                list.push(Move::new(from, to, None));
                            }
                        } else if to == pos.en_passant {
                            list.push(Move::new(from, to, None));
                        }
                    }
                }
                if file < 7 {
                    let to = from + 9;
                    if rank < 7 {
                        if opp_occ & (1u64 << to) != 0 {
                            if rank == 6 {
                                for promo in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                                    list.push(Move::new(from, to, Some(promo)));
                                }
                            } else {
                                list.push(Move::new(from, to, None));
                            }
                        } else if to == pos.en_passant {
                            list.push(Move::new(from, to, None));
                        }
                    }
                }
            }
        } else {
            // Black
            if rank > 0 {
                let to = from - 8;
                if pos.board[to as usize] == NO_PIECE {
                    if rank == 1 {
                        for promo in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                            list.push(Move::new(from, to, Some(promo)));
                        }
                    } else {
                        list.push(Move::new(from, to, None));
                        if rank == 6 {
                            let to2 = from - 16;
                            if pos.board[to2 as usize] == NO_PIECE {
                                list.push(Move::new(from, to2, None));
                            }
                        }
                    }
                }
                if file > 0 {
                    let to = from - 9;
                    if opp_occ & (1u64 << to) != 0 {
                        if rank == 1 {
                            for promo in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                                list.push(Move::new(from, to, Some(promo)));
                            }
                        } else {
                            list.push(Move::new(from, to, None));
                        }
                    } else if to == pos.en_passant {
                        list.push(Move::new(from, to, None));
                    }
                }
                if file < 7 {
                    let to = from - 7;
                    if opp_occ & (1u64 << to) != 0 {
                        if rank == 1 {
                            for promo in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                                list.push(Move::new(from, to, Some(promo)));
                            }
                        } else {
                            list.push(Move::new(from, to, None));
                        }
                    } else if to == pos.en_passant {
                        list.push(Move::new(from, to, None));
                    }
                }
            }
        }
    }

    // Knights
    let knights = pos.bb[us_idx][PieceType::Knight as usize];
    let mut bb = knights;
    while bb != 0 {
        let from = pop_lsb(&mut bb);
        let attacks = knight_attacks(from) & !own_occ;
        add_moves(list, from, attacks);
    }

    // Bishops
    let bishops = pos.bb[us_idx][PieceType::Bishop as usize];
    let mut bb = bishops;
    while bb != 0 {
        let from = pop_lsb(&mut bb);
        let attacks = bishop_attacks(from, occ) & !own_occ;
        add_moves(list, from, attacks);
    }

    // Rooks
    let rooks = pos.bb[us_idx][PieceType::Rook as usize];
    let mut bb = rooks;
    while bb != 0 {
        let from = pop_lsb(&mut bb);
        let attacks = rook_attacks(from, occ) & !own_occ;
        add_moves(list, from, attacks);
    }

    // Queens
    let queens = pos.bb[us_idx][PieceType::Queen as usize];
    let mut bb = queens;
    while bb != 0 {
        let from = pop_lsb(&mut bb);
        let attacks = queen_attacks(from, occ) & !own_occ;
        add_moves(list, from, attacks);
    }

    // King
    let king_sq = pos.king_square(us);
    let king_att = king_attacks(king_sq) & !own_occ;
    add_moves(list, king_sq, king_att);

    // Castling (pseudo-legal, need to check not in check and squares not attacked)
    if !is_square_attacked(pos, king_sq, them) {
        if us == Color::White {
            if pos.castling & CASTLE_WK != 0 {
                // f1,g1 empty, and f1,g1 not attacked
                if pos.board[5] == NO_PIECE && pos.board[6] == NO_PIECE {
                    if !is_square_attacked(pos, 5, them) && !is_square_attacked(pos, 6, them) {
                        list.push(Move::new(king_sq, 6, None));
                    }
                }
            }
            if pos.castling & CASTLE_WQ != 0 {
                if pos.board[1] == NO_PIECE && pos.board[2] == NO_PIECE && pos.board[3] == NO_PIECE {
                    if !is_square_attacked(pos, 3, them) && !is_square_attacked(pos, 2, them) {
                        list.push(Move::new(king_sq, 2, None));
                    }
                }
            }
        } else {
            if pos.castling & CASTLE_BK != 0 {
                if pos.board[61] == NO_PIECE && pos.board[62] == NO_PIECE {
                    if !is_square_attacked(pos, 61, them) && !is_square_attacked(pos, 62, them) {
                        list.push(Move::new(king_sq, 62, None));
                    }
                }
            }
            if pos.castling & CASTLE_BQ != 0 {
                if pos.board[57] == NO_PIECE && pos.board[58] == NO_PIECE && pos.board[59] == NO_PIECE {
                    if !is_square_attacked(pos, 59, them) && !is_square_attacked(pos, 58, them) {
                        list.push(Move::new(king_sq, 58, None));
                    }
                }
            }
        }
    }
}

pub fn generate_legal(pos: &mut Position, list: &mut MoveList) {
    let mut pseudo = MoveList::new();
    generate_pseudo_legal(pos, &mut pseudo);
    list.clear();
    let us = pos.side_to_move;
    for &mv in pseudo.as_slice() {
        pos.make_move(mv);
        // after make, side flipped, so king of us is now opposite of current side
        let king_sq = pos.king_square(us);
        let in_check = is_square_attacked(pos, king_sq, pos.side_to_move);
        pos.unmake_move(mv);
        if !in_check {
            list.push(mv);
        }
    }
}

pub fn generate_captures(pos: &Position, list: &mut MoveList) {
    // For quiescence: only captures and queen promotions (or all promos?)
    // We'll generate pseudo captures then filter en passant as capture
    let mut pseudo = MoveList::new();
    generate_pseudo_legal(pos, &mut pseudo);
    list.clear();
    let opp_occ = pos.occupied[pos.side_to_move.opposite() as usize];
    for &mv in pseudo.as_slice() {
        let to = mv.to_sq();
        let from = mv.from_sq();
        let is_capture = pos.board[to as usize] != NO_PIECE || to == pos.en_passant;
        let is_promo = mv.is_promotion();
        // In quiescence we want captures and promotions (especially queen)
        if is_capture || is_promo {
            // For promotions, include only queen promotion? We'll include all but ordering favors queen
            list.push(mv);
        }
        // also include en passant
    }
    // Alternative direct generation would be more efficient but this is okay for V1.
}

pub fn is_legal_move(pos: &mut Position, mv: Move) -> bool {
    let mut list = MoveList::new();
    generate_legal(pos, &mut list);
    for &m in list.as_slice() {
        if m.0 == mv.0 {
            return true;
        }
    }
    false
}
