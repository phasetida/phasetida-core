use std::{collections::HashSet, default::Default};

use crate::{
    CHART_STATISTICS, FLATTEN_NOTE_INDEX, HIT_EFFECT_POOL, LINE_STATES, SOUND_POOL,
    SPLASH_EFFECT_POOL, TOUCH_STATES,
    chart::{self, ChartRaw, JudgeLine, WithTimeRange},
    input::TouchInfo,
    states::{LineState, Metadata, NoteState, get_seconds_per_tick},
    states_effect::{HitEffect, SoundEffect, SplashEffect},
    states_statistics::{self, ChartStatistics},
};

/// Initialize state of lines from raw json.
///
/// # Errors
///
/// This function will return an error if the deserialization failed.
pub fn init_line_states_from_json(json: &str) -> Result<Metadata, serde_json::Error> {
    let chart_raw = serde_json::from_str::<ChartRaw>(json)?;
    Ok(init_line_states(chart_raw))
}

/// Initialize state of lines from standard V3 chart
#[must_use]
pub fn init_line_states(chart_raw: chart::ChartRaw) -> Metadata {
    let format_version = match chart_raw {
        ChartRaw::V1(_) => 1,
        ChartRaw::V3(_) => 3,
    };
    let mut chart = chart_raw.convert_to_v3();
    chart.judge_line_list = chart
        .judge_line_list
        .into_iter()
        .map(|mut line| {
            line.notes_above.sort_by(|a, b| (a.time).cmp(&b.time));
            line.notes_below.sort_by(|a, b| (a.time).cmp(&b.time));
            line
        })
        .collect::<Vec<_>>();
    let metadata = LINE_STATES.with_borrow_mut(|states| {
        *states = std::array::from_fn(|_| LineState::default());
        let available_len = chart.judge_line_list.len();
        for (i, it) in chart.judge_line_list.into_iter().enumerate() {
            let JudgeLine {
                bpm,
                notes_above,
                notes_below,
                speed_events,
                move_events,
                rotate_events,
                alpha_events,
            } = it;
            states[i] = LineState {
                enable: true,
                bpm,
                move_events,
                alpha_events,
                speed_events,
                rotate_events,
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
                ..Default::default()
            }
        }
        states
            .iter_mut()
            .skip(available_len)
            .for_each(|it| it.enable = false);
        process_highlight(states.as_mut());
        Metadata {
            length_in_second: get_estimated_length(states),
            offset: chart.offset,
            format_version,
        }
    });
    states_statistics::init_flatten_line_state();
    metadata
}

/// Clear the states of lines
pub fn clear_states() {
    FLATTEN_NOTE_INDEX.with_borrow_mut(std::vec::Vec::clear);
    LINE_STATES.with_borrow_mut(|it| *it = std::array::from_fn(|_| LineState::default()));
    TOUCH_STATES.with_borrow_mut(|it| *it = std::array::from_fn(|_| TouchInfo::default()));
    HIT_EFFECT_POOL.with_borrow_mut(|it| *it = std::array::from_fn(|_| HitEffect::default()));
    SPLASH_EFFECT_POOL.with_borrow_mut(|it| *it = std::array::from_fn(|_| SplashEffect::default()));
    CHART_STATISTICS.with_borrow_mut(|it| *it = ChartStatistics::default());
    SOUND_POOL.with_borrow_mut(|it| *it = SoundEffect::default());
}

fn process_highlight(judge_line_states: &mut [LineState]) {
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

fn get_estimated_length(state: &[LineState]) -> f64 {
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
