use std::cell::RefCell;

mod chart;
mod draw;
mod states_effect;
mod input;
mod math;
mod renders;
mod states;
mod states_initializing;
mod states_judge;
mod states_statistics;
mod states_lines;

thread_local! {
    pub static DRAW_IMAGE_OFFSET:RefCell<draw::DrawImageOffset> = RefCell::new(std::default::Default::default());
    pub static FLATTEN_NOTE_INDEX:RefCell<Vec<states_statistics::NoteIndex>>= RefCell::new(Vec::<_>::new());
    pub static LINE_STATES: RefCell<[states::LineState;50]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub static TOUCH_STATES: RefCell<[input::TouchInfo; 30]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub static HIT_EFFECT_POOL: RefCell<[states_effect::HitEffect; 64]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub static SPLASH_EFFECT_POOL : RefCell<[states_effect::SplashEffect;256]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub static CHART_STATISTICS: RefCell<states_statistics::ChartStatistics> = RefCell::new(std::default::Default::default());
    pub static SOUND_POOL: RefCell<states_effect::SoundEffect> = RefCell::new(std::default::Default::default());
}

pub use states::Metadata;
pub use draw::BufferWithCursor;
pub use chart::Chart;

pub use draw::load_image_offset;
pub use draw::process_state_to_drawable;

pub use states_judge::tick_lines_judge;
pub use states_statistics::refresh_chart_statistics;
pub use states_lines::tick_lines;
pub use states_effect::tick_effect;

pub use states_initializing::init_line_states;

pub use states::reset_note_state;
pub use states::tick_all;