use std::{collections::HashSet, default::Default};

use crate::{
    BLOCK_AREA_STATES, CHART_STATISTICS, FLATTEN_NOTE_INDEX, HIT_EFFECT_POOL, LINE_STATES,
    SOUND_POOL, SPLASH_EFFECT_POOL, TOUCH_STATES, WORLD_RECT,
    chart::{self, ChartRaw, Event1, JudgeLine, WithTimeRange},
    input::TouchInfo,
    math::Rect,
    states::{
        self, LineData, LineState, Metadata, NoteState,
        effect::{HitEffect, SoundEffect, SplashEffect},
        get_seconds_per_tick,
        initializing::block_area::init_block_area,
        lines::get_line_y_direct,
        statistics::ChartStatistics,
    },
};

/// Parse a chart from raw json.
///
/// # Errors
///
/// This function will return an error if the deserialization failed.
pub fn parse_chart_json(json: &str) -> Result<chart::Chart, serde_json::Error> {
    let chart_raw = serde_json::from_str::<ChartRaw>(json)?;
    Ok(chart_raw.convert_to_v3())
}

/// Initialize state of lines from chart.
///
/// # Errors
///
/// This function will return an error if the deserialization failed.
pub fn init_states(chart: &mut chart::Chart) -> Result<Metadata, serde_json::Error> {
    let world_rect = WORLD_RECT.with_borrow(|it| *it);
    let block_areas = chart
        .block_area_list
        .as_ref()
        .map_or_default(std::vec::Vec::len) as u32;
    init_block_area(chart, &world_rect);
    Ok(Metadata {
        format_version: chart.format_version,
        block_areas,
        ..init_line_states(chart)
    })
}

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

/// Initialize world rect from width and height, then update block areas
pub fn init_world_rect_with_block_area_update(chart: &chart::Chart, width: f64, height: f64) {
    let new_world_rect = Rect::centered(width, height);
    init_world_rect(width, height);
    block_area::init_block_area(chart, &new_world_rect);
}

/// Initialize world rect from width and height
pub fn init_world_rect(width: f64, height: f64) {
    let new_world_rect = Rect::centered(width, height);
    WORLD_RECT.with_borrow_mut(move |world_rect| {
        *world_rect = new_world_rect;
    });
}

/// Clear the states of lines
pub fn clear_states() {
    FLATTEN_NOTE_INDEX.with_borrow_mut(std::vec::Vec::clear);
    LINE_STATES.with_borrow_mut(std::vec::Vec::clear);
    BLOCK_AREA_STATES.with_borrow_mut(std::vec::Vec::clear);
    TOUCH_STATES.with_borrow_mut(|it| *it = std::array::from_fn(|_| TouchInfo::default()));
    HIT_EFFECT_POOL.with_borrow_mut(|it| *it = std::array::from_fn(|_| HitEffect::default()));
    SPLASH_EFFECT_POOL.with_borrow_mut(|it| *it = std::array::from_fn(|_| SplashEffect::default()));
    CHART_STATISTICS.with_borrow_mut(|it| *it = ChartStatistics::default());
    SOUND_POOL.with_borrow_mut(|it| *it = SoundEffect::default());
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

pub mod block_area {
    use geo::{Coord, MapCoords, Polygon};

    use crate::{
        BLOCK_AREA_STATES,
        chart::{
            self, BlockArea, BlockAreaMoveEvent, BlockAreaPosition, BlockAreaRotateEvent,
            BlockAreaScaleEvent, BlockAreaTrait,
        },
        math,
        states::block_area::{BlockAreaData, EaseType, Transform},
    };

    struct Sample {
        position: BlockAreaPosition,
        rotate_anchor: BlockAreaPosition,
        rotate: f64,
        scale_anchor: BlockAreaPosition,
        scale: BlockAreaPosition,
    }

    // 已完成今日……
    const SAMPLE_FREQUENCY: u32 = 325;

    /// Initialize the block areas from a chart
    pub fn init_block_area(chart: &chart::Chart, world_rect: &math::Rect) {
        let Some(ref block_area_list) = chart.block_area_list else {
            return;
        };
        BLOCK_AREA_STATES.with_borrow_mut(|data_sink| {
            data_sink.clear();
            for block_area in block_area_list {
                let mut data = BlockAreaData::default();
                data.state.enable = true;
                data.appear_time = block_area.appear_time;
                data.enable_time = block_area.enable_time;
                data.disable_time = block_area.disable_time;
                data.disappear_time = block_area.disappear_time;
                data.is_subtract = block_area.is_subtract;
                let duration = block_area.disappear_time - block_area.appear_time;
                let sample_step = duration / f64::from(SAMPLE_FREQUENCY);
                let init_polygon = get_init_polygon(block_area, world_rect);
                for sample_i in 0..SAMPLE_FREQUENCY {
                    let time = f64::from(sample_i).mul_add(sample_step, block_area.appear_time);
                    let sample = take_sample(block_area, time);
                    let polygon = apply_transform(block_area, &init_polygon, &sample, world_rect);
                    data.events.push((time, polygon));
                }
                data_sink.push(data);
            }
        });
    }

    fn get_init_polygon(block_area: &BlockArea, world_rect: &math::Rect) -> Polygon {
        let top_right = &block_area.top_right_percentage;
        let bottom_left = &block_area.bottom_left_percentage;
        let exterior = vec![
            (
                top_right.x * world_rect.width,
                top_right.y * world_rect.height,
            ),
            (
                bottom_left.x * world_rect.width,
                top_right.y * world_rect.height,
            ),
            (
                bottom_left.x * world_rect.width,
                bottom_left.y * world_rect.height,
            ),
            (
                top_right.x * world_rect.width,
                bottom_left.y * world_rect.height,
            ),
            (
                top_right.x * world_rect.width,
                top_right.y * world_rect.height,
            ),
        ];
        Polygon::new(exterior.into(), vec![])
    }

    fn apply_transform(
        block_area: &BlockArea,
        polygon: &Polygon,
        sample: &Sample,
        world_rect: &math::Rect,
    ) -> Polygon {
        let Sample {
            position,
            rotate_anchor,
            rotate,
            scale_anchor,
            scale,
        } = sample;
        let top_right = &block_area.top_right_percentage;
        let bottom_left = &block_area.bottom_left_percentage;
        let origin = Coord {
            x: f64::midpoint(top_right.x, bottom_left.x) * world_rect.width,
            y: f64::midpoint(top_right.y, bottom_left.y) * world_rect.height,
        };
        let center = Coord {
            x: 0.5 * world_rect.width,
            y: 0.5 * world_rect.height,
        };
        polygon.map_coords(|coord| {
            coord
                .translate(&-origin)
                .translate(&center)
                .scale(&scale_anchor.to_coord(world_rect), scale.x, scale.y)
                .rotate(&rotate_anchor.to_coord(world_rect), *rotate)
                .translate(&-center)
                .translate(&position.to_coord(world_rect))
        })
    }

    fn take_sample(block_area: &BlockArea, time: f64) -> Sample {
        let move_frame = get_key_frame(&block_area.move_events, time);
        let rotate_frame = get_key_frame(&block_area.rotate_events, time);
        let scale_frame = get_key_frame(&block_area.scale_events, time);
        let initial_center = BlockAreaPosition {
            x: f64::midpoint(
                block_area.top_right_percentage.x,
                block_area.bottom_left_percentage.x,
            ),
            y: f64::midpoint(
                block_area.top_right_percentage.y,
                block_area.bottom_left_percentage.y,
            ),
        };
        let position = get_position(move_frame, &initial_center, time);
        let (rotate_anchor, rotate) = get_rotate(rotate_frame, time);
        let (scale_anchor, scale) = get_scale(scale_frame, time);
        Sample {
            position,
            rotate_anchor,
            rotate,
            scale_anchor,
            scale,
        }
    }

    fn get_position(
        move_frame: Option<(&BlockAreaMoveEvent, Option<&BlockAreaMoveEvent>)>,
        initial_center: &BlockAreaPosition,
        time: f64,
    ) -> BlockAreaPosition {
        move_frame.map_or_else(
            || initial_center.clone(),
            |(this, option_next)| {
                let duration = get_frame_duration(this, option_next);
                let duration_percent = if duration == 0.0 {
                    1.0
                } else {
                    (time - this.time()) / duration
                };
                let this_position = &this.end_position;
                let ease_type_x =
                    EaseType::try_from(this.ease_type_x).expect("illegal ease type_x");
                let ease_type_y =
                    EaseType::try_from(this.ease_type_y).expect("illegal ease type_y");
                option_next.map_or_else(
                    || this_position.clone(),
                    |next| {
                        let next_position = &next.end_position;
                        BlockAreaPosition {
                            x: ease_type_x
                                .ease(duration_percent)
                                .mul_add(next_position.x - this_position.x, this_position.x),
                            y: ease_type_y
                                .ease(duration_percent)
                                .mul_add(next_position.y - this_position.y, this_position.y),
                        }
                    },
                )
            },
        )
    }

    fn get_rotate(
        rotate_frame: Option<(&BlockAreaRotateEvent, Option<&BlockAreaRotateEvent>)>,
        time: f64,
    ) -> (BlockAreaPosition, f64) {
        rotate_frame.map_or_else(
            || (BlockAreaPosition::default(), 0.0),
            |(this, option_next)| {
                let duration = get_frame_duration(this, option_next);
                let duration_percent = if duration == 0.0 {
                    1.0
                } else {
                    (time - this.time()) / duration
                };
                let this_rotate = this.rotation;
                let ease_type = EaseType::try_from(this.ease_type).expect("illegal ease type");
                option_next.map_or_else(
                    || (this.anchor.clone(), this_rotate),
                    |next| {
                        let next_rotate = next.rotation;
                        (
                            this.anchor.clone(),
                            ease_type
                                .ease(duration_percent)
                                .mul_add(next_rotate - this_rotate, this_rotate),
                        )
                    },
                )
            },
        )
    }

    fn get_scale(
        scale_frame: Option<(&BlockAreaScaleEvent, Option<&BlockAreaScaleEvent>)>,
        time: f64,
    ) -> (BlockAreaPosition, BlockAreaPosition) {
        scale_frame.map_or_else(
            || {
                (
                    BlockAreaPosition { x: 0.5, y: 0.5 },
                    BlockAreaPosition { x: 1.0, y: 1.0 },
                )
            },
            |(this, option_next)| {
                let duration = get_frame_duration(this, option_next);
                let duration_percent = if duration == 0.0 {
                    1.0
                } else {
                    (time - this.time()) / duration
                };
                let this_scale = &this.scale;
                let ease_type_x =
                    EaseType::try_from(this.ease_type_x).expect("illegal ease type_x");
                let ease_type_y =
                    EaseType::try_from(this.ease_type_y).expect("illegal ease type_y");
                option_next.map_or_else(
                    || (this.anchor.clone(), this_scale.clone()),
                    |next| {
                        let next_scale = &next.scale;
                        (
                            this.anchor.clone(),
                            BlockAreaPosition {
                                x: ease_type_x
                                    .ease(duration_percent)
                                    .mul_add(next_scale.x - this_scale.x, this_scale.x),
                                y: ease_type_y
                                    .ease(duration_percent)
                                    .mul_add(next_scale.y - this_scale.y, this_scale.y),
                            },
                        )
                    },
                )
            },
        )
    }

    fn get_frame_duration<T>(this: &T, option_next: Option<&T>) -> f64
    where
        T: BlockAreaTrait,
    {
        option_next.map_or_else(|| this.time(), BlockAreaTrait::time) - this.time()
    }

    fn get_key_frame<T>(events: &[T], time: f64) -> Option<(&T, Option<&T>)>
    where
        T: BlockAreaTrait + Clone,
    {
        events
            .iter()
            .enumerate()
            .rev()
            .find(|(_, it)| it.time() <= time)
            .map(|(i, this)| (this, events.get(i + 1)))
    }
}
