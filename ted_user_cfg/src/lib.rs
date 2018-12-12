extern crate ted_core;
extern crate ted_common_commands;
extern crate ted_mark;
extern crate ted_kill_ring;

use std::sync::Arc;
use ted_core::*;
use ted_common_commands::*;
use ted_mark::*;
use ted_kill_ring::*;

pub fn setup_state(state: &mut State) {
    let mut default_key_map = state.default_key_map.lock();
    default_key_map.bind(vec![kbd!(C-'a')], Arc::new(begin_of_line_command));
    default_key_map.bind(vec![kbd!(C-'b')], Arc::new(backward_char_command));
    default_key_map.bind(vec![kbd!(A-'b')], Arc::new(backward_word_command));
    default_key_map.bind(vec![kbd!(C-A-'b')], Arc::new(backward_group_command));
    default_key_map.bind(vec![kbd!(C-'d')], Arc::new(delete_forward_char_command));
    default_key_map.bind(vec![kbd!(C-'e')], Arc::new(end_of_line_command));
    default_key_map.bind(vec![kbd!(C-'f')], Arc::new(forward_char_command));
    default_key_map.bind(vec![kbd!(A-'f')], Arc::new(forward_word_command));
    default_key_map.bind(vec![kbd!(C-A-'f')], Arc::new(forward_group_command));
    default_key_map.bind(vec![kbd!(C-'g')], Arc::new(remove_mark_command));
    default_key_map.bind(vec![kbd!(C-'n')], Arc::new(forward_line_command));
    default_key_map.bind(vec![kbd!(C-'p')], Arc::new(backward_line_command));
    default_key_map.bind(vec![kbd!(C-A-'u')], Arc::new(up_group_command));
    default_key_map.bind(vec![kbd!(C-'w')], Arc::new(kill_region_command));
    default_key_map.bind(vec![kbd!(A-'w')], Arc::new(copy_region_command));
    default_key_map.bind(vec![kbd!(C-'y')], Arc::new(paste_command));
    default_key_map.bind(vec![kbd!(A-'y')], Arc::new(paste_pop_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('1')], Arc::new(close_other_windows_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('2')], Arc::new(horizontal_split_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('3')], Arc::new(vertical_split_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('0')], Arc::new(close_window_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!(C-'c')], Arc::new(close_ted_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!(C-'f')], Arc::new(open_file_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('n')], Arc::new(end_of_buffer_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('p')], Arc::new(begin_of_buffer_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('o')], Arc::new(other_window_clockwise_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!(C-'o')], Arc::new(other_window_counter_clockwise_command));
    default_key_map.bind(vec![kbd!(C-'z')], Arc::new(undo_command));
    default_key_map.bind(vec![kbd!(A-'z')], Arc::new(redo_command));
    default_key_map.bind(vec![kbd!(BACKSPACE)], Arc::new(delete_backward_char_command));
    default_key_map.bind(vec![kbd!(C-'@')], Arc::new(set_mark_command));
}
