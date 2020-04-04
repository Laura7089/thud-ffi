//! Foreign Function Interface bindings
//!
//! The functions provided in this thud can be called as C functions from non-rust languages.
//! Hopefully, this will allow clients to be developed in other ecosystems, dependent on this
//! library but not tied to using rust.
//!
//! The bindings are in an early state at present; they simply return integers representing the
//! results of operations.
use libc::c_int;
use libc::c_uint;
use std::ptr;
use std::slice;
use thud::Coord;
use thud::Direction;
use thud::EndState;
use thud::Piece;
use thud::Player;
use thud::Thud;

fn piece_to_int(piece: Piece) -> c_uint {
    match piece {
        Piece::Empty => 0,
        Piece::Dwarf => 1,
        Piece::Troll => 2,
        Piece::Thudstone => 3,
    }
}

/// Wrapper for [`Thud::new()`](struct.Thud.html#method.new)
#[no_mangle]
pub extern "C" fn thud_new() -> *mut Thud {
    Box::into_raw(Box::new(Thud::new()))
}

/// Wrapper for [`Coord::zero_based()`](struct.Coord.html#method.zero_based).
///
/// Returns a null pointer if initialisation of the coord failed.
#[no_mangle]
pub extern "C" fn coord_new(x: c_uint, y: c_uint) -> *mut Coord {
    match Coord::zero_based(x as usize, y as usize) {
        Ok(coord) => Box::into_raw(Box::new(coord)),
        _ => ptr::null_mut(),
    }
}

/// Release a Thud from memory.
#[no_mangle]
pub unsafe extern "C" fn thud_destroy(thud_raw: *mut Thud) {
    if !thud_raw.is_null() {
        drop(Box::from_raw(thud_raw));
    }
}

/// Release a Coord from memory.
#[no_mangle]
pub unsafe extern "C" fn coord_destroy(coord_raw: *mut Coord) {
    if !coord_raw.is_null() {
        drop(Box::from_raw(coord_raw));
    }
}

/// Wrapper for [`Thud::move_piece`](struct.Thud.html#method.move_piece).
///
/// Returns:
///
/// - `0` if the move was made successfully
/// - `-1` if any pointers passed were null
/// - `-2` if the move was illegal
#[no_mangle]
pub unsafe extern "C" fn thud_move(
    thud_raw: *mut Thud,
    src_raw: *mut Coord,
    dest_raw: *mut Coord,
) -> c_int {
    if thud_raw.is_null() || src_raw.is_null() || dest_raw.is_null() {
        return -1;
    }
    let mut thud = Box::from_raw(thud_raw);
    match thud.move_piece(*src_raw, *dest_raw) {
        Ok(_) => 0,
        _ => -2,
    }
}

/// Wrapper for [`Thud::attack()`](struct.Thud.html#method.attack).
///
/// Returns:
///
/// - `0` if the move was made successfully
/// - `-1` if any pointers passed were null
/// - `-2` if the move was illegal
#[no_mangle]
pub unsafe extern "C" fn thud_attack(
    thud_raw: *mut Thud,
    src_raw: *mut Coord,
    dest_raw: *mut Coord,
) -> c_int {
    if thud_raw.is_null() || src_raw.is_null() || dest_raw.is_null() {
        return -1;
    }
    let mut thud = Box::from_raw(thud_raw);
    match thud.attack(*src_raw, *dest_raw) {
        Ok(_) => 0,
        _ => -2,
    }
}

/// Wrapper for [`Thud::turn()`](struct.Thud.html#method.turn).
///
/// Returns:
///
/// - `-1` if `thud_raw` is a null pointer
/// - `0` for a Dwarf turn
/// - `1` for a Troll turn
/// - `2` for an ended game
#[no_mangle]
pub unsafe extern "C" fn thud_get_turn(thud_raw: *mut Thud) -> c_int {
    if thud_raw.is_null() {
        return -1;
    }
    let thud = Box::from_raw(thud_raw);
    match thud.turn() {
        Some(Player::Dwarf) => 0,
        Some(Player::Troll) => 1,
        _ => 2,
    }
}

/// Wrapper for [`Thud::winner()`](struct.Thud.html#method.winner).
///
/// Returns:
///
/// - `-1` if `thud_raw` is a null pointer
/// - `0` for a Dwarf victory
/// - `1` for a Troll victory
/// - `2` for a draw
/// - `3` if the game hasn't ended yet
#[no_mangle]
pub unsafe extern "C" fn thud_get_winner(thud_raw: *mut Thud) -> c_int {
    if thud_raw.is_null() {
        return -1;
    }
    let mut thud = Box::from_raw(thud_raw);
    match thud.winner() {
        Some(EndState::Won(Player::Dwarf)) => 0,
        Some(EndState::Won(Player::Troll)) => 1,
        Some(EndState::Draw) => 2,
        _ => 3,
    }
}

/// Wrapper for [`Thud::score()`](struct.Thud.html#method.score)
///
/// Returns a 2-element array of `c_int` holding:
///
/// 1. The Dwarf score
/// 2. The Troll score
///
/// Returns a null pointer if `thud_raw` is a null pointer.
#[no_mangle]
pub unsafe extern "C" fn thud_get_score(thud_raw: *mut Thud) -> *mut c_int {
    if thud_raw.is_null() {
        return ptr::null_mut();
    }
    let thud = Box::from_raw(thud_raw);
    let (dwarf, troll) = thud.score();
    ([dwarf as c_int, troll as c_int]).as_mut_ptr()
}

/// Wrapper for [`Thud::troll_cap()`](struct.Thud.html#method.troll_cap).
///
/// Takes:
///
/// - Pointer to a `Thud`
/// - Pointer to a `Coord`
/// - 8-element array of `c_uint`; each of these should be between `0` and `8` inclusive.
///   They map to directions, with `Direction::Right` being 0, incrementing clockwise.
///
/// Returns:
///
/// - `-3` if any elements of targets_raw were invalid directions.
/// - `-2` if the move was illegal
/// - `-1` if any arguments were null pointers
/// - `0` if the move finished successfully
#[no_mangle]
pub unsafe extern "C" fn thud_troll_cap(
    thud_raw: *mut Thud,
    src_raw: *mut Coord,
    targets_raw: *mut c_uint,
) -> c_int {
    if thud_raw.is_null() || src_raw.is_null() || targets_raw.is_null() {
        return -1;
    }
    let targets = slice::from_raw_parts(targets_raw, 8);
    let mut attack_dirs = Vec::with_capacity(8);
    for i in 0..8 {
        if targets[i] == 1 {
            attack_dirs.push(match Direction::from_num(i) {
                Ok(dir) => dir,
                _ => return -3,
            });
        }
    }

    let mut thud = Box::from_raw(thud_raw);
    match thud.troll_cap(*src_raw, attack_dirs) {
        Ok(_) => 0,
        _ => -2,
    }
}

/// Wrapper for [`Thud::board()`](struct.Thud.html#method.board).
///
/// Returns a 15 by 15 nested array of `c_uint` with piece represented as:
///
/// - `0` for an empty space
/// - `1` for a Dwarf piece
/// - `2` for a Troll piece
/// - `3` for the Thundstone
#[no_mangle]
pub unsafe extern "C" fn thud_get_board(thud_raw: *mut Thud) -> *mut *mut c_uint {
    if thud_raw.is_null() {
        return ptr::null_mut();
    }
    let board = Box::from_raw(thud_raw).board().full_raw();
    let mut result = Vec::with_capacity(15);
    for x in 0..15 {
        result.push(
            (0..15)
                .map(|y| piece_to_int(board[x][y]) as c_uint)
                .collect::<Vec<c_uint>>()
                .as_mut_ptr(),
        );
    }
    result.as_mut_ptr()
}
