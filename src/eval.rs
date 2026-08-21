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
        // PST index: mirror for black
        let pst_sq = if color == Color::White {
            sq
        } else {
            sq ^ 56
        };
        mg += sign * (MG_VALUES[pt] + PST_MG[pt][pst_sq]);
        eg += sign * (EG_VALUES[pt] + PST_EG[pt][pst_sq]);
        phase += PHASE_WEIGHTS[pt];
    }

    if phase > MAX_PHASE {
        phase = MAX_PHASE;
    }
    // tapered
    let score = (mg * phase + eg * (MAX_PHASE - phase)) / MAX_PHASE;
    // tempo bonus
    let tempo = 10;
    let final_score = if pos.side_to_move == Color::White {
        score + tempo
    } else {
        -score - tempo
    };
    // Ensure we return from side to move perspective
    // Our mg/eg calculation already gave white-positive, we flipped above.
    // Actually above we did white-positive then flipped? Let's recompute: mg/eg already white-positive (white + , black -). So score is white perspective. Then we return side-to-move perspective: if white to move, return score, if black to move, return -score + tempo? We already baked tempo incorrectly.
    // Simplify: if side is white, return score+tempo, else return -score + tempo? Wait tempo is bonus for side to move irrespective of color. So if white to move, white gets +10, if black to move, black gets +10 which is -10 white perspective.
    // So white perspective adjusted: adjusted = score + (if white to move { tempo } else { -tempo })
    // Then side perspective = if white to move { adjusted } else { -adjusted }
    // That equals: if white { score+tempo } else { -score+tempo }? Let's compute: adjusted = score + side_sign*tempo where side_sign = +1 white, -1 black. Then side score = side_sign * adjusted = side_sign*score + tempo.
    // So side score = side_sign*score + tempo. That matches we did: white: score+tempo, black: -score + tempo? Wait we did -score -tempo earlier wrong.
    // Correct:
    // let side_sign = if white {1} else {-1}
    // adjusted_white = score + side_sign*tempo
    // side_score = side_sign * adjusted_white = side_sign*score + tempo
    // So fix:

    let side_sign = if pos.side_to_move == Color::White { 1 } else { -1 };
    side_sign * score + tempo
}
