use std::cell::RefCell;

pub mod chart;
pub mod draw;
pub mod effect;
pub mod input;
pub mod math;
pub mod renders;
pub mod states;
pub mod states_initializing;
pub mod states_judge;
pub mod states_statistics;
pub mod states_ticking;

thread_local! {
    pub static DRAW_IMAGE_OFFSET:RefCell<draw::DrawImageOffset> = RefCell::new(std::default::Default::default());
    pub static FLATTEN_NOTE_INDEX:RefCell<Vec<states_statistics::NoteIndex>>= RefCell::new(Vec::<_>::new());
    pub static LINE_STATES: RefCell<[states::LineState;50]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub static TOUCH_STATES: RefCell<[input::TouchInfo; 30]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub static HIT_EFFECT_POOL: RefCell<[effect::HitEffect; 64]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub static SPLASH_EFFECT_POOL : RefCell<[effect::SplashEffect;256]> = RefCell::new(std::array::from_fn(|_|std::default::Default::default()));
    pub static CHART_STATISTICS: RefCell<states_statistics::ChartStatistics> = RefCell::new(std::default::Default::default());
    pub static SOUND_POOL: RefCell<effect::SoundEffect> = RefCell::new(std::default::Default::default());
}
