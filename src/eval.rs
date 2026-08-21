use crate::movegen::{bishop_attacks, knight_attacks, queen_attacks, rook_attacks};
use crate::position::Position;
use crate::types::*;

// Material values tapered
const MG_VALUES: [i32; 6] = [82, 337, 365, 477, 1025, 0];
const EG_VALUES: [i32; 6] = [94, 281, 297, 512, 936, 0];

const PHASE_WEIGHTS: [i32; 6] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

// PST MG - index 0 = a1 (white bottom)
// rank 0 = rank1, rank 7 = rank8
// Each array is 64 entries rank*8+file

// Pawn MG
const PST_PAWN_MG: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0, // rank1
     0,  0,  0,  0,  0,  0,  0,  0, // rank2
     5,  5,  5, 10, 10,  5,  5,  5, // rank3
    10, 10, 15, 20, 20, 15, 10, 10, // rank4
    15, 15, 20, 30, 30, 20, 15, 15, // rank5
    25, 25, 30, 35, 35, 30, 25, 25, // rank6
    40, 40, 40, 50, 50, 40, 40, 40, // rank7
     0,  0,  0,  0,  0,  0,  0,  0, // rank8
];
const PST_PAWN_EG: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    10, 10, 10, 10, 10, 10, 10, 10,
    20, 20, 25, 30, 30, 25, 20, 20,
    30, 30, 35, 40, 40, 35, 30, 30,
    40, 40, 45, 50, 50, 45, 40, 40,
    50, 50, 55, 60, 60, 55, 50, 50,
    60, 60, 60, 70, 70, 60, 60, 60,
     0,  0,  0,  0,  0,  0,  0,  0,
];

// Knight MG
const PST_KNIGHT_MG: [i32; 64] = [
   -50, -30, -20, -20, -20, -20, -30, -50,
   -30, -10,   0,   5,   5,   0, -10, -30,
   -20,   0,  15,  20,  20,  15,   0, -20,
   -20,   5,  20,  30,  30,  20,   5, -20,
   -20,   5,  20,  30,  30,  20,   5, -20,
   -20,   0,  15,  20,  20,  15,   0, -20,
   -30, -10,   0,   5,   5,   0, -10, -30,
   -50, -30, -20, -20, -20, -20, -30, -50,
];
const PST_KNIGHT_EG: [i32; 64] = [
   -40, -20, -10, -10, -10, -10, -20, -40,
   -20,   0,   5,  10,  10,   5,   0, -20,
   -10,   5,  15,  20,  20,  15,   5, -10,
   -10,  10,  20,  25,  25,  20,  10, -10,
   -10,  10,  20,  25,  25,  20,  10, -10,
   -10,   5,  15,  20,  20,  15,   5, -10,
   -20,   0,   5,  10,  10,   5,   0, -20,
   -40, -20, -10, -10, -10, -10, -20, -40,
];

// Bishop MG
const PST_BISHOP_MG: [i32; 64] = [
   -20, -10, -10, -10, -10, -10, -10, -20,
   -10,   0,   5,   5,   5,   5,   0, -10,
   -10,   5,  10,  10,  10,  10,   5, -10,
   -10,   5,  10,  15,  15,  10,   5, -10,
   -10,   5,  10,  15,  15,  10,   5, -10,
   -10,   5,  10,  10,  10,  10,   5, -10,
   -10,   0,   5,   5,   5,   5,   0, -10,
   -20, -10, -10, -10, -10, -10, -10, -20,
];
const PST_BISHOP_EG: [i32; 64] = [
   -15, -10, -10, -10, -10, -10, -10, -15,
   -10,   5,   5,   5,   5,   5,   5, -10,
   -10,   5,  10,  10,  10,  10,   5, -10,
   -10,   5,  10,  15,  15,  10,   5, -10,
   -10,   5,  10,  15,  15,  10,   5, -10,
   -10,   5,  10,  10,  10,  10,   5, -10,
   -10,   5,   5,   5,   5,   5,   5, -10,
   -15, -10, -10, -10, -10, -10, -10, -15,
];

// Rook MG
const PST_ROOK_MG: [i32; 64] = [
     0,   0,   5,  10,  10,   5,   0,   0,
     0,   5,   5,   5,   5,   5,   5,   0,
     0,   0,   5,   5,   5,   5,   0,   0,
     5,   5,   5,   5,   5,   5,   5,   5,
     5,   5,   5,   5,   5,   5,   5,   5,
     0,   0,   5,   5,   5,   5,   0,   0,
     0,   5,   5,   5,   5,   5,   5,   0,
     0,   0,   5,  10,  10,   5,   0,   0,
];
const PST_ROOK_EG: [i32; 64] = [
     5,   5,   5,   5,   5,   5,   5,   5,
    10,  10,  10,  10,  10,  10,  10,  10,
    10,  10,  10,  10,  10,  10,  10,  10,
    10,  10,  10,  10,  10,  10,  10,  10,
    10,  10,  10,  10,  10,  10,  10,  10,
    10,  10,  10,  10,  10,  10,  10,  10,
    15,  15,  15,  15,  15,  15,  15,  15,
     5,   5,   5,   5,   5,   5,   5,   5,
];

// Queen MG
const PST_QUEEN_MG: [i32; 64] = [
   -20, -10, -10,  -5,  -5, -10, -10, -20,
   -10,   0,   5,   0,   0,   5,   0, -10,
   -10,   5,   5,   5,   5,   5,   5, -10,
    -5,   0,   5,   5,   5,   5,   0,  -5,
    -5,   0,   5,   5,   5,   5,   0,  -5,
   -10,   5,   5,   5,   5,   5,   5, -10,
   -10,   0,   5,   0,   0,   5,   0, -10,
   -20, -10, -10,  -5,  -5, -10, -10, -20,
];
const PST_QUEEN_EG: [i32; 64] = [
   -15, -10, -10,  -5,  -5, -10, -10, -15,
    -5,   0,   5,   5,   5,   5,   0,  -5,
    -5,   5,   5,   5,   5,   5,   5,  -5,
     0,   5,   5,  10,  10,   5,   5,   0,
     0,   5,   5,  10,  10,   5,   5,   0,
    -5,   5,   5,   5,   5,   5,   5,  -5,
    -5,   0,   5,   5,   5,   5,   0,  -5,
   -15, -10, -10,  -5,  -5, -10, -10, -15,
];

// King MG (middlegame - stay castled)
const PST_KING_MG: [i32; 64] = [
   -30, -30, -30, -30, -30, -30, -30, -30,
   -30, -30, -30, -30, -30, -30, -30, -30,
   -20, -20, -20, -20, -20, -20, -20, -20,
   -10, -10, -10, -10, -10, -10, -10, -10,
    -5,  -5,  -5,  -5,  -5,  -5,  -5,  -5,
     0,   0,   0,   0,   0,   0,   0,   0,
    20,  20,  10,   0,   0,  10,  20,  20,
    30,  30,   0, -10, -10,   0,  30,  30,
];
// King EG (centralize)
const PST_KING_EG: [i32; 64] = [
   -50, -30, -20, -20, -20, -20, -30, -50,
   -30, -10,   0,   5,   5,   0, -10, -30,
   -20,   0,  20,  30,  30,  20,   0, -20,
   -20,   5,  30,  40,  40,  30,   5, -20,
   -20,   5,  30,  40,  40,  30,   5, -20,
   -20,   0,  20,  30,  30,  20,   0, -20,
   -30, -10,   0,   5,   5,   0, -10, -30,
   -50, -30, -20, -20, -20, -20, -30, -50,
];

const PST_MG: [&[i32; 64]; 6] = [
    &PST_PAWN_MG,
    &PST_KNIGHT_MG,
    &PST_BISHOP_MG,
    &PST_ROOK_MG,
    &PST_QUEEN_MG,
    &PST_KING_MG,
];
const PST_EG: [&[i32; 64]; 6] = [
    &PST_PAWN_EG,
    &PST_KNIGHT_EG,
    &PST_BISHOP_EG,
    &PST_ROOK_EG,
    &PST_QUEEN_EG,
    &PST_KING_EG,
];

// Passed pawn bonuses by rank (0 = rank1)
const PASS_MG: [i32; 8] = [0, 5, 10, 20, 35, 60, 100, 0];
const PASS_EG: [i32; 8] = [0, 10, 20, 40, 70, 120, 180, 0];

pub fn evaluate(pos: &Position) -> i32 {
    let mut mg: i32 = 0;
    let mut eg: i32 = 0;
    let mut phase: i32 = 0;

    for sq in 0..64 {
        let piece = pos.board[sq];
        if piece == NO_PIECE {
            continue;
        }
        let color = piece_color(piece);
        let pt = piece_type(piece) as usize;
        let sign = if color == Color::White { 1 } else { -1 };
        let pst_sq = if color == Color::White { sq } else { sq ^ 56 };
        mg += sign * (MG_VALUES[pt] + PST_MG[pt][pst_sq]);
        eg += sign * (EG_VALUES[pt] + PST_EG[pt][pst_sq]);
        phase += PHASE_WEIGHTS[pt];
    }

    // Additional positional terms (tapered where relevant)
    let (pmg, peg) = eval_pawn_structure(pos);
    mg += pmg;
    eg += peg;

    let (bmg, beg) = eval_bishop_pair(pos);
    mg += bmg;
    eg += beg;

    let (mmg, meg) = eval_mobility(pos);
    mg += mmg;
    eg += meg;

    let (rmg, reg) = eval_rooks(pos);
    mg += rmg;
    eg += reg;

    let (kmg, keg) = eval_king_safety(pos);
    mg += kmg;
    eg += keg;

    if phase > MAX_PHASE {
        phase = MAX_PHASE;
    }
    let score = (mg * phase + eg * (MAX_PHASE - phase)) / MAX_PHASE;
    let tempo = 10;
    let side_sign = if pos.side_to_move == Color::White { 1 } else { -1 };
    side_sign * score + tempo
}

fn eval_pawn_structure(pos: &Position) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = 0;
    // File counts for isolated/doubled
    let mut white_file = [0; 8];
    let mut black_file = [0; 8];
    let mut white_pawns: Vec<u8> = Vec::new();
    let mut black_pawns: Vec<u8> = Vec::new();
    for sq in 0..64 {
        let p = pos.board[sq];
        if p == NO_PIECE || piece_type(p) != PieceType::Pawn {
            continue;
        }
        let f = (sq % 8) as usize;
        if piece_color(p) == Color::White {
            white_file[f] += 1;
            white_pawns.push(sq as u8);
        } else {
            black_file[f] += 1;
            black_pawns.push(sq as u8);
        }
    }
    // Isolated and doubled
    for &sq in &white_pawns {
        let f = (sq % 8) as usize;
        let isolated = (f == 0 || white_file[f - 1] == 0) && (f == 7 || white_file[f + 1] == 0);
        if isolated {
            mg -= 12;
            eg -= 18;
        }
        if white_file[f] > 1 {
            mg -= 8;
            eg -= 12;
        }
        // passed?
        if is_passed(sq, Color::White, pos) {
            let rank = sq / 8;
            mg += PASS_MG[rank as usize];
            eg += PASS_EG[rank as usize];
        }
    }
    for &sq in &black_pawns {
        let f = (sq % 8) as usize;
        let isolated = (f == 0 || black_file[f - 1] == 0) && (f == 7 || black_file[f + 1] == 0);
        if isolated {
            mg += 12;
            eg += 18;
        }
        if black_file[f] > 1 {
            mg += 8;
            eg += 12;
        }
        if is_passed(sq, Color::Black, pos) {
            let rank = 7 - (sq / 8) as usize;
            mg -= PASS_MG[rank];
            eg -= PASS_EG[rank];
        }
    }
    (mg, eg)
}

fn is_passed(sq: u8, color: Color, pos: &Position) -> bool {
    let file = (sq % 8) as i8;
    let rank = (sq / 8) as i8;
    if (color == Color::White && rank >= 7) || (color == Color::Black && rank <= 0) {
        return false;
    }
    for opp_sq in 0..64 {
        let p = pos.board[opp_sq];
        if p == NO_PIECE || piece_type(p) != PieceType::Pawn || piece_color(p) == color {
            continue;
        }
        let of = (opp_sq % 8) as i8;
        let or = (opp_sq / 8) as i8;
        if (of - file).abs() <= 1 {
            if color == Color::White && or > rank {
                return false;
            }
            if color == Color::Black && or < rank {
                return false;
            }
        }
    }
    true
}

fn eval_bishop_pair(pos: &Position) -> (i32, i32) {
    let white_bishops = pos.bb[Color::White as usize][PieceType::Bishop as usize].count_ones();
    let black_bishops = pos.bb[Color::Black as usize][PieceType::Bishop as usize].count_ones();
    let mut mg = 0;
    let mut eg = 0;
    if white_bishops >= 2 {
        mg += 20;
        eg += 40;
    }
    if black_bishops >= 2 {
        mg -= 20;
        eg -= 40;
    }
    (mg, eg)
}

fn eval_mobility(pos: &Position) -> (i32, i32) {
    let occ = pos.occupied_all;
    let mut mg = 0;
    let mut eg = 0;
    for &color in &[Color::White, Color::Black] {
        let sign = if color == Color::White { 1 } else { -1 };
        let own = pos.occupied[color as usize];
        // Knights
        let mut bb = pos.bb[color as usize][PieceType::Knight as usize];
        while bb != 0 {
            let sq = bb.trailing_zeros() as u8;
            bb &= bb - 1;
            let mob = (knight_attacks(sq) & !own).count_ones() as i32;
            mg += sign * mob * 4;
            eg += sign * mob * 4;
        }
        // Bishops
        let mut bb = pos.bb[color as usize][PieceType::Bishop as usize];
        while bb != 0 {
            let sq = bb.trailing_zeros() as u8;
            bb &= bb - 1;
            let mob = (bishop_attacks(sq, occ) & !own).count_ones() as i32;
            mg += sign * mob * 3;
            eg += sign * mob * 3;
        }
        // Rooks
        let mut bb = pos.bb[color as usize][PieceType::Rook as usize];
        while bb != 0 {
            let sq = bb.trailing_zeros() as u8;
            bb &= bb - 1;
            let mob = (rook_attacks(sq, occ) & !own).count_ones() as i32;
            mg += sign * mob * 2;
            eg += sign * mob * 2;
        }
        // Queens
        let mut bb = pos.bb[color as usize][PieceType::Queen as usize];
        while bb != 0 {
            let sq = bb.trailing_zeros() as u8;
            bb &= bb - 1;
            let mob = (queen_attacks(sq, occ) & !own).count_ones() as i32;
            mg += sign * mob * 1;
            eg += sign * mob * 1;
        }
    }
    (mg, eg)
}

fn eval_rooks(pos: &Position) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = 0;
    for &color in &[Color::White, Color::Black] {
        let sign = if color == Color::White { 1 } else { -1 };
        let mut bb = pos.bb[color as usize][PieceType::Rook as usize];
        // pawn per file masks quickly via file occupancy
        let mut pawn_files_white = [false; 8];
        let mut pawn_files_black = [false; 8];
        for sq in 0..64 {
            let p = pos.board[sq];
            if p != NO_PIECE && piece_type(p) == PieceType::Pawn {
                let f = (sq % 8) as usize;
                if piece_color(p) == Color::White { pawn_files_white[f]=true; } else { pawn_files_black[f]=true; }
            }
        }
        while bb != 0 {
            let sq = bb.trailing_zeros() as u8;
            bb &= bb - 1;
            let f = (sq % 8) as usize;
            let own_pawn = if color == Color::White { pawn_files_white[f] } else { pawn_files_black[f] };
            let opp_pawn = if color == Color::White { pawn_files_black[f] } else { pawn_files_white[f] };
            if !own_pawn && !opp_pawn {
                mg += sign * 15;
                eg += sign * 15;
            } else if !own_pawn {
                mg += sign * 8;
                eg += sign * 12;
            }
        }
    }
    (mg, eg)
}

fn eval_king_safety(pos: &Position) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = 0;
    for &color in &[Color::White, Color::Black] {
        let sign = if color == Color::White { 1 } else { -1 };
        let king_sq = pos.king_square(color);
        let king_file = (king_sq % 8) as i8;
        let king_rank = (king_sq / 8) as i8;
        // Only evaluate shield if king is somewhat castled (rank 0 or 1 for white, 6-7 for black)
        let is_white = color == Color::White;
        let shield_rank = if is_white { king_rank + 1 } else { king_rank - 1 };
        if shield_rank < 0 || shield_rank > 7 {
            continue;
        }
        let mut shield = 0;
        for df in -1..=1 {
            let f = king_file + df;
            if f < 0 || f > 7 { continue; }
            let sq = (shield_rank * 8 + f) as u8;
            let p = pos.board[sq as usize];
            if p != NO_PIECE && piece_type(p) == PieceType::Pawn && piece_color(p) == color {
                shield += 1;
                mg += sign * 8;
                eg += sign * 0;
            } else {
                // missing pawn
                mg += sign * -10;
                eg += sign * 0;
            }
            // second rank shield (more distant)
            let shield_rank2 = if is_white { king_rank + 2 } else { king_rank - 2 };
            if shield_rank2 >=0 && shield_rank2 <=7 {
                let sq2 = (shield_rank2*8 + f) as u8;
                let p2 = pos.board[sq2 as usize];
                if p2 != NO_PIECE && piece_type(p2)==PieceType::Pawn && piece_color(p2)==color {
                    mg += sign * 4;
                }
            }
        }
        // If no shield pawns at all, extra penalty already via -10 per file -> -30
        // Mobility-like king safety could add more but keep simple
    }
    (mg, eg)
}
