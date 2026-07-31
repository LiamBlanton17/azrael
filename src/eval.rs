mod pst;

use crate::search::magics::bishop::get_bishop_moves;
use crate::search::magics::rook::get_rook_moves;
use crate::search::move_generation::KNIGHT_MOVES;
use crate::types::bidboard::BitBoard;
use crate::types::color::Color;
use crate::types::eval::Eval;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::square::Square;

use pst::{ENDGAME, OPENING, PST};

// Define material evaluations
pub const PAWN: Eval = 100;
pub const KNIGHT: Eval = 320;
pub const BISHOP: Eval = 330;
pub const ROOK: Eval = 515;
pub const QUEEN: Eval = 945;

// Weights for calculating game phase
const PAWN_PHASE: i32 = 0;
const KNIGHT_PHASE: i32 = 2;
const BISHOP_PHASE: i32 = 2;
const ROOK_PHASE: i32 = 5;
const QUEEN_PHASE: i32 = 9;

// The maximum possible phase
const TOTAL_PHASE: i32 = PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

// Lazy eval margin to beta
const LAZY_MARGIN: Eval = 250;

impl Position {

    pub fn eval(&self, lazy_eval: bool, alpha: Eval, beta: Eval) -> Eval {
        let phase = get_phase_score(self);
        let mut eval = pst_eval(self, phase);
        if lazy_eval { // if clearly outside of a/b with margin, return only the lazy pst eval
            if eval - LAZY_MARGIN >= beta {
                return eval;
            }
            if eval + LAZY_MARGIN <= alpha {
                return eval;
            }
        }
        eval += pawn_structure_eval(self, phase);
        eval += bishop_pair_eval(self, phase);
        eval += tempo_eval(self, phase);
        eval += king_safety_eval(self, phase);
        eval += file_based_eval(self, phase);
        eval += mobility_eval(self, phase);

        eval
    }

    pub fn eval_relative_lazy(&self, alpha: Eval, beta: Eval) -> Eval {
        match self.turn  {
            Color::White => self.eval(true, alpha, beta),
            Color::Black => -self.eval(true, alpha, beta),
        }
    }

    pub fn eval_relative(&self) -> Eval {
        match self.turn  {
            Color::White => self.eval(false, 0, 0),
            Color::Black => -self.eval(false, 0, 0),
        }
    }

    // Add a piece from the pst scores
    #[inline]
    pub fn pst_add_piece(&mut self, piece: Piece, color: Color, sq: Square) {
        let piece_idx = piece.idx();
        let (idx, sign) = match color {
            Color::White => (sq.idx(), 1),
            Color::Black => (sq.idx() ^ 56, -1),
        };
        self.pst_opening += sign * PST[OPENING][piece_idx][idx] as i32;
        self.pst_endgame += sign * PST[ENDGAME][piece_idx][idx] as i32;
        self.phase_material += phase_weight(piece);
    }

    // Remove a piece from the pst scores
    #[inline]
    pub fn pst_remove_piece(&mut self, piece: Piece, color: Color, sq: Square) {
        let piece_idx = piece.idx();
        let (idx, sign) = match color {
            Color::White => (sq.idx(), 1),
            Color::Black => (sq.idx() ^ 56, -1),
        };
        self.pst_opening -= sign * PST[OPENING][piece_idx][idx] as i32;
        self.pst_endgame -= sign * PST[ENDGAME][piece_idx][idx] as i32;
        self.phase_material -= phase_weight(piece);
    }

    // Recompute the psts from scratch - used once at board setup
    pub fn recompute_psts(&mut self) {
        let (opening, endgame, phase_material) = self.compute_psts();
        self.pst_opening = opening;
        self.pst_endgame = endgame;
        self.phase_material = phase_material;
    }

    // Pure from-scratch computation of the psts 
    pub fn compute_psts(&self) -> (i32, i32, i32) {
        let mut opening: i32 = 0;
        let mut endgame: i32 = 0;
        let mut phase_material: i32 = 0;

        for color in [Color::White, Color::Black] {
            for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King] {
                let pi = piece.idx();
                for sq in self.get_piece(piece, color) {
                    let (idx, sign) = match color {
                        Color::White => (sq.idx(), 1),
                        Color::Black => (sq.idx() ^ 56, -1),
                    };
                    opening += sign * PST[OPENING][pi][idx] as i32;
                    endgame += sign * PST[ENDGAME][pi][idx] as i32;
                    phase_material += phase_weight(piece);
                }
            }
        }

        (opening, endgame, phase_material)
    }

}

// Game-phase weight of a piece
#[inline]
fn phase_weight(piece: Piece) -> i32 {
    match piece {
        Piece::Knight => KNIGHT_PHASE,
        Piece::Bishop => BISHOP_PHASE,
        Piece::Rook => ROOK_PHASE,
        Piece::Queen => QUEEN_PHASE,
        _ => 0,
    }
}

/* Mobility eval scores are from https://github.com/gabtar/aconcagua

MIT License

Copyright (c) 2023 Gabriel

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
const MOB_OPEN_KNIGHT: [Eval; 9] = [-142, -37, -11, 0, 12, 15, 28, 39, 53];
const MOB_ENDG_KNIGHT: [Eval; 9] = [-78, -21, 9, 33, 44, 57, 58, 61, 54];
const MOB_OPEN_BISHOP: [Eval; 14] = [-51, -63, -29, -19, -7, 1, 7, 12, 13, 18, 21, 37, 43, 55];
const MOB_ENDG_BISHOP: [Eval; 14] = [-135, -56, -4, 21, 32, 38, 48, 53, 58, 58, 59, 49, 50, 37];
const MOB_OPEN_ROOK: [Eval; 15] = [-41, -31, -8, -1, 3, 4, 5, 6, 9, 13, 16, 18, 21, 25, 29];
const MOB_ENDG_ROOK: [Eval; 15] = [-17, 6, 30, 51, 63, 73, 80, 86, 88, 90, 93, 96, 97, 95, 91];
const MOB_OPEN_QUEEN: [Eval; 28] = [-21, -18, -34, -52, -42, -26, -23, -21, -18, -16, -13, -9, -6, -1, 0, 0, 1, 0, 0, 2, 10, 23, 39, 54, 40, 75, 30, 16];
const MOB_ENDG_QUEEN: [Eval; 28] = [-77, -66, -56, -74, 8, 71, 114, 138, 157, 179, 185, 190, 196, 194, 196, 199, 198, 201, 202, 197, 193, 171, 161, 141, 146, 127, 149, 147];

// Knight mobility eval
pub fn knight_mobility(p: &Position, c: Color, phase: i32, pawn_attacks: BitBoard) -> Eval {
    let knights = p.get_piece(Piece::Knight, c);
    let friendly = p.get_all_pieces_of_color(c);

    let mut score = 0;
    for knight in knights {
        let moves = KNIGHT_MOVES[knight.idx()] & !friendly;
        let safe_moves = (moves & !pawn_attacks).0.count_ones() as usize;
        score += interpolate_phase(
            phase, 
            MOB_OPEN_KNIGHT[safe_moves],
            MOB_ENDG_KNIGHT[safe_moves]
        );
    }

    score
}

// Bishop mobility eval
pub fn bishop_mobility(p: &Position,  c: Color, phase: i32, pawn_attacks: BitBoard) -> Eval {
    let bishops = p.get_piece(Piece::Bishop, c);
    let friendly = p.get_all_pieces_of_color(c);
    let enemy = p.get_all_pieces_of_color(c.flip());
    let occupancy = friendly | enemy;

    let mut score: Eval = 0;
    for bishop in bishops {
        let moves = get_bishop_moves(bishop, occupancy) & !friendly;
        let safe_moves = (moves & !pawn_attacks).0.count_ones() as usize;
        score += interpolate_phase(
            phase, 
            MOB_OPEN_BISHOP[safe_moves],
            MOB_ENDG_BISHOP[safe_moves]
        );
    }

    score
}

// Rook mobility eval
pub fn rook_mobility(p: &Position,  c: Color, phase: i32, pawn_attacks: BitBoard) -> Eval {
    let rooks = p.get_piece(Piece::Rook, c);
    let friendly = p.get_all_pieces_of_color(c);
    let enemy = p.get_all_pieces_of_color(c.flip());
    let occupancy = friendly | enemy;

    let mut score: Eval = 0;
    for rook in rooks {
        let moves = get_rook_moves(rook, occupancy) & !friendly;
        let safe_moves = (moves & !pawn_attacks).0.count_ones() as usize;
        score += interpolate_phase(
            phase, 
            MOB_OPEN_ROOK[safe_moves],
            MOB_ENDG_ROOK[safe_moves]
        );
    }

    score
}

// Queen mobility eval
pub fn queens_mobility(p: &Position,  c: Color, phase: i32, pawn_attacks: BitBoard) -> Eval {
    let queens = p.get_piece(Piece::Queen, c);
    let friendly = p.get_all_pieces_of_color(c);
    let enemy = p.get_all_pieces_of_color(c.flip());
    let occupancy = friendly | enemy;

    let mut score: Eval = 0;
    for queen in queens {
        let moves = (get_rook_moves(queen, occupancy) | get_bishop_moves(queen, occupancy)) & !friendly;
        let safe_moves = (moves & !pawn_attacks).0.count_ones() as usize;
        score += interpolate_phase(
            phase, 
            MOB_OPEN_QUEEN[safe_moves],
            MOB_ENDG_QUEEN[safe_moves]
        );
    }

    score
}

// Calculate a mobility bonus for the position
fn mobility_eval(p: &Position, phase: i32) -> Eval {
    let mut score: Eval = 0;

    // Do mobility score for white
    let black_pawn_attacks = p.pawn_control_bitboard(Color::Black);
    score += knight_mobility(p, Color::White, phase, black_pawn_attacks);
    score += bishop_mobility(p, Color::White, phase, black_pawn_attacks);
    score += rook_mobility(p, Color::White, phase, black_pawn_attacks);
    score += queens_mobility(p, Color::White, phase, black_pawn_attacks);

    // Do mobility score for black
    let white_pawn_attacks = p.pawn_control_bitboard(Color::White);
    score -= knight_mobility(p, Color::Black, phase, white_pawn_attacks);
    score -= bishop_mobility(p, Color::Black, phase, white_pawn_attacks);
    score -= rook_mobility(p, Color::Black, phase, white_pawn_attacks);
    score -= queens_mobility(p, Color::Black, phase, white_pawn_attacks);

    score
}

// Give a slight bonus for tempo, slight better later in game
fn tempo_eval(p: &Position, phase: i32) -> Eval {
    if p.turn == Color::White { 
        interpolate_phase(phase, 9, 17)
    } else {
        -interpolate_phase(phase, 9, 17)
    }
}

// Evaluate king safety: reward friendly pawns/pieces shielding each king and punish enemy
// pawns/pieces crowding it. Every term fades to zero in the endgame, where king safety matters less.
fn king_safety_eval(p: &Position, phase: i32) -> Eval {
    let mut eval: Eval = 0;

    // Phase scores: friendly shelter is worth more than enemy pressure per pawn, and enemy
    // pieces next to the king are as dangerous as enemy pawns.
    let friendly_pawn_score = interpolate_phase(phase, 19, 5);
    let enemy_pawn_score = interpolate_phase(phase, 17, 0);
    let friendly_piece_score = interpolate_phase(phase, 3, 0);
    let enemy_piece_score = interpolate_phase(phase, 22, 0);

    let white_occ = p.color[Color::White.idx()];
    let black_occ = p.color[Color::Black.idx()];
    let white_pawns = p.get_piece(Piece::Pawn, Color::White);
    let black_pawns = p.get_piece(Piece::Pawn, Color::Black);

    // White king: friendly = white, enemy = black. Count pawns/pieces in the ring around the king.
    let safety_mask = KING_SAFETY_MASK[p.get_piece(Piece::King, Color::White).lsb_as_square().idx()];
    let friendly_pawns = white_pawns & safety_mask;
    let enemy_pawns = black_pawns & safety_mask;
    let friendly_pieces = (white_occ & !friendly_pawns) & safety_mask;
    let enemy_pieces = (black_occ & !enemy_pawns) & safety_mask;
    eval += friendly_pawns.0.count_ones() as Eval * friendly_pawn_score;
    eval += friendly_pieces.0.count_ones() as Eval * friendly_piece_score;
    eval -= enemy_pawns.0.count_ones() as Eval * enemy_pawn_score;
    eval -= enemy_pieces.0.count_ones() as Eval * enemy_piece_score;

    // Black king: friendly = black, enemy = white. Signs are flipped so + still favours White.
    let safety_mask = KING_SAFETY_MASK[p.get_piece(Piece::King, Color::Black).lsb_as_square().idx()];
    let friendly_pawns = black_pawns & safety_mask;
    let enemy_pawns = white_pawns & safety_mask;
    let friendly_pieces = (black_occ & !friendly_pawns) & safety_mask;
    let enemy_pieces = (white_occ & !enemy_pawns) & safety_mask;
    eval -= friendly_pawns.0.count_ones() as Eval * friendly_pawn_score;
    eval -= friendly_pieces.0.count_ones() as Eval * friendly_piece_score;
    eval += enemy_pawns.0.count_ones() as Eval * enemy_pawn_score;
    eval += enemy_pieces.0.count_ones() as Eval * enemy_piece_score;

    eval
}

// Evaluate open-file control: reward rooks/queens on open and semi-open files, penalise a king
// sitting on one. "Your semi" = open on the mover's side, "opp semi" = open on the opponent's.
fn file_based_eval(p: &Position, phase: i32) -> Eval {
    let mut eval: Eval = 0;

    let bonus_for_rook_your_semi = interpolate_phase(phase, 11, 23);
    let bonus_for_rook_opp_semi = interpolate_phase(phase, 7, 16);
    let bonus_for_rook_full = interpolate_phase(phase, 19, 26);
    let bonus_for_queen_your_semi = interpolate_phase(phase, 7, 14);
    let bonus_for_queen_opp_semi = interpolate_phase(phase, 4, 10);
    let bonus_for_queen_full = interpolate_phase(phase, 9, 17);
    let penalty_for_king_your_semi = interpolate_phase(phase, 19, 0);
    let penalty_for_king_opp_semi = interpolate_phase(phase, 14, 0);
    let penalty_for_king_full = interpolate_phase(phase, 26, 0);

    let white_pawns = p.get_piece(Piece::Pawn, Color::White);
    let black_pawns = p.get_piece(Piece::Pawn, Color::Black);
    let white_rooks = p.get_piece(Piece::Rook, Color::White);
    let black_rooks = p.get_piece(Piece::Rook, Color::Black);
    let white_queens = p.get_piece(Piece::Queen, Color::White);
    let black_queens = p.get_piece(Piece::Queen, Color::Black);
    let white_king = p.get_piece(Piece::King, Color::White);
    let black_king = p.get_piece(Piece::King, Color::Black);

    for file in 0..8 {
        let mask = FILE_MASK[file];
        let has_white_pawn = (white_pawns & mask) != BitBoard(0);
        let has_black_pawn = (black_pawns & mask) != BitBoard(0);
        let white_rook_count = (white_rooks & mask).0.count_ones() as Eval;
        let black_rook_count = (black_rooks & mask).0.count_ones() as Eval;
        let white_queen_count = (white_queens & mask).0.count_ones() as Eval;
        let black_queen_count = (black_queens & mask).0.count_ones() as Eval;
        let has_white_king = (white_king & mask) != BitBoard(0);
        let has_black_king = (black_king & mask) != BitBoard(0);

        let full_open = !has_white_pawn && !has_black_pawn;
        let white_semi_open = !has_white_pawn && has_black_pawn;
        let black_semi_open = !has_black_pawn && has_white_pawn;

        // Fully open file
        if full_open {
            eval += white_rook_count * bonus_for_rook_full;
            eval -= black_rook_count * bonus_for_rook_full;
            eval += white_queen_count * bonus_for_queen_full;
            eval -= black_queen_count * bonus_for_queen_full;

            if has_white_king {
                eval -= penalty_for_king_full;
            }
            if has_black_king {
                eval += penalty_for_king_full;
            }
        }

        // Semi-open for White
        if white_semi_open {
            eval += white_rook_count * bonus_for_rook_your_semi;
            eval -= black_rook_count * bonus_for_rook_opp_semi;
            eval += white_queen_count * bonus_for_queen_your_semi;
            eval -= black_queen_count * bonus_for_queen_opp_semi;

            if has_white_king {
                eval -= penalty_for_king_your_semi;
            }
            if has_black_king {
                eval += penalty_for_king_opp_semi;
            }
        }

        // Semi-open for Black
        if black_semi_open {
            eval += white_rook_count * bonus_for_rook_opp_semi;
            eval -= black_rook_count * bonus_for_rook_your_semi;
            eval += white_queen_count * bonus_for_queen_opp_semi;
            eval -= black_queen_count * bonus_for_queen_your_semi;

            if has_white_king {
                eval -= penalty_for_king_opp_semi;
            }
            if has_black_king {
                eval += penalty_for_king_your_semi;
            }
        }
    }

    eval
}

// Get an evaluation of the minor piece match ups, based on the phase of game
fn bishop_pair_eval(p: &Position, phase: i32) -> Eval {
    let count_white_bishops = p.get_piece(Piece::Bishop, Color::White).0.count_ones();
    let count_black_bishops = p.get_piece(Piece::Bishop, Color::Black).0.count_ones();
    let bishop_pair_bonus = interpolate_phase(phase, 24, 42);

    let mut eval = if count_white_bishops > 1 { bishop_pair_bonus } else { 0 };
    eval -= if count_black_bishops > 1 { bishop_pair_bonus } else { 0 };
    eval
}

// Get a PST evaluation of the position, based on phase of game
fn pst_eval(p: &Position, phase: i32) -> Eval {
    ((p.pst_opening * (256 - phase) + p.pst_endgame * phase) / 256) as Eval
}

// Game phase in [0, 256], where 0 is the opening and 256 is the endgame
fn get_phase_score(p: &Position) -> i32 {
    let phase = TOTAL_PHASE - p.phase_material;

    // Normalize to range [0, 256] (0 = opening, 256 = endgame)
    (phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE
}

// Blend opening and endgame scores based on the phase of the game
fn interpolate_phase(phase_score: i32, opening: Eval, endgame: Eval) -> Eval {
    (((opening as i32) * (256 - phase_score) + (endgame as i32) * phase_score) / 256) as Eval
}

// Score doubled, isolated, and passed pawns from White's perspective (+ favours White).
fn pawn_structure_eval(p: &Position, phase: i32) -> Eval {
    let mut eval: Eval = 0;

    let white_pawns = p.get_piece(Piece::Pawn, Color::White);
    let black_pawns = p.get_piece(Piece::Pawn, Color::Black);

    // Doubled pawns - penalty that increases in the endgame
    let doubled_penalty = interpolate_phase(phase, 13, 35);
    eval -= doubled_pawns(white_pawns) as Eval * doubled_penalty;
    eval += doubled_pawns(black_pawns) as Eval * doubled_penalty;

    // Isolated pawns - penalty that increases in the endgame
    let isolated_penalty = interpolate_phase(phase, 16, 30);
    eval -= isolated_pawns(white_pawns).0.count_ones() as Eval * isolated_penalty;
    eval += isolated_pawns(black_pawns).0.count_ones() as Eval * isolated_penalty;

    // Passed pawns - bonus that increases in the endgame and scales with how far the pawn
    // has advanced (closer to promotion is worth more).
    let passed_bonus = interpolate_phase(phase, 19, 61);
    eval += passed_pawns_white(white_pawns, black_pawns, passed_bonus);
    eval -= passed_pawns_black(black_pawns, white_pawns, passed_bonus);

    eval
}

// Per-rank multiplier (in percent of the base bonus) for a passed pawn
const PASSED_RANK_SCALE: [i32; 8] = [0, 100, 105, 125, 155, 215, 295, 0];

// Sum the passed-pawn bonus for white pawns that no black pawn can stop from promoting,
// scaling each by its advancement toward promotion.
fn passed_pawns_white(white_pawns: BitBoard, black_pawns: BitBoard, base_bonus: Eval) -> Eval {
    let mut score: Eval = 0;
    for sq in white_pawns {
        if WHITE_PASSED_MASK[sq.idx()] & black_pawns == BitBoard(0) {
            let rank = sq.idx() / 8;
            score += (base_bonus as i32 * PASSED_RANK_SCALE[rank] / 100) as Eval;
        }
    }
    score
}

// Sum the passed-pawn bonus for black pawns that no white pawn can stop from promoting,
// scaling each by its advancement toward promotion.
fn passed_pawns_black(black_pawns: BitBoard, white_pawns: BitBoard, base_bonus: Eval) -> Eval {
    let mut score: Eval = 0;
    for sq in black_pawns {
        if BLACK_PASSED_MASK[sq.idx()] & white_pawns == BitBoard(0) {
            let rank = 7 - sq.idx() / 8;
            score += (base_bonus as i32 * PASSED_RANK_SCALE[rank] / 100) as Eval;
        }
    }
    score
}

// Count files holding two or more of these pawns (each such file counts once).
fn doubled_pawns(pawns: BitBoard) -> u32 {
    let mut doubled = 0;
    for file in 0..8 {
        let on_file = pawns & FILE_MASK[file];
        if on_file.0.count_ones() >= 2 {
            doubled += 1;
        }
    }
    doubled
}

// Mark pawns with no friendly pawn on either adjacent file.
fn isolated_pawns(pawns: BitBoard) -> BitBoard {
    let mut isolated = BitBoard(0);
    for file in 0..8 {
        let on_file = pawns & FILE_MASK[file];

        // Friendly pawns on the adjacent files.
        let mut neighbors = BitBoard(0);
        if file > 0 {
            neighbors |= pawns & FILE_MASK[file - 1];
        }
        if file < 7 {
            neighbors |= pawns & FILE_MASK[file + 1];
        }

        // No neighbours on either side means every pawn on this file is isolated.
        if neighbors == BitBoard(0) {
            isolated |= on_file;
        }
    }
    isolated
}

const FILE_MASK: [BitBoard; 8] = {
    let mut masks = [BitBoard(0); 8];
    let mut file = 0;
    while file < 8 {
        masks[file] = BitBoard(0x0101010101010101u64 << file);
        file += 1;
    }
    masks
};

// King safety mask: the ring of up to 8 squares surrounding each square (its own square excluded).
// Direction-agnostic, so the same table serves both kings.
const KING_SAFETY_MASK: [BitBoard; 64] = build_king_safety_mask();
const fn build_king_safety_mask() -> [BitBoard; 64] {
    let mut masks = [BitBoard(0); 64];
    let mut sq = 0;
    while sq < 64 {
        let row = sq / 8;
        let col = sq % 8;
        let row_lo = if row == 0 { 0 } else { row - 1 };
        let row_hi = if row == 7 { 7 } else { row + 1 };
        let col_lo = if col == 0 { 0 } else { col - 1 };
        let col_hi = if col == 7 { 7 } else { col + 1 };

        let mut mask: u64 = 0;
        let mut r = row_lo;
        while r <= row_hi {
            let mut c = col_lo;
            while c <= col_hi {
                // Skip the king's own square; only the surrounding ring counts.
                if !(r == row && c == col) {
                    mask |= 1u64 << (r * 8 + c);
                }
                c += 1;
            }
            r += 1;
        }

        masks[sq] = BitBoard(mask);
        sq += 1;
    }
    masks
}

// Passed pawn masks look at every sq in the file ahead for the pawn
const WHITE_PASSED_MASK: [BitBoard; 64] = build_passed_mask(true);
const BLACK_PASSED_MASK: [BitBoard; 64] = build_passed_mask(false);
const fn build_passed_mask(white: bool) -> [BitBoard; 64] {
    let mut masks = [BitBoard(0); 64];
    let mut sq = 0;
    while sq < 64 {
        let row = sq / 8;
        let col = sq % 8;
        let file_lo = if col == 0 { 0 } else { col - 1 };
        let file_hi = if col == 7 { 7 } else { col + 1 };

        let mut mask: u64 = 0;
        let mut r = 0;
        while r < 8 {
            // "Ahead" is toward rank 8 for White, toward rank 1 for Black.
            let ahead = if white { r > row } else { r < row };
            if ahead {
                let mut f = file_lo;
                while f <= file_hi {
                    mask |= 1u64 << (r * 8 + f);
                    f += 1;
                }
            }
            r += 1;
        }

        masks[sq] = BitBoard(mask);
        sq += 1;
    }
    masks
}
