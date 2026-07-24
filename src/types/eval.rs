// Centipawns
pub type Eval = i16;

// Define infinity scores
pub const MIN_EVAL: Eval = -32_768;

// Defineing a mate threshold
pub const MATE: Eval = 32000;

// If those close to mate, it is going to be mate
pub const MATE_BOUND: Eval = MATE - 1024;

// Convert a search-relative score (mate distance measured from the root) into a node-relative score
pub fn score_to_tt(score: Eval, ply: usize) -> Eval {
    if score >= MATE_BOUND {
        score + ply as Eval
    } else if score <= -MATE_BOUND {
        score - ply as Eval
    } else {
        score
    }
}

// Inverse of score_to_tt
pub fn score_from_tt(score: Eval, ply: usize) -> Eval {
    if score >= MATE_BOUND {
        score - ply as Eval
    } else if score <= -MATE_BOUND {
        score + ply as Eval
    } else {
        score
    }
}
