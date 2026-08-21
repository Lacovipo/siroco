use crate::position::Position;
use crate::types::*;

/// Syzygy stub para V0.5
/// Probe WDL para finales <=5 piezas sin ficheros TB
/// Retorna Some(2)=win, Some(1)=draw, Some(0)=loss desde perspectiva del lado a mover
/// None = desconocido (fallback a búsqueda)
/// Cuando se disponga de ficheros Syzygy, aquí se llamaría a shakmaty-syzygy
pub struct SyzygyTablebase {
    pub path: Option<String>,
    pub enabled: bool,
}

impl SyzygyTablebase {
    pub fn new() -> Self {
        Self { path: None, enabled: false }
    }
    pub fn set_path(&mut self, path: String) {
        if path.is_empty() || path == "<empty>" {
            self.path = None;
            self.enabled = false;
        } else {
            // En V0.5 stub, no usamos ficheros pero guardamos path para UCI
            // Si el path existe y contiene .rtbw, en V0.6 se cargaría con shakmaty-syzygy
            self.path = Some(path);
            self.enabled = true;
        }
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for SyzygyTablebase {
    fn default() -> Self {
        Self::new()
    }
}

/// Probe WDL simple para 2-5 piezas
/// Lógica basada en conocimiento de finales, no en TB reales, pero demuestra integración
pub fn probe_wdl(pos: &Position) -> Option<i32> {
    let total = pos.occupied_all.count_ones() as usize;
    if total > 5 {
        return None;
    }
    if total == 2 {
        return Some(1);
    }
    // Si estamos en jaque? Syzygy no distingue, pero lo manejamos
    // Count material
    let white_pieces: Vec<(PieceType, u8)> = (0..64)
        .filter_map(|sq| {
            let p = pos.board[sq];
            if p != NO_PIECE && piece_color(p) == Color::White {
                Some((piece_type(p), sq as u8))
            } else { None }
        })
        .collect();
    let black_pieces: Vec<(PieceType, u8)> = (0..64)
        .filter_map(|sq| {
            let p = pos.board[sq];
            if p != NO_PIECE && piece_color(p) == Color::Black {
                Some((piece_type(p), sq as u8))
            } else { None }
        })
        .collect();

    let w_has = |pt| white_pieces.iter().any(|(t, _)| *t == pt);
    let b_has = |pt| black_pieces.iter().any(|(t, _)| *t == pt);
    let w_count = |pt| white_pieces.iter().filter(|(t, _)| *t == pt).count();
    let b_count = |pt| black_pieces.iter().filter(|(t, _)| *t == pt).count();

    // K vs K
    if total == 2 {
        return Some(1); // draw
    }

    // 3 piezas
    if total == 3 {
        let side = pos.side_to_move;
        // KQ vs K, KR vs K -> win for side with Q/R, loss for bare king side
        if side == Color::White {
            if w_count(PieceType::Queen) == 1 && white_pieces.len() == 2 {
                return Some(2);
            }
            if w_count(PieceType::Rook) == 1 && white_pieces.len() == 2 {
                return Some(2);
            }
            if b_count(PieceType::Queen) == 1 && black_pieces.len() == 2 {
                return Some(0);
            }
            if b_count(PieceType::Rook) == 1 && black_pieces.len() == 2 {
                return Some(0);
            }
        } else {
            if b_count(PieceType::Queen) == 1 && black_pieces.len() == 2 {
                return Some(2);
            }
            if b_count(PieceType::Rook) == 1 && black_pieces.len() == 2 {
                return Some(2);
            }
            if w_count(PieceType::Queen) == 1 && white_pieces.len() == 2 {
                return Some(0);
            }
            if w_count(PieceType::Rook) == 1 && white_pieces.len() == 2 {
                return Some(0);
            }
        }
        // KB vs K, KN vs K -> draw (both sides)
        if (w_count(PieceType::Bishop) == 1 || w_count(PieceType::Knight) == 1) && white_pieces.len() == 2 {
            return Some(1);
        }
        if (b_count(PieceType::Bishop) == 1 || b_count(PieceType::Knight) == 1) && black_pieces.len() == 2 {
            return Some(1);
        }
        // KP vs K -> unknown
        return None;
    }

    // 4 piezas: KBB vs K -> win for bishop side, loss for bare
    if total == 4 {
        let side = pos.side_to_move;
        if w_count(PieceType::Bishop) == 2 && white_pieces.len() == 3 {
            return if side == Color::White { Some(2) } else { Some(0) };
        }
        if b_count(PieceType::Bishop) == 2 && black_pieces.len() == 3 {
            return if side == Color::Black { Some(2) } else { Some(0) };
        }
        return None;
    }

    // 5 piezas: desconocido
    None
}

/// Convierte WDL a score para búsqueda
/// Win -> MATE - ply, Loss -> -MATE + ply, Draw -> 0
pub fn wdl_to_score(wdl: i32, ply: i32) -> i32 {
    match wdl {
        2 => 29000 - ply, // win
        0 => -29000 + ply, // loss
        _ => 0, // draw or unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;
    #[test]
    fn syzygy_kq_vs_k() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/K1k1Q3 w - - 0 1").unwrap();
        assert_eq!(probe_wdl(&pos), Some(2));
    }
    #[test]
    fn syzygy_k_vs_k() {
        let pos = Position::from_fen("8/8/8/8/8/8/4k3/4K3 w - - 0 1").unwrap();
        assert_eq!(probe_wdl(&pos), Some(1));
    }
    #[test]
    fn syzygy_kb_vs_k() {
        let pos = Position::from_fen("8/8/8/8/8/8/4k3/4KB2 w - - 0 1").unwrap();
        assert_eq!(probe_wdl(&pos), Some(1));
    }
}
