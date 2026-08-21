use siroco::position::Position;
use siroco::search::{search, SearchLimits};
use siroco::types::Move;

/// Plays a game from startpos using shallow search, ensures no illegal moves, no panic, and game terminates reasonably
fn play_game(max_plies: usize, depth: u32) -> (String, usize, bool) {
    let mut pos = Position::new();
    let mut moves: Vec<String> = Vec::new();
    let mut illegal = false;
    for ply in 0..max_plies {
        if pos.is_draw() {
            return ("draw".to_string(), ply, false);
        }
        let mut list = siroco::types::MoveList::new();
        siroco::movegen::generate_legal(&mut pos, &mut list);
        if list.is_empty() {
            // mate or stalemate
            let is_check = siroco::movegen::is_square_attacked(
                &pos,
                pos.king_square(pos.side_to_move),
                pos.side_to_move.opposite(),
            );
            if is_check {
                return ("mate".to_string(), ply, false);
            } else {
                return ("stalemate".to_string(), ply, false);
            }
        }
        let limits = SearchLimits {
            depth: Some(depth),
            ..Default::default()
        };
        let (best, _score) = search(&mut pos, limits);
        if best.is_null() {
            illegal = true;
            break;
        }
        // verify legal
        let mut legal_list = siroco::types::MoveList::new();
        siroco::movegen::generate_legal(&mut pos, &mut legal_list);
        let mut found = false;
        for &m in legal_list.as_slice() {
            if m.0 == best.0 {
                found = true;
                break;
            }
        }
        if !found {
            illegal = true;
            eprintln!("illegal move generated: {} at ply {}", best.to_uci(), ply);
            eprintln!("FEN: {}", pos.to_fen());
            break;
        }
        moves.push(best.to_uci());
        pos.make_move(best);
        // verify hash consistency
        assert_eq!(pos.hash, pos.compute_hash(), "hash mismatch after {}", best.to_uci());
    }
    ("maxplies".to_string(), max_plies, illegal)
}

#[test]
fn autoplay_shallow() {
    let (result, plies, illegal) = play_game(200, 3);
    assert!(!illegal, "generated illegal move");
    println!("autoplay shallow result {} plies {}", result, plies);
    // should have terminated via mate/draw or maxplies, but not crash
    assert!(plies > 10, "game too short, maybe search broken");
}

#[test]
fn autoplay_multiple_games() {
    for i in 0..5 {
        let depth = 2 + (i % 2); // alternate depth 2,3
        let (result, plies, illegal) = play_game(150, depth);
        assert!(!illegal, "game {} illegal", i);
        println!("game {} result {} plies {}", i, result, plies);
        assert!(plies > 5);
    }
}

#[test]
fn autoplay_from_tricky_positions() {
    let fens = vec![
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    ];
    for fen in fens {
        let mut pos = Position::from_fen(fen).unwrap();
        let limits = SearchLimits { depth: Some(3), ..Default::default() };
        let (best, _score) = search(&mut pos, limits);
        assert!(!best.is_null(), "no move from fen {}", fen);
        // verify legal
        let mut list = siroco::types::MoveList::new();
        siroco::movegen::generate_legal(&mut pos, &mut list);
        assert!(list.as_slice().contains(&best), "best not legal for {}", fen);
        pos.make_move(best);
        assert_eq!(pos.hash, pos.compute_hash());
        pos.unmake_move(best);
        assert_eq!(pos.to_fen(), fen);
    }
}
