extern crate ted_common_commands;
extern crate ted_core;
extern crate ted_git;
extern crate ted_kill_ring;
extern crate ted_mark;

use ted_common_commands::*;
use ted_core::*;
use ted_git::*;
use ted_kill_ring::*;
use ted_mark::*;

pub fn setup_state(state: &mut State) {
    let mut default_key_map = state.default_key_map.lock();
    default_key_map.bind(vec![kbd!(C-'a')], begin_of_line_command());
    default_key_map.bind(vec![kbd!(C-'b')], backward_char_command());
    default_key_map.bind(vec![kbd!(A-'b')], backward_word_command());
    default_key_map.bind(vec![kbd!(C-A-'b')], backward_group_command());
    default_key_map.bind(vec![kbd!(C-'d')], delete_forward_char_command());
    default_key_map.bind(vec![kbd!(C-'e')], end_of_line_command());
    default_key_map.bind(vec![kbd!(C-'f')], forward_char_command());
    default_key_map.bind(vec![kbd!(A-'f')], forward_word_command());
    default_key_map.bind(vec![kbd!(C-A-'f')], forward_group_command());
    default_key_map.bind(vec![kbd!(C-'g')], remove_mark_command());
    default_key_map.bind(vec![kbd!(C-'n')], forward_line_command());
    default_key_map.bind(vec![kbd!(C-'p')], backward_line_command());
    default_key_map.bind(vec![kbd!(C-A-'u')], up_group_command());
    default_key_map.bind(vec![kbd!(C-'w')], kill_region_command());
    default_key_map.bind(vec![kbd!(A-'w')], copy_region_command());
    default_key_map.bind(vec![kbd!(C-'y')], paste_command());
    default_key_map.bind(vec![kbd!(A-'y')], paste_pop_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('1')], close_other_windows_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('2')], horizontal_split_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('3')], vertical_split_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('0')], close_window_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!(C-'c')], close_ted_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!(C-'f')], open_file_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('g')], open_git_repository_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('n')], end_of_buffer_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('p')], begin_of_buffer_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('o')], other_window_clockwise_command());
    default_key_map.bind(vec![kbd!(C-'x'), kbd!(C-'o')], other_window_counter_clockwise_command());
    default_key_map.bind(vec![kbd!(C-'z')], undo_command());
    default_key_map.bind(vec![kbd!(A-'z')], redo_command());
    default_key_map.bind(vec![kbd!(BACKSPACE)], delete_backward_char_command());
    default_key_map.bind(vec![kbd!(C-'@')], set_mark_command());
}
