use crate::{
    BLOCK_AREA_STATES, CHART_STATISTICS, ChartRaw, FLATTEN_NOTE_INDEX, HIT_EFFECT_POOL,
    LINE_STATES, Metadata, SOUND_POOL, SPLASH_EFFECT_POOL, TOUCH_STATES, WORLD_RECT, chart,
    init_line_states,
    input::TouchInfo,
    math::Rect,
    states::{
        effect::{HitEffect, SoundEffect, SplashEffect},
        initializing::block_area::init_block_area,
        statistics::ChartStatistics,
    },
};

pub mod block_area;
pub mod lines;

/// Initialize world rect from width and height, then update block areas
pub fn init_world_rect_with_block_area_update(chart: &chart::Chart, width: f64, height: f64) {
    let new_world_rect = Rect::centered(width, height);
    init_world_rect(width, height);
    init_block_area(chart, &new_world_rect);
}

/// Initialize world rect from width and height
pub fn init_world_rect(width: f64, height: f64) {
    let new_world_rect = Rect::centered(width, height);
    WORLD_RECT.with_borrow_mut(move |world_rect| {
        *world_rect = new_world_rect;
    });
}

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
