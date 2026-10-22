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
            let ease_type_x = EaseType::try_from(this.ease_type_x).expect("illegal ease type_x");
            let ease_type_y = EaseType::try_from(this.ease_type_y).expect("illegal ease type_y");
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
            let ease_type_x = EaseType::try_from(this.ease_type_x).expect("illegal ease type_x");
            let ease_type_y = EaseType::try_from(this.ease_type_y).expect("illegal ease type_y");
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
