use siroco::perft::perft;
use siroco::position::Position;

fn assert_perft(fen: &str, depth: u32, expected: u64) {
    let mut pos = Position::from_fen(fen).unwrap();
    let nodes = perft(&mut pos, depth);
    assert_eq!(nodes, expected, "FEN: {} depth {}", fen, depth);
}

#[test]
fn perft_extended_suite() {
    // Standard perft suite from CPW
    let tests = vec![
        // startpos depth 5 already tested but include
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5, 4865609),
        // Kiwipete
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 3, 97862),
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 4, 4085603),
        // position 3
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, 674624),
        // position 4 - with castling and promotion tricky
        ("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 3, 9467),
        ("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 4, 422333),
        // position 5
        ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 3, 62379),
        ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 4, 2103487),
    ];
    for (fen, depth, exp) in tests {
        assert_perft(fen, depth, exp);
    }
}

#[test]
fn perft_en_passant_promo() {
    // En passant and promo heavy
    let fen = "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1";
    // known perft values for this tricky position
    // from CPW: depth 1 = 24, 2=496 etc? We'll test depth 1,2
    let mut pos = Position::from_fen(fen).unwrap();
    assert_eq!(perft(&mut pos, 1), 24);
    assert_eq!(perft(&mut pos, 2), 496);
}
