//! A simple library that renders the official chart format of Phigros to dense
//! structured data.
//!
//! To use this library, you need to implement trait `BufferWithCursor` to
//! receive the structured data.
//!
//! This library only maintains states, you need to provide time parameter for
//! ticking the states
//!
//! # Example
//!
//! ```rust
//! fn main() {
//!     phasetida_core::clear_states();
//!     let length = phasetida_core::init_line_states_from_json(json).unwrap();
//!     phasetida_core::load_image_offset(
//!         /*hold_head_height*/1.14,
//!         /*hold_head_highlight_height*/5.14,
//!         /*hold_end_height*/1.91,
//!         /*hold_end_highlight_height*/9.810,
//!     );
//!     let mut your_buffer=YourBuffer::new();
//!     phasetida_core::process_state_to_drawable(&mut your_buffer);
//!     // process your buffer
//! }
//! ```
//!

#![deny(missing_docs)]

use std::cell::RefCell;

mod chart;
mod draw;
mod input;
mod math;
mod renders;
mod states;
mod states_effect;
mod states_initializing;
mod states_judge;
mod states_lines;
mod states_statistics;

thread_local! {
    pub(crate) static DRAW_IMAGE_OFFSET:RefCell<draw::DrawImageOffset> = RefCell::new(std::default::Default::default());
    pub(crate) static FLATTEN_NOTE_INDEX:RefCell<Vec<states_statistics::NoteIndex>>= RefCell::new(Vec::<_>::new());
    pub(crate) static LINE_STATES: RefCell<[states::LineState;50]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub(crate) static TOUCH_STATES: RefCell<[input::TouchInfo; 30]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub(crate) static HIT_EFFECT_POOL: RefCell<[states_effect::HitEffect; 64]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub(crate) static SPLASH_EFFECT_POOL : RefCell<[states_effect::SplashEffect;256]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub(crate) static CHART_STATISTICS: RefCell<states_statistics::ChartStatistics> = RefCell::new(std::default::Default::default());
    pub(crate) static SOUND_POOL: RefCell<states_effect::SoundEffect> = RefCell::new(std::default::Default::default());
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

pub use states::reset_note_state;
pub use states::tick_all;
