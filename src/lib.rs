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
#![deny(missing_docs)]
#![allow(clippy::cast_possible_truncation)]

use std::cell::RefCell;

mod chart;
mod draw;
mod input;
mod math;
mod renders;
mod states;
mod states_effect;
mod states_initializing;
mod states_input;
mod states_judge;
mod states_lines;
mod states_statistics;

thread_local! {
    pub(crate) static DRAW_IMAGE_OFFSET:RefCell<draw::DrawImageOffset> = RefCell::new(draw::DrawImageOffset::default());
    pub(crate) static FLATTEN_NOTE_INDEX:RefCell<Vec<states_statistics::NoteIndex>>= const{RefCell::new(Vec::<_>::new())};
    pub(crate) static LINE_STATES: RefCell<[states::LineState;50]> = RefCell::new(std::array::from_fn(|_|states::LineState::default()));
    pub(crate) static TOUCH_STATES: RefCell<[input::TouchInfo; 30]> = RefCell::new(std::array::from_fn(|_|input::TouchInfo::default()));
    pub(crate) static HIT_EFFECT_POOL: RefCell<[states_effect::HitEffect; 64]> = RefCell::new(std::array::from_fn(|_|states_effect::HitEffect::default()));
    pub(crate) static SPLASH_EFFECT_POOL : RefCell<[states_effect::SplashEffect;256]> = RefCell::new(std::array::from_fn(|_|states_effect::SplashEffect::default()));
    pub(crate) static CHART_STATISTICS: RefCell<states_statistics::ChartStatistics> = RefCell::new(states_statistics::ChartStatistics::default());
    pub(crate) static SOUND_POOL: RefCell<states_effect::SoundEffect> = RefCell::new(states_effect::SoundEffect::default());
}

pub use chart::Chart;
pub use chart::ChartRaw;
pub use draw::BufferWithCursor;
pub use states::Metadata;

pub use draw::load_image_offset;
pub use draw::process_state_to_drawable;

pub use states_initializing::clear_states;
pub use states_initializing::init_line_states;
pub use states_initializing::init_line_states_from_json;

pub use states_input::clear_touch;
pub use states_input::set_touch_down;
pub use states_input::set_touch_move;
pub use states_input::set_touch_up;

pub use states::reset_note_state;
pub use states::tick_all;
