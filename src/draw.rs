use crate::chart::{Note, NoteType};
use crate::math::{self, Point, Rect};
use crate::renders::{
    self, Dense, RendClickEffect, RendNote, RendPoint, RendSound, RendSplashEffect, RendStatistics,
};
use crate::states::{LineData, LineState, NoteScore, NoteState};
use crate::states_effect::{HitEffect, SoundEffect, SplashEffect};
use crate::{
    CHART_STATISTICS, DRAW_IMAGE_OFFSET, HIT_EFFECT_POOL, LINE_STATES, SOUND_POOL,
    SPLASH_EFFECT_POOL, TOUCH_STATES, WORLD_RECT,
};

#[allow(clippy::struct_field_names)]
pub struct DrawImageOffset {
    pub hold_head_height: f64,
    pub hold_head_highlight_height: f64,
    pub hold_end_height: f64,
    pub hold_end_highlight_height: f64,
}

/// A trait for observing write operations on a buffer.
///
/// The caller needs to implement this trait to listen for write events on the
/// buffer, which typically contains a cursor.
pub trait BufferWithCursor {
    /// Called when `slice` is written to the buffer.
    fn write(&mut self, slice: &[u8]);
}

impl Default for DrawImageOffset {
    fn default() -> Self {
        Self {
            hold_head_height: 0.0,
            hold_head_highlight_height: 0.0,
            hold_end_height: 0.0,
            hold_end_highlight_height: 0.0,
        }
    }
}

/// Preloads image section heights.
///
/// Takes heights for hold head and hold end sections in both normal and highlighted states.
pub fn load_image_offset(
    hold_head_height: f64,
    hold_head_highlight_height: f64,
    hold_end_height: f64,
    hold_end_highlight_height: f64,
) {
    DRAW_IMAGE_OFFSET.with_borrow_mut(|offset| {
        *offset = DrawImageOffset {
            hold_head_height,
            hold_head_highlight_height,
            hold_end_height,
            hold_end_highlight_height,
        };
    });
}

/// Render and writes the internal state to the buffer.
///
/// The state is written by calling `write` on the provided `BufferWithCursor`.
pub fn process_state_to_drawable(wrapped_buffer: &mut impl BufferWithCursor) {
    WORLD_RECT.with_borrow(|world_rect| {
        CHART_STATISTICS.with_borrow(|statistics| {
            wrapped_buffer.write(
                RendStatistics {
                    rend_type: 5,
                    combo: statistics.combo,
                    max_combo: statistics.max_combo,
                    score: statistics.score as f32,
                    accurate: statistics.accurate as f32,
                }
                .to_bytes(),
            );
        });
        LINE_STATES.with_borrow(|states| {
            DRAW_IMAGE_OFFSET.with_borrow(|offset| {
                for it in states {
                    write_line(wrapped_buffer, it, world_rect);
                }
                write_notes(wrapped_buffer, states.as_ref(), offset, world_rect);
            });
        });
        HIT_EFFECT_POOL.with_borrow(|effects| {
            write_click_effects(wrapped_buffer, effects);
        });
        SPLASH_EFFECT_POOL.with_borrow(|effects| {
            write_splash_effects(wrapped_buffer, effects);
        });
        SOUND_POOL.with_borrow(|effects| write_sound_effects(wrapped_buffer, effects));
        TOUCH_STATES.with_borrow(|touches| {
            for it in touches {
                if !it.enable {
                    continue;
                }
                wrapped_buffer.write(
                    RendPoint {
                        rend_type: 4,
                        x: it.x,
                        y: it.y,
                    }
                    .to_bytes(),
                );
            }
        });
        wrapped_buffer.write(&[0]);
    });
}

fn write_sound_effects(wrapped_buffer: &mut impl BufferWithCursor, states: &SoundEffect) {
    wrapped_buffer.write(
        RendSound {
            rend_type: 7,
            tap_sound: states.tap_count,
            drag_sound: states.drag_count,
            flick_sound: states.flick_count,
        }
        .to_bytes(),
    );
}

fn write_splash_effects(wrapped_buffer: &mut impl BufferWithCursor, states: &[SplashEffect]) {
    for it in states {
        if !it.enable {
            continue;
        }
        wrapped_buffer.write(
            RendSplashEffect {
                rend_type: 6,
                x: it.x as f32,
                y: it.y as f32,
                frame: ((30.0 * it.progress).floor() as i8).clamp(0, 29),
                tint_type: it.tint_type,
            }
            .to_bytes(),
        );
    }
}

fn write_click_effects(wrapped_buffer: &mut impl BufferWithCursor, states: &[HitEffect]) {
    for it in states {
        if !it.enable {
            continue;
        }
        wrapped_buffer.write(
            RendClickEffect {
                rend_type: 3,
                x: it.x as f32,
                y: it.y as f32,
                frame: ((30.0 * it.progress).floor() as i8).clamp(0, 29),
                tint_type: it.tint_type,
            }
            .to_bytes(),
        );
    }
}

fn write_line(wrapped_buffer: &mut impl BufferWithCursor, state: &LineData, world_rect: &Rect) {
    fn eq(a: f64, b: f64) -> bool {
        (a - b).abs() <= f64::EPSILON
    }
    let p1 = math::get_cross_point_with_screen(
        state.x,
        state.y,
        math::fix_degree(state.rotate),
        world_rect,
    );
    let p2 = math::get_cross_point_with_screen(
        state.x,
        state.y,
        math::fix_degree(state.rotate + 180.0),
        world_rect,
    );
    if state.alpha <= 0.0 {
        return;
    }
    if (((eq(p1.x, 0.0) && eq(p2.x, world_rect.width))
        || (eq(p2.x, 0.0) && eq(p1.x, world_rect.width)))
        && ((p1.y <= 0.0 && p2.y <= 0.0)
            || (p1.y >= world_rect.height && p2.y >= world_rect.height)))
        || (((eq(p1.y, 0.0) && eq(p2.y, world_rect.height))
            || (eq(p2.y, 0.0) && eq(p1.y, world_rect.height)))
            && ((p1.x <= 0.0 && p2.x <= 0.0)
                || (p1.x >= world_rect.width && p2.x >= world_rect.width)))
    {
        return;
    }
    let line = renders::RendLine {
        rend_type: 1,
        x1: p1.x as f32,
        y1: p1.y as f32,
        x2: p2.x as f32,
        y2: p2.y as f32,
        alpha: state.alpha as f32,
    };
    let line_slice = line.to_bytes();
    wrapped_buffer.write(line_slice);
}

fn write_notes(
    wrapped_buffer: &mut impl BufferWithCursor,
    states: &[LineData],
    offset: &DrawImageOffset,
    world_rect: &Rect,
) {
    let notes = states
        .iter()
        .fold((Vec::new(), Vec::new()), |(v1, v2), it| {
            process_notes(it, offset, world_rect, v1, v2)
        });
    notes
        .1
        .iter()
        .chain(notes.0.iter())
        .for_each(|it| wrapped_buffer.write(it.to_bytes()));
}

fn process_notes(
    state: &LineData,
    offset: &DrawImageOffset,
    world_rect: &Rect,
    mut vec: Vec<RendNote>,
    mut hold_vec: Vec<RendNote>,
) -> (Vec<RendNote>, Vec<RendNote>) {
    process_notes_half(
        state,
        offset,
        &state.notes_above_state,
        false,
        world_rect,
        &mut vec,
        &mut hold_vec,
    );
    process_notes_half(
        state,
        offset,
        &state.notes_below_state,
        true,
        world_rect,
        &mut vec,
        &mut hold_vec,
    );
    (vec, hold_vec)
}

fn check_in_bound(x: f64, y: f64) -> bool {
    (-200.0..=2120.0).contains(&x) && (-200.0..=1280.0).contains(&y)
}

fn process_notes_half(
    line_state: &LineState,
    offset: &DrawImageOffset,
    notes: &[NoteState],
    reverse: bool,
    world_rect: &Rect,
    out: &mut Vec<RendNote>,
    out_hold: &mut Vec<RendNote>,
) {
    let iter = notes.iter();
    for note_state in iter {
        let NoteState {
            note: Note {
                r#type: note_type, ..
            },
            score,
            ..
        } = note_state;
        if *score != NoteScore::None && *note_type != NoteType::Hold {
            continue;
        }
        match note_type {
            NoteType::Tap | NoteType::Drag | NoteType::Flick => {
                process_normal_note(reverse, line_state, note_state, world_rect, out);
            }
            NoteType::Hold => process_hold_note(
                reverse, line_state, note_state, offset, world_rect, out_hold,
            ),
        }
    }
}

fn process_normal_note(
    reverse: bool,
    line_state: &LineState,
    note_state: &NoteState,
    world_rect: &Rect,
    out: &mut Vec<RendNote>,
) {
    let LineState {
        x,
        y,
        rotate,
        line_y,
        ..
    } = line_state;
    let NoteState {
        note:
            Note {
                time,
                r#type: note_type,
                position_x,
                floor_position,
                speed,
                ..
            },
        highlight,
        ..
    } = note_state;
    let should_high_light = i8::from(*highlight);
    let delta_y = floor_position - line_y;
    if *time <= line_state.tick_time as i32 || *line_y > *floor_position + 0.01 {
        return;
    }
    let Point { x: raw_x, y: raw_y } =
        math::get_pos_out_of_line(*x, *y, *rotate, position_x * world_rect.get_unit_width());
    let Point { x, y } = math::get_pos_out_of_line(
        raw_x,
        raw_y,
        *rotate + if reverse { 90.0 } else { -90.0 },
        delta_y * world_rect.get_unit_height() * speed,
    );
    if !check_in_bound(x, y) {
        return;
    }
    out.push(RendNote {
        rend_type: 2,
        note_type: (*note_type).into(),
        x: x as f32,
        y: y as f32,
        rotate: *rotate as f32,
        height: 0.0,
        high_light: should_high_light,
    });
}

#[allow(clippy::too_many_lines)]
fn process_hold_note(
    reverse: bool,
    line_state: &LineState,
    note_state: &NoteState,
    offset: &DrawImageOffset,
    world_rect: &Rect,
    out_hold: &mut Vec<RendNote>,
) {
    let LineState {
        x,
        y,
        rotate,
        line_y,
        bpm,
        tick_time,
        ..
    } = line_state;
    let NoteState {
        note:
            Note {
                time,
                position_x,
                floor_position,
                speed,
                hold_time,
                ..
            },
        highlight,
        ..
    } = note_state;
    let should_high_light = i8::from(*highlight);
    let seconds_per_tick = 60.0 / bpm / 32.0;
    let head_position = floor_position - line_y;
    let body_height = (hold_time * speed).mul_add(seconds_per_tick, -0.0f64.max(-head_position));
    let body_position = floor_position + body_height / 2.0 - line_y + 0.0f64.max(-head_position);
    if *time + *hold_time as i32 <= *tick_time as i32 {
        return;
    }
    if body_position <= -body_height / 2.0 {
        return;
    }
    let Point {
        x: temp_x,
        y: temp_y,
    } = math::get_pos_out_of_line(*x, *y, *rotate, position_x * world_rect.get_unit_width());
    let Point { x: hx, y: hy } = math::get_pos_out_of_line(
        temp_x,
        temp_y,
        math::fix_degree(rotate + if reverse { 90.0 } else { -90.0 }),
        head_position.mul_add(
            world_rect.get_unit_height(),
            -(if *highlight {
                offset.hold_head_highlight_height / 2.0
            } else {
                offset.hold_head_height / 2.0
            }),
        ),
    );
    let Point { x: bx, y: by } = math::get_pos_out_of_line(
        temp_x,
        temp_y,
        math::fix_degree(rotate + if reverse { 90.0 } else { -90.0 }),
        body_position * world_rect.get_unit_height()
            + if body_position <= 0.0 {
                body_height / 2.0
            } else {
                0.0
            },
    );
    let hold_rect = math::Rect {
        cx: bx,
        cy: by,
        width: world_rect.width / 4.0,
        height: body_height * world_rect.get_unit_height(),
        rotate: rotate.to_radians(),
    };
    if !math::check_rectangles_overlap(world_rect, &hold_rect) {
        return;
    }
    let Point { x: ex, y: ey } = math::get_pos_out_of_line(
        temp_x,
        temp_y,
        math::fix_degree(rotate + if reverse { 90.0 } else { -90.0 }),
        (body_position + body_height / 2.0).mul_add(
            world_rect.get_unit_height(),
            if *highlight {
                offset.hold_end_highlight_height / 2.0
            } else {
                offset.hold_end_height / 2.0
            },
        ),
    );
    out_hold.push(RendNote {
        rend_type: 2,
        note_type: 7,
        x: ex as f32,
        y: ey as f32,
        rotate: math::fix_degree(*rotate + if reverse { 180.0 } else { 0.0 }) as f32,
        height: 0.0,
        high_light: 0,
    });
    out_hold.push(RendNote {
        rend_type: 2,
        note_type: 6,
        x: bx as f32,
        y: by as f32,
        rotate: math::fix_degree(*rotate + if reverse { 180.0 } else { 0.0 }) as f32,
        height: (body_height * world_rect.get_unit_height()) as f32,
        high_light: should_high_light,
    });
    if *time > *tick_time as i32 {
        out_hold.push(RendNote {
            rend_type: 2,
            note_type: 5,
            x: hx as f32,
            y: hy as f32,
            rotate: math::fix_degree(*rotate + if reverse { 180.0 } else { 0.0 }) as f32,
            height: 0.0,
            high_light: should_high_light,
        });
    }
}
