use crate::movegen::generate_legal;
use crate::position::Position;
use crate::types::MoveList;

pub fn perft(pos: &mut Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut list = MoveList::new();
    generate_legal(pos, &mut list);
    if depth == 1 {
        return list.len() as u64;
    }
    let mut nodes = 0u64;
    let moves = list.as_slice().to_vec();
    for mv in moves {
        pos.make_move(mv);
        nodes += perft(pos, depth - 1);
        pos.unmake_move(mv);
    }
    nodes
}

pub fn perft_divide(pos: &mut Position, depth: u32) -> Vec<(String, u64)> {
    let mut list = MoveList::new();
    generate_legal(pos, &mut list);
    let mut result = Vec::new();
    let moves = list.as_slice().to_vec();
    for mv in moves {
        pos.make_move(mv);
        let nodes = if depth > 1 { perft(pos, depth - 1) } else { 1 };
        pos.unmake_move(mv);
        result.push((mv.to_uci(), nodes));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perft_test(fen: &str, depth: u32, expected: u64) {
        let mut pos = Position::from_fen(fen).unwrap();
        let nodes = perft(&mut pos, depth);
        assert_eq!(nodes, expected, "perft fen={} depth={}", fen, depth);
    }

    #[test]
    fn perft_startpos() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let expected = [20, 400, 8902, 197281, 4865609];
        for (i, &exp) in expected.iter().enumerate() {
            perft_test(fen, (i + 1) as u32, exp);
        }
    }

    #[test]
    fn perft_kiwipete() {
        // Kiwipete from CPW
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let expected = [(1, 48), (2, 2039), (3, 97862), (4, 4085603)];
        for (d, exp) in expected {
            perft_test(fen, d, exp);
        }
    }

    #[test]
    fn perft_position3() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let expected = [(1, 14), (2, 191), (3, 2812), (4, 43238)];
        for (d, exp) in expected {
            perft_test(fen, d, exp);
        }
    }
}
