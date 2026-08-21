use crate::history::{HistoryTable, KillerTable};
use crate::position::Position;
use crate::search::{search_with_tt, SearchLimits, MATE};
use crate::syzygy::SyzygyTablebase;
use crate::tt::TranspositionTable;
use crate::types::Move;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Instant;

/// Lazy SMP: lanza N-1 helpers que comparten TT.
/// Cada helper busca iterativamente a profundidades crecientes; el main controla tiempo.
pub fn search_parallel(
    pos: &Position,
    limits: SearchLimits,
    tt: Arc<Mutex<TranspositionTable>>,
    num_threads: usize,
    syzygy: Arc<SyzygyTablebase>,
) -> (Move, i32) {
    if num_threads <= 1 {
        // fallback a single thread
        let mut tt_guard = tt.lock().unwrap();
        let mut hist = HistoryTable::new();
        let mut killers = KillerTable::new();
        let mut pos_clone = pos.clone();
        return search_with_tt(&mut pos_clone, limits, &mut *tt_guard, &mut hist, &mut killers, &*syzygy);
    }

    let stop = Arc::new(AtomicBool::new(false));
    let best_shared = Arc::new(Mutex::new((Move::NULL, -MATE, 0u64))); // move, score, depth
    let start = Instant::now();

    // Soft/hard limits para helpers: usan mismo limits, pero respetan stop
    let mut handles = Vec::new();

    // Helpers = num_threads -1, main is the remaining
    let num_helpers = num_threads.saturating_sub(1);
    for thread_id in 0..num_helpers {
        let pos_clone = pos.clone();
        let tt_clone = Arc::clone(&tt);
        let stop_clone = Arc::clone(&stop);
        let best_clone = Arc::clone(&best_shared);
        let limits_clone = limits.clone();
        let syzygy_clone = Arc::clone(&syzygy);
        let start_clone = start;

        let handle = std::thread::spawn(move || {
            // Cada helper tiene su propio history/killers
            let mut hist = HistoryTable::new();
            let mut killers = KillerTable::new();
            let mut local_pos = pos_clone;
            // Stagger start depth to diversify: thread 0 start depth 1, thread 1 start depth 2 etc.
            // Para simplicidad, todos empiezan en 1 pero helper duerme un poco si thread_id>0
            if thread_id > 0 {
                std::thread::sleep(std::time::Duration::from_millis((thread_id * 10) as u64));
            }
            // Helper loop: iterative deepening até max_depth o stop
            let max_depth = limits_clone.depth.unwrap_or(64) as i32;
            for depth in 1..=max_depth {
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                // Lock TT for this depth iteration (coarse, pero simple)
                // Para reducir contención, probamos a lock solo para probe/store ya dentro de search;
                // aquí hacemos un lock por depth: search_with_tt necesita &mut TT, así que lockeamos
                let mut tt_guard = match tt_clone.try_lock() {
                    Ok(g) => g,
                    Err(_) => {
                        // TT ocupada, duerme y reintenta
                        std::thread::sleep(std::time::Duration::from_millis(1));
                        if stop_clone.load(Ordering::Relaxed) {
                            break;
                        }
                        match tt_clone.lock() {
                            Ok(g) => g,
                            Err(_) => break,
                        }
                    }
                };
                // Para helpers, usamos límites sin tiempo (solo depth) y respetan stop global
                // Creamos limits sin soft/hard para que no corten por tiempo, solo depth
                let mut helper_limits = limits_clone.clone();
                // Helpers ignoran movetime/wtime, solo depth
                helper_limits.movetime = None;
                helper_limits.wtime = None;
                helper_limits.btime = None;
                helper_limits.winc = None;
                helper_limits.binc = None;
                // Pero si main ya superó soft, stop será true y helper saldrá
                let (best, score) = {
                    search_with_tt(
                        &mut local_pos,
                        helper_limits,
                        &mut *tt_guard,
                        &mut hist,
                        &mut killers,
                        &*syzygy_clone,
                    )
                };
                drop(tt_guard);
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                if !best.is_null() {
                    let mut shared = best_clone.lock().unwrap();
                    // Actualiza si profundidad mayor o score mejor a misma profundidad
                    if depth as u64 > shared.2 || (depth as u64 == shared.2 && score > shared.1) {
                        *shared = (best, score, depth as u64);
                    }
                }
                // Pequeña pausa para no saturar
                if depth >= 6 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                // Helpers siguen hasta stop
                if start_clone.elapsed().as_millis() > 10000 && depth > 8 {
                    // evita bucle infinito en test
                }
            }
        });
        handles.push(handle);
    }

    // Main thread hace búsqueda real con tiempo
    let (main_best, main_score) = {
        let mut tt_guard = tt.lock().unwrap();
        let mut hist = HistoryTable::new();
        let mut killers = KillerTable::new();
        let mut pos_clone = pos.clone();
        search_with_tt(&mut pos_clone, limits.clone(), &mut *tt_guard, &mut hist, &mut killers, &*syzygy)
    };

    // Señala stop y espera helpers
    stop.store(true, Ordering::Relaxed);
    for h in handles {
        let _ = h.join();
    }

    // Elige mejor entre main y helpers
    let shared = best_shared.lock().unwrap();
    if shared.0.is_null() {
        (main_best, main_score)
    } else {
        // Si helper encontró mate más rápido o score mejor a mayor profundidad, prefiere helper
        // Pero main ya tiene score con tiempo controlado, así que prioriza main si no es claramente peor
        // Para simplicidad, si helper depth > main depth estimado, usa helper
        // Como no tenemos depth de main, comparamos scores: si helper mate y main no, usa helper
        if shared.1.abs() > crate::search::MATE - 100 && main_score.abs() <= crate::search::MATE - 100 {
            (shared.0, shared.1)
        } else if shared.2 > 12 && shared.1 > main_score + 30 {
            (shared.0, shared.1)
        } else {
            (main_best, main_score)
        }
    }
}
