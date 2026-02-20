use serde::Serialize;

use crate::{
    LINE_STATES,
    chart::{self},
    states_effect, states_judge, states_statistics, states_lines,
};

pub struct LineState {
    pub enable: bool,
    pub x: f64,
    pub y: f64,
    pub rotate: f64,
    pub alpha: f64,
    pub speed: f64,
    pub line_y: f64,
    pub tick_time: f64,
    pub event_speed_index_cache: i32,
    pub event_move_index_cache: i32,
    pub event_rotate_index_cache: i32,
    pub event_alpha_index_cache: i32,
    pub notes_above_state: Vec<NoteState>,
    pub notes_below_state: Vec<NoteState>,
    pub speed_events: Vec<chart::Event1>,
    pub move_events: Vec<chart::Event4>,
    pub rotate_events: Vec<chart::Event2>,
    pub alpha_events: Vec<chart::Event2>,
    pub bpm: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoteScore {
    Perfect,
    Good,
    Bad,
    Miss,
    None,
}

pub struct NoteState {
    pub note: chart::Note,
    pub highlight: bool,
    pub score: NoteScore,
    pub hold_cool_down: f64,
    pub extra_score: NoteScore,
}

/// Metadata of the level
#[derive(Serialize)]
pub struct Metadata {

    /// The estimated length of the chart
    pub length_in_second: f64,

    /// The offset of the chart
    pub offset: f64,

    /// The format version of the chart
    pub format_version: i32,
}

impl Default for LineState {
    fn default() -> Self {
        LineState {
            enable: false,
            x: 0.0,
            y: 0.0,
            rotate: 0.0,
            alpha: 0.0,
            speed: 1.0,
            line_y: 0.0,
            tick_time: 0.0,
            event_speed_index_cache: 0,
            event_move_index_cache: 0,
            event_rotate_index_cache: 0,
            event_alpha_index_cache: 0,
            notes_above_state: vec![],
            notes_below_state: vec![],
            speed_events: vec![],
            move_events: vec![],
            alpha_events: vec![],
            rotate_events: vec![],
            bpm: 0.0,
        }
    }
}

impl Default for NoteState {
    fn default() -> Self {
        NoteState {
            highlight: false,
            score: NoteScore::None,
            hold_cool_down: 0.0,
            extra_score: NoteScore::None,
            note: chart::Note {
                note_type: chart::NoteType::Tap,
                time: 0,
                position_x: 0.0,
                hold_time: 0.25,
                speed: 0.0,
                floor_position: 0.0,
            },
        }
    }
}

pub fn get_seconds_per_tick(bpm: f64) -> f64 {
    60.0 / bpm / 32.0
}

/// Reset the state of notes that before the `before_time_in_second` to PERFECT
pub fn reset_note_state(before_time_in_second: f64) {
    LINE_STATES.with_borrow_mut(|state| {
        state.iter_mut().for_each(|line| {
            let seconds_per_tick = get_seconds_per_tick(line.bpm);
            let process_notes = |notes: &mut [NoteState]| {
                notes.iter_mut().for_each(|note| {
                    note.hold_cool_down = 0.0;
                    let note_time_in_second = note.note.time as f64 * seconds_per_tick;
                    let hold_time_in_second =
                        (note.note.time as f64 + note.note.hold_time) * seconds_per_tick;
                    if note_time_in_second >= before_time_in_second {
                        note.extra_score = NoteScore::None;
                        note.score = NoteScore::None;
                    } else if hold_time_in_second >= before_time_in_second {
                        note.score = NoteScore::None;
                    } else {
                        note.score = NoteScore::Perfect;
                        note.extra_score = NoteScore::Perfect;
                    }
                });
            };
            process_notes(&mut line.notes_above_state);
            process_notes(&mut line.notes_below_state);
        });
    });
    states_statistics::refresh_chart_statistics();
}

/// Ticking all states, including lines, judges and chart statistics
pub fn tick_all(time_in_second: f64, delta_time_in_second: f64, auto: bool) {
    states_lines::tick_lines(time_in_second);
    states_effect::tick_effect(delta_time_in_second);
    if states_judge::tick_lines_judge(delta_time_in_second, auto){
        states_statistics::refresh_chart_statistics();
    }
}