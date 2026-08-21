use crate::position::Position;
use crate::types::Move;
use std::fs::File;
use std::io::Write;

/// Simple tree dumper for debugging search
/// Usage: call `dump_tree(pos, depth, "tree.log")` to dump principal tree
pub fn dump_tree(pos: &mut Position, depth: u32, path: &str) {
    let mut file = File::create(path).expect("create tree.log");
    writeln!(file, "FEN: {}", pos.to_fen()).unwrap();
    writeln!(file, "depth {}", depth).unwrap();
    dump_recursive(pos, depth, 0, &mut file, -30000, 30000).unwrap();
}

fn dump_recursive(
    pos: &mut Position,
    depth: u32,
    ply: u32,
    file: &mut File,
    alpha: i32,
    beta: i32,
) -> std::io::Result<()> {
    if depth == 0 {
        let eval = crate::eval::evaluate(pos);
        writeln!(
            file,
            "{:indent$}ply {} eval {} alpha {} beta {} fen {}",
            "",
            ply,
            eval,
            alpha,
            beta,
            pos.to_fen(),
            indent = (ply as usize) * 2
        )?;
        return Ok(());
    }
    let mut list = crate::types::MoveList::new();
    crate::movegen::generate_legal(pos, &mut list);
    if list.is_empty() {
        writeln!(
            file,
            "{:indent$}ply {} no moves fen {}",
            "",
            ply,
            pos.to_fen(),
            indent = (ply as usize) * 2
        )?;
        return Ok(());
    }
    for &mv in list.as_slice() {
        let indent = (ply as usize) * 2;
        writeln!(
            file,
            "{:indent$}ply {} move {} alpha {} beta {}",
            "",
            ply,
            mv.to_uci(),
            alpha,
            beta,
            indent = indent
        )?;
        pos.make_move(mv);
        dump_recursive(pos, depth - 1, ply + 1, file, -beta, -alpha)?;
        pos.unmake_move(mv);
    }
    Ok(())
}

/// Trace a single line PV
pub fn trace_pv(pos: &mut Position, moves: &[Move], path: &str) {
    let mut file = File::create(path).unwrap();
    writeln!(file, "start {}", pos.to_fen()).unwrap();
    for &mv in moves {
        writeln!(file, "make {}", mv.to_uci()).unwrap();
        pos.make_move(mv);
        writeln!(file, "fen {}", pos.to_fen()).unwrap();
        writeln!(file, "eval {}", crate::eval::evaluate(pos)).unwrap();
    }
}
