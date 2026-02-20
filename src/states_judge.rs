use crate::{
    LINE_STATES, TOUCH_STATES,
    chart::{Note, NoteType},
    input::TouchInfo,
    math::{self, Point},
    states::{LineState, NoteScore, NoteState},
    states_effect,
};

pub(crate) fn tick_lines_judge(delta_time_in_second: f64, auto: bool) -> bool {
    states_effect::clear_sound_effect();
    TOUCH_STATES.with_borrow_mut(|touches| {
        LINE_STATES.with_borrow_mut(|lines| {
            tick_line_judge(delta_time_in_second, touches.as_mut(), lines.as_mut(), auto)
        })
    })
}

fn tick_line_judge(
    delta_time_in_second: f64,
    touches: &mut [TouchInfo],
    lines: &mut [LineState],
    auto: bool,
) -> bool {
    let mut judged = false;
    for line in lines.iter_mut() {
        if !line.enable {
            continue;
        }
        let current_tick = line.tick_time;
        line.notes_above_state
            .iter_mut()
            .chain(line.notes_below_state.iter_mut())
            .for_each(|note| {
                let line_x = line.x;
                let line_y = line.y;
                let line_rotate = line.rotate;
                let bpm = line.bpm;
                let note_type = note.note.r#type;
                let local_judged = if auto {
                    match note_type {
                        NoteType::Hold => tick_hold_note_auto(
                            delta_time_in_second,
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                        _ => tick_normal_note_auto(
                            current_tick,
                            note,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                    }
                } else {
                    match note_type {
                        NoteType::Tap => tick_tap_note(
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                        NoteType::Drag => tick_drag_note(
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                        NoteType::Hold => tick_hold_note(
                            delta_time_in_second,
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                        NoteType::Flick => tick_flick_note(
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                    }
                };
                judged |= local_judged;
            });
    }
    for touch in touches.iter_mut() {
        if touch.enable {
            touch.touch_valid = false;
        }
    }
    judged
}

fn check_point_in_judge_range(
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    Note {
        position_x: note_position_x,
        ..
    }: &Note,
    TouchInfo {
        x: touch_x,
        y: touch_y,
        ..
    }: &TouchInfo,
) -> (bool, (f64, f64)) {
    let Point {
        x: root_x,
        y: root_y,
    } = math::get_pos_out_of_line(
        line_x,
        line_y,
        line_rotate,
        *note_position_x * math::UNIT_WIDTH,
    );
    let Point {
        x: touch_root_x,
        y: touch_root_y,
    } = math::get_pos_point_vertical_in_line(
        line_x,
        line_y,
        line_rotate,
        f64::from(*touch_x),
        f64::from(*touch_y),
    );
    (
        math::is_point_in_judge_range(
            root_x,
            root_y,
            math::fix_degree(line_rotate),
            touch_root_x,
            touch_root_y,
            300.0,
        ),
        (root_x, root_y),
    )
}

fn check_judge_result(current_tick: f64, note: &NoteState, bpm: f64) -> (f64, NoteScore) {
    let seconds_per_tick = 60.0 / bpm / 32.0;
    let perfect_range_in_tick = 0.08 / seconds_per_tick;
    let good_range_in_tick = 0.16 / seconds_per_tick;
    let bad_range_in_tick = 0.18 / seconds_per_tick;
    let time_delta = current_tick - f64::from(note.note.time);
    (
        time_delta,
        match time_delta.abs() {
            x if 0.0 <= x && x <= perfect_range_in_tick => NoteScore::Perfect,
            x if perfect_range_in_tick < x && x <= good_range_in_tick => NoteScore::Good,
            x if good_range_in_tick < x && x <= bad_range_in_tick => NoteScore::Bad,
            _ => NoteScore::Miss,
        },
    )
}

fn create_splash(seed: f64, x: f64, y: f64, note_score: NoteScore) {
    match note_score {
        NoteScore::Perfect => states_effect::new_click_effect(seed, x, y, 0),
        NoteScore::Good => states_effect::new_click_effect(seed, x, y, 1),
        _ => {}
    }
}

fn tick_normal_note_auto(
    current_tick: f64,
    note: &mut NoteState,
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) -> bool {
    if note.score != NoteScore::None {
        return false;
    }
    let (judge_delta, _) = check_judge_result(current_tick, note, bpm);
    if judge_delta >= 0.0 {
        let Point {
            x: root_x,
            y: root_y,
        } = math::get_pos_out_of_line(
            line_x,
            line_y,
            line_rotate,
            note.note.position_x * math::UNIT_WIDTH,
        );
        note.score = NoteScore::Perfect;
        create_splash(current_tick, root_x, root_y, NoteScore::Perfect);
        states_effect::new_sound_effect(note.note.r#type);
        return true;
    }
    false
}

fn tick_flick_note(
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) -> bool {
    if note.score != NoteScore::None {
        return false;
    }
    let (judge_delta, judge_result) = check_judge_result(current_tick, note, bpm);
    if judge_delta < 0.0 && judge_result == NoteScore::Miss {
        return false;
    }
    if note.extra_score != NoteScore::None {
        if judge_delta > 0.0 {
            let Point {
                x: root_x,
                y: root_y,
            } = math::get_pos_out_of_line(
                line_x,
                line_y,
                line_rotate,
                note.note.position_x * math::UNIT_WIDTH,
            );
            note.score = NoteScore::Perfect;
            create_splash(current_tick, root_x, root_y, NoteScore::Perfect);
            states_effect::new_sound_effect(NoteType::Flick);
            return true;
        }
        return false;
    }
    if judge_delta > 0.0 && judge_result == NoteScore::Miss {
        note.score = NoteScore::Miss;
        return true;
    }
    for touch in touches {
        if !touch.enable {
            continue;
        }
        let (is_in_judge_range, _) =
            check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
        if is_in_judge_range && touch.length() >= 50.0 {
            note.extra_score = NoteScore::Perfect;
            touch.reset_length();
            return false;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn tick_hold_note_auto(
    delta_time_in_second: f64,
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) -> bool {
    if note.score != NoteScore::None {
        return false;
    }
    let (judge_delta, _) = check_judge_result(current_tick, note, bpm);
    if judge_delta >= 0.0 && note.extra_score != NoteScore::Perfect {
        note.extra_score = NoteScore::Perfect;
        states_effect::new_sound_effect(NoteType::Hold);
    }
    tick_hold_note_common(
        delta_time_in_second,
        current_tick,
        note,
        touches,
        line_x,
        line_y,
        line_rotate,
        bpm,
        true,
    )
    .1
}

#[allow(clippy::too_many_arguments)]
fn tick_hold_note_common(
    delta_time_in_second: f64,
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
    auto: bool,
) -> (bool, bool) {
    if note.extra_score != NoteScore::None {
        let seconds_per_tick = 60.0 / bpm / 32.0;
        let delta_tick = delta_time_in_second / seconds_per_tick;
        let mut judged = false;
        note.hold_cool_down -= delta_tick;
        if note.hold_cool_down <= 0.0 {
            let Point {
                x: root_x,
                y: root_y,
            } = math::get_pos_out_of_line(
                line_x,
                line_y,
                line_rotate,
                note.note.position_x * math::UNIT_WIDTH,
            );
            if auto
                || touches.iter().any(|touch| {
                    let (is_in_judge_range, _) =
                        check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
                    is_in_judge_range && touch.enable
                })
            {
                note.hold_cool_down = if note.hold_cool_down < -16.0 {
                    0.0
                } else {
                    note.hold_cool_down + 16.0
                };
                create_splash(current_tick, root_x, root_y, note.extra_score);
            } else {
                note.score = NoteScore::Miss;
                judged = true;
            }
        }
        if note.note.hold_time + f64::from(note.note.time) <= current_tick {
            note.score = note.extra_score;
            judged = true;
        }
        return (true, judged);
    }
    (false, false)
}

#[allow(clippy::too_many_arguments)]
fn tick_hold_note(
    delta_time_in_second: f64,
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) -> bool {
    if note.score != NoteScore::None {
        return false;
    }
    let (hold, hold_judged) = tick_hold_note_common(
        delta_time_in_second,
        current_tick,
        note,
        touches,
        line_x,
        line_y,
        line_rotate,
        bpm,
        false,
    );
    if hold {
        return hold_judged;
    }
    let (judge_delta, judge_result) = check_judge_result(current_tick, note, bpm);
    if judge_delta < 0.0 && judge_result == NoteScore::Miss {
        return false;
    }
    if judge_delta > 0.0 && judge_result == NoteScore::Miss {
        note.score = NoteScore::Miss;
        return true;
    }
    for touch in touches {
        if !touch.enable {
            continue;
        }
        let (is_in_judge_range, _) =
            check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
        if is_in_judge_range && touch.touch_valid {
            if judge_result != NoteScore::Perfect && judge_result != NoteScore::Good {
                continue;
            }
            touch.touch_valid = false;
            note.extra_score = judge_result;
            states_effect::new_sound_effect(NoteType::Hold);
            return false;
        }
    }
    false
}

fn tick_drag_note(
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) -> bool {
    if note.score != NoteScore::None {
        return false;
    }
    let (judge_delta, judge_result) = check_judge_result(current_tick, note, bpm);
    if judge_delta < 0.0 && judge_result == NoteScore::Miss {
        return false;
    }
    if note.extra_score != NoteScore::None {
        if judge_delta > 0.0 {
            let Point {
                x: root_x,
                y: root_y,
            } = math::get_pos_out_of_line(
                line_x,
                line_y,
                line_rotate,
                note.note.position_x * math::UNIT_WIDTH,
            );
            note.score = NoteScore::Perfect;
            states_effect::new_sound_effect(NoteType::Drag);
            create_splash(current_tick, root_x, root_y, NoteScore::Perfect);
            return true;
        }
        return false;
    }
    if judge_delta > 0.0 && judge_result == NoteScore::Miss {
        note.score = NoteScore::Miss;
        return true;
    }
    for touch in touches {
        if !touch.enable {
            continue;
        }
        let (is_in_judge_range, _) =
            check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
        if is_in_judge_range {
            note.extra_score = NoteScore::Perfect;
            return false;
        }
    }
    false
}

fn tick_tap_note(
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) -> bool {
    if note.score != NoteScore::None {
        return false;
    }
    let (judge_delta, judge_result) = check_judge_result(current_tick, note, bpm);
    if judge_delta < 0.0 && judge_result == NoteScore::Miss {
        return false;
    }
    //+ late
    if judge_delta > 0.0 && judge_result == NoteScore::Miss {
        note.score = NoteScore::Miss;
        return true;
    }
    for touch in touches {
        if !touch.enable {
            continue;
        }
        let (is_in_judge_range, (root_x, root_y)) =
            check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
        if is_in_judge_range && touch.touch_valid {
            touch.touch_valid = false;
            note.score = judge_result;
            states_effect::new_sound_effect(NoteType::Tap);
            create_splash(current_tick, root_x, root_y, judge_result);
            return true;
        }
    }
    false
}
