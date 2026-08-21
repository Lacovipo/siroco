use crate::types::Move;

pub const MAX_PLY: usize = 64;

#[derive(Clone)]
pub struct KillerTable {
    pub killers: [[Move; 2]; MAX_PLY],
}

impl KillerTable {
    pub fn new() -> Self {
        KillerTable {
            killers: [[Move::NULL; 2]; MAX_PLY],
        }
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    #[inline]
    pub fn is_killer(&self, ply: usize, mv: Move) -> bool {
        if ply >= MAX_PLY {
            return false;
        }
        self.killers[ply][0].0 == mv.0 || self.killers[ply][1].0 == mv.0
    }
    #[inline]
    pub fn score_killer(&self, ply: usize, mv: Move) -> i32 {
        if ply >= MAX_PLY {
            return 0;
        }
        if self.killers[ply][0].0 == mv.0 {
            8000
        } else if self.killers[ply][1].0 == mv.0 {
            7000
        } else {
            0
        }
    }
    pub fn store(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_PLY {
            return;
        }
        if self.killers[ply][0].0 == mv.0 {
            return;
        }
        // shift
        self.killers[ply][1] = self.killers[ply][0];
        self.killers[ply][0] = mv;
    }
}

#[derive(Clone)]
pub struct HistoryTable {
    // [color][from][to]
    pub table: [[[i32; 64]; 64]; 2],
}

impl HistoryTable {
    pub fn new() -> Self {
        HistoryTable {
            table: [[[0; 64]; 64]; 2],
        }
    }
    pub fn clear(&mut self) {
        self.table = [[[0; 64]; 64]; 2];
    }
    #[inline]
    pub fn get(&self, color: usize, from: u8, to: u8) -> i32 {
        self.table[color][from as usize][to as usize]
    }
    #[inline]
    pub fn score(&self, color: usize, mv: Move) -> i32 {
        let from = mv.from_sq();
        let to = mv.to_sq();
        self.table[color][from as usize][to as usize]
    }
    pub fn update(&mut self, color: usize, mv: Move, depth: i32, bonus: i32) {
        let from = mv.from_sq() as usize;
        let to = mv.to_sq() as usize;
        let entry = &mut self.table[color][from][to];
        // Clamp to avoid overflow, using gravity
        // Stockfish style: history += bonus - history * bonus / 16384
        // For V1.1 simple: add depth*depth, clamp 16000
        let delta = bonus;
        *entry += delta - (*entry * delta.abs() / 16384);
        // Clamp
        if *entry > 16384 {
            *entry = 16384;
        } else if *entry < -16384 {
            *entry = -16384;
        }
    }
    pub fn update_quiet(&mut self, color: usize, mv: Move, depth: i32, is_good: bool) {
        let bonus = if is_good { depth * depth } else { -depth * depth };
        self.update(color, mv, depth, bonus);
    }
}

impl Default for KillerTable {
    fn default() -> Self {
        Self::new()
    }
}
impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}
