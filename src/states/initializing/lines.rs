use std::{collections::HashSet, default::Default};

use crate::{
    LINE_STATES,
    chart::{self, Event1, JudgeLine, WithTimeRange},
    states::{
        self, LineData, LineState, Metadata, NoteState, get_seconds_per_tick,
        lines::get_line_y_direct,
    },
};

/// Initialize state of lines from standard V3 chart
#[must_use]
pub fn init_line_states(chart: &mut chart::Chart) -> Metadata {
    for line in &mut chart.judge_line_list {
        line.notes_above.sort_by_key(|a| a.time);
        line.notes_below.sort_by_key(|a| a.time);
    }
    let metadata = LINE_STATES.with_borrow_mut(|states| {
        states.clear();
        let available_len = chart.judge_line_list.len();
        for it in &chart.judge_line_list {
            let JudgeLine {
                bpm,
                notes_above,
                notes_below,
                speed_events,
                move_events,
                rotate_events,
                alpha_events,
            } = it;
            states.push(LineData {
                line_state: LineState {
                    enable: true,
                    bpm: *bpm,
                    ..Default::default()
                },
                move_events: move_events.clone(),
                alpha_events: alpha_events.clone(),
                speed_events: speed_events.clone(),
                rotate_events: rotate_events.clone(),
                notes_above_state: notes_above
                    .clone()
                    .into_iter()
                    .map(|it| NoteState {
                        note: it,
                        ..Default::default()
                    })
                    .collect(),
                notes_below_state: notes_below
                    .clone()
                    .into_iter()
                    .map(|it| NoteState {
                        note: it,
                        ..Default::default()
                    })
                    .collect(),
            });
        }
        states
            .iter_mut()
            .skip(available_len)
            .for_each(|it| it.enable = false);
        process_highlight(states.as_mut());
        Metadata {
            length_in_second: get_estimated_length(states),
            offset: chart.offset,
            lines: states.len() as u32,
            format_version: 0,
            block_areas: 0,
        }
    });
    states::statistics::init_flatten_line_state();
    metadata
}

/// Ignore present value, then re calculate [`floor_position`]
pub fn recalculate_floor_position() {
    fn process_half(states: &mut [NoteState], bpm: f64, speed_event: &[Event1]) {
        for note_state in states {
            let note = &mut note_state.note;
            let line_y = get_line_y_direct(note.time.into(), bpm, speed_event);
            note.floor_position = line_y;
        }
    }
    LINE_STATES.with_borrow_mut(|state| {
        for line in state {
            let bpm = line.bpm;
            process_half(&mut line.notes_below_state, bpm, &line.speed_events);
            process_half(&mut line.notes_above_state, bpm, &line.speed_events);
        }
    });
}

fn process_highlight(judge_line_states: &mut [LineData]) {
    let mut set1 = HashSet::<i32>::new();
    let mut set2 = HashSet::<i32>::new();
    for it in judge_line_states.iter() {
        if !it.enable {
            continue;
        }
        let seconds_per_tick = get_seconds_per_tick(it.bpm);
        let mut process = |notes: &Vec<NoteState>| {
            for n in notes {
                let tick_time = n.note.time;
                let second_time = ((seconds_per_tick * 32768.0) as i32) * tick_time;
                if set1.contains(&second_time) {
                    set2.insert(second_time);
                } else {
                    set1.insert(second_time);
                }
            }
        };
        process(&it.notes_above_state);
        process(&it.notes_below_state);
    }
    for it in judge_line_states.iter_mut() {
        if !it.enable {
            continue;
        }
        let seconds_per_tick = get_seconds_per_tick(it.bpm);
        let process = |notes: &mut Vec<NoteState>| {
            for n in notes.iter_mut() {
                let tick_time = n.note.time;
                let second_time = ((seconds_per_tick * 32768.0) as i32) * tick_time;
                if set2.contains(&second_time) {
                    n.highlight = true;
                }
            }
        };
        process(&mut it.notes_above_state);
        process(&mut it.notes_below_state);
    }
}

fn get_estimated_length(state: &[LineData]) -> f64 {
    let note_max_time = state.iter().fold(0.0, |last, it| {
        let seconds_per_tick = get_seconds_per_tick(it.bpm);
        let get_time = |note: &NoteState| -> f64 {
            (f64::from(note.note.time) + note.note.hold_time) * seconds_per_tick
        };
        [
            it.notes_above_state.last().map_or(0.0, get_time),
            it.notes_below_state.last().map_or(0.0, get_time),
        ]
        .iter()
        .fold(last, |l, i| i.max(l))
    });
    let event_max_time = state.iter().fold(0.0, |last, it| {
        fn event_folder(seconds_per_tick: f64, events: &[impl WithTimeRange]) -> f64 {
            events
                .iter()
                .fold(0.0, |last, it| last.max(it.time_start() * seconds_per_tick))
        }
        let seconds_per_tick = get_seconds_per_tick(it.bpm);
        [
            event_folder(seconds_per_tick, &it.move_events),
            event_folder(seconds_per_tick, &it.alpha_events),
            event_folder(seconds_per_tick, &it.speed_events),
            event_folder(seconds_per_tick, &it.rotate_events),
        ]
        .iter()
        .fold(last, |l, i| i.max(l))
    });
    note_max_time.max(event_max_time)
}
