use crate::types::Move;

/// TT flags
pub const TT_EXACT: u8 = 0;
pub const TT_LOWER: u8 = 1; // beta cutoff
pub const TT_UPPER: u8 = 2; // alpha fail

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub key: u64,
    pub best_move: Move,
    pub score: i32,
    pub depth: u8,
    pub flag: u8,
    pub age: u8,
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            key: 0,
            best_move: Move::NULL,
            score: 0,
            depth: 0,
            flag: TT_EXACT,
            age: 0,
        }
    }
}

pub struct TranspositionTable {
    pub entries: Vec<TTEntry>,
    pub mask: usize,
    pub age: u8,
    // stats
    pub hits: u64,
    pub stores: u64,
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        let bytes = mb * 1024 * 1024;
        let entry_size = std::mem::size_of::<TTEntry>();
        // ensure at least 1 entry
        let mut num_entries = (bytes / entry_size).max(1);
        // round down to power of two
        let mut pow2 = 1usize;
        while pow2 * 2 <= num_entries {
            pow2 *= 2;
        }
        num_entries = pow2;
        let entries = vec![TTEntry::default(); num_entries];
        let mask = num_entries - 1;
        Self {
            entries,
            mask,
            age: 0,
            hits: 0,
            stores: 0,
        }
    }

    pub fn with_default() -> Self {
        Self::new(16)
    }

    pub fn resize(&mut self, mb: usize) {
        *self = Self::new(mb);
    }

    pub fn clear(&mut self) {
        for e in &mut self.entries {
            *e = TTEntry::default();
        }
        self.age = self.age.wrapping_add(1);
        self.hits = 0;
        self.stores = 0;
    }

    pub fn new_search(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    #[inline]
    fn index(&self, hash: u64) -> usize {
        // Use upper bits for better distribution? For now simple
        (hash as usize) & self.mask
        // Alternative: (hash >> 16) as usize & mask for better?
    }

    pub fn probe(&mut self, hash: u64) -> Option<TTEntry> {
        let idx = self.index(hash);
        let e = self.entries[idx];
        if e.key == hash && e.key != 0 {
            self.hits += 1;
            Some(e)
        } else {
            None
        }
    }

    pub fn store(&mut self, hash: u64, best_move: Move, score: i32, depth: u8, flag: u8, ply: i32) {
        let idx = self.index(hash);
        let mut adjusted_score = score;
        // Mate score adjustment to make it independent of ply
        // When storing, convert mate scores to be from root perspective
        if score > 29000 {
            adjusted_score += ply;
        } else if score < -29000 {
            adjusted_score -= ply;
        }
        let existing = self.entries[idx];
        // Replacement: if empty, or age different, or depth >= existing, or existing is old
        let should_replace = existing.key == 0
            || existing.age != self.age
            || depth >= existing.depth
            || flag == TT_EXACT;

        if should_replace {
            self.entries[idx] = TTEntry {
                key: hash,
                best_move,
                score: adjusted_score,
                depth,
                flag,
                age: self.age,
            };
            self.stores += 1;
        }
    }

    pub fn retrieve_with_correction(&self, entry: TTEntry, ply: i32) -> (i32, Move, u8, u8) {
        let mut score = entry.score;
        if score > 29000 {
            score -= ply;
        } else if score < -29000 {
            score += ply;
        }
        (score, entry.best_move, entry.depth, entry.flag)
    }

    pub fn hashfull(&self) -> u32 {
        // per mille
        let used = self.entries.iter().filter(|e| e.key != 0 && e.age == self.age).count();
        ((used * 1000) / self.entries.len()) as u32
    }

    pub fn size_mb(&self) -> usize {
        self.entries.len() * std::mem::size_of::<TTEntry>() / (1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Move;
    #[test]
    fn tt_store_probe() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x12345678abcdef00;
        let mv = Move::new(12, 28, None);
        tt.store(hash, mv, 100, 5, TT_EXACT, 0);
        let e = tt.probe(hash).unwrap();
        assert_eq!(e.best_move.0, mv.0);
        assert_eq!(e.score, 100);
        assert_eq!(e.depth, 5);
    }
    #[test]
    fn tt_mate_adjust() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1;
        tt.store(hash, Move::NULL, 29990, 10, TT_EXACT, 5);
        let e = tt.probe(hash).unwrap();
        let (score, _, _, _) = tt.retrieve_with_correction(e, 5);
        assert_eq!(score, 29990);
        // When probing at different ply, should adjust
        let (score2, _, _, _) = tt.retrieve_with_correction(e, 0);
        assert_eq!(score2, 29995); // 29990+5
    }
}
