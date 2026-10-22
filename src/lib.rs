//! A simple library that renders the official chart format of Phigros to dense
//! structured data.
//!
//! To use this library, you need to implement trait `BufferWithCursor` to
//! receive the structured data.
//!
//! This library only maintains states, you need to provide time parameter for
//! ticking the states
//!
//!

#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(missing_docs)]
#![allow(clippy::cast_possible_truncation)]

use std::cell::RefCell;

mod chart;
mod draw;
mod input;
mod math;
mod renders;
mod states;

thread_local! {
    pub(crate) static DRAW_IMAGE_OFFSET:RefCell<draw::DrawImageOffset> = RefCell::new(draw::DrawImageOffset::default());
    pub(crate) static FLATTEN_NOTE_INDEX:RefCell<Vec<states::statistics::NoteIndex>>= const { RefCell::new(Vec::<_>::new()) };
    pub(crate) static LINE_STATES: RefCell<Vec<states::LineData>> = const { RefCell::new(Vec::<_>::new()) };
    pub(crate) static TOUCH_STATES: RefCell<[input::TouchInfo; 30]> = RefCell::new(std::array::from_fn(|_|input::TouchInfo::default()));
    pub(crate) static BLOCK_AREA_STATES: RefCell<Vec<states::block_area::BlockAreaData>> = const { RefCell::new(Vec::new()) };
    pub(crate) static HIT_EFFECT_POOL: RefCell<[states::effect::HitEffect; 64]> = RefCell::new(std::array::from_fn(|_|states::effect::HitEffect::default()));
    pub(crate) static SPLASH_EFFECT_POOL : RefCell<[states::effect::SplashEffect;256]> = RefCell::new(std::array::from_fn(|_|states::effect::SplashEffect::default()));
    pub(crate) static CHART_STATISTICS: RefCell<states::statistics::ChartStatistics> = RefCell::new(states::statistics::ChartStatistics::default());
    pub(crate) static SOUND_POOL: RefCell<states::effect::SoundEffect> = RefCell::new(states::effect::SoundEffect::default());
    pub(crate) static WORLD_RECT: RefCell<math::Rect> = RefCell::new(math::Rect::world_default());
}

pub use chart::Chart;
pub use chart::ChartRaw;
pub use draw::BufferWithCursor;
pub use states::Metadata;

pub use draw::load_image_offset;
pub use draw::process_state_to_drawable;

pub use states::initializing::clear_states;
pub use states::initializing::init_line_states;
pub use states::initializing::init_states;
pub use states::initializing::init_world_rect;
pub use states::initializing::init_world_rect_with_block_area_update;
pub use states::initializing::parse_chart_json;
pub use states::initializing::recalculate_floor_position;

pub use states::input::clear_touch;
pub use states::input::set_touch_down;
pub use states::input::set_touch_move;
pub use states::input::set_touch_up;

pub use states::reset_note_state;
pub use states::tick_all;
