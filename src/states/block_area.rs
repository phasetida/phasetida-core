//! [Reference](https://teamflos.github.io/phira-docs/chart-standard/chart-format/phi/blockArea.html)
//!
//! Thank you!

use geo::{
    Coord, Distance, Euclidean, LineString, LinesIter, Polygon, line_measures::LengthMeasurable,
};
use num_enum::TryFromPrimitive;

use crate::BLOCK_AREA_STATES;

#[derive(Default)]
pub struct BlockAreaData {
    pub appear_time: f64,
    pub enable_time: f64,
    pub disable_time: f64,
    pub disappear_time: f64,
    pub is_subtract: bool,
    pub events: Vec<(f64, Polygon)>,
    pub state: BlockAreaState,
}

pub struct BlockAreaState {
    pub enable: bool,
    pub time_state: BlockAreaTimeState,
    pub ready_progress: f64,
    pub polygon: Polygon,
}

impl Default for BlockAreaState {
    fn default() -> Self {
        Self {
            enable: false,
            time_state: BlockAreaTimeState::default(),
            ready_progress: 0.0,
            polygon: Polygon::empty(),
        }
    }
}

#[repr(i8)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum BlockAreaTimeState {
    #[default]
    Hidden = 0,
    Disabled = 1,
    Ready = 2,
    Active = 3,
}

#[repr(u32)]
#[derive(TryFromPrimitive)]
pub enum EaseType {
    Linear = 0,
    InSine = 1,
    OutSine = 2,
    InOutSine = 3,
    InQuad = 4,
    OutQuad = 5,
    InOutQuad = 6,
    InCubic = 7,
    OutCubic = 8,
    InOutCubic = 9,
    InQuart = 10,
    OutQuart = 11,
    InOutQuart = 12,
    Zero = 13,
    One = 14,
}

impl EaseType {
    fn ease_in(times: f64, x: f64) -> f64 {
        x.powf(times)
    }

    fn ease_out(times: f64, x: f64) -> f64 {
        1.0 - (1.0 - x).powf(times)
    }

    fn ease_in_out(times: f64, x: f64) -> f64 {
        if x < 0.5 {
            0.5 * (2.0 * x).powf(times)
        } else {
            0.5f64.mul_add(-2.0f64.mul_add(-x, 2.0).powf(times), 1.0)
        }
    }

    /// 注意：这里的缓动类型与公式不符，为有意为之
    pub fn ease(&self, x: f64) -> f64 {
        match self {
            Self::Linear => x,
            Self::InSine => Self::ease_in(2.0, x),
            Self::OutSine => Self::ease_out(2.0, x),
            Self::InOutSine => Self::ease_in_out(2.0, x),
            Self::InQuad => Self::ease_in(3.0, x),
            Self::OutQuad => Self::ease_out(3.0, x),
            Self::InOutQuad => Self::ease_in_out(3.0, x),
            Self::InCubic => Self::ease_in(4.0, x),
            Self::OutCubic => Self::ease_out(4.0, x),
            Self::InOutCubic => Self::ease_in_out(4.0, x),
            Self::InQuart => Self::ease_in(5.0, x),
            Self::OutQuart => Self::ease_out(5.0, x),
            Self::InOutQuart => Self::ease_in_out(5.0, x),
            Self::Zero => 0.0,
            Self::One => 1.0,
        }
        .clamp(0.0, 1.0)
    }
}

const ENABLE_DELAY: f64 = 0.5;

pub fn tick_block_areas(time_in_second: f64) {
    BLOCK_AREA_STATES.with_borrow_mut(|states| {
        for data in states {
            let state = &mut data.state;
            if !state.enable {
                continue;
            }
            if time_in_second <= data.appear_time || time_in_second >= data.disappear_time {
                state.time_state = BlockAreaTimeState::Hidden;
            } else if time_in_second >= data.appear_time
                && time_in_second <= data.enable_time - ENABLE_DELAY
            {
                state.time_state = BlockAreaTimeState::Disabled;
            } else if time_in_second >= data.enable_time - ENABLE_DELAY
                && time_in_second <= data.enable_time
            {
                state.time_state = BlockAreaTimeState::Ready;
            } else if time_in_second >= data.enable_time && time_in_second <= data.disable_time {
                state.time_state = BlockAreaTimeState::Active;
            } else if time_in_second >= data.disable_time && time_in_second <= data.disappear_time {
                state.time_state = BlockAreaTimeState::Disabled;
            }
            state.ready_progress = ((time_in_second - (data.enable_time - ENABLE_DELAY))
                / ENABLE_DELAY)
                .clamp(0.0, 1.0);
            if state.time_state != BlockAreaTimeState::Hidden {
                let base_event = data
                    .events
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, (time, _))| *time <= time_in_second)
                    .or_else(|| data.events.first().map(|it| (0usize, it)));
                let polygon = if let Some((index, (base_time, base_polygon))) = base_event {
                    if let Some((next_time, next_polygon)) = data.events.get(index + 1) {
                        let duration = next_time - base_time;
                        let progress_percent = if duration == 0.0 {
                            0.0
                        } else {
                            (time_in_second - base_time) / duration
                        };
                        lerp_quad_polygon(base_polygon, next_polygon, progress_percent)
                    } else {
                        base_polygon.clone()
                    }
                } else {
                    // 这……不可能吧……？
                    Polygon::empty()
                };
                state.polygon = polygon;
            }
        }
    });
}

struct ArcTable<'a> {
    pts: &'a Vec<Coord<f64>>,
    cum: Vec<f64>,
    total: f64,
}

impl<'a> ArcTable<'a> {
    fn new(ring: &'a LineString<f64>) -> Self {
        let lines = ring.lines_iter();
        let mut cum = vec![0.0];
        let mut total = 0.0;
        for line in lines {
            total += line.length(&Euclidean);
            cum.push(total);
        }
        Self {
            pts: &ring.0,
            cum,
            total,
        }
    }

    fn at(&self, total_distance: f64) -> (Coord<f64>, Coord<f64>) {
        const EPS: f64 = 1e-9;
        let distance_within_total = total_distance.rem_euclid(self.total);
        let base_distance_index = self
            .cum
            .partition_point(|&c| c < distance_within_total)
            .saturating_sub(1);
        let base_distance_index = base_distance_index.min(self.pts.len() - 2);
        let segment_length = self.cum[base_distance_index + 1] - self.cum[base_distance_index];
        let segment_percentage = if segment_length > EPS {
            (distance_within_total - self.cum[base_distance_index]) / segment_length
        } else {
            0.0
        };
        let point_a = self.pts[base_distance_index];
        let point_b = self.pts[base_distance_index + 1];
        let point_target = Coord {
            x: (point_b.x - point_a.x).mul_add(segment_percentage, point_a.x),
            y: (point_b.y - point_a.y).mul_add(segment_percentage, point_a.y),
        };
        let len = Euclidean.distance(point_a, point_b);
        let unit_vector = if len > EPS {
            Coord {
                x: (point_b.x - point_a.x) / len,
                y: (point_b.y - point_a.y) / len,
            }
        } else {
            Coord { x: 1.0, y: 0.0 }
        };
        (point_target, unit_vector)
    }
}

fn lerp_coord(a: Coord<f64>, b: Coord<f64>, t: f64) -> Coord<f64> {
    Coord {
        x: (b.x - a.x).mul_add(t, a.x),
        y: (b.y - a.y).mul_add(t, a.y),
    }
}

fn lerp_quad_polygon(a: &Polygon<f64>, b: &Polygon<f64>, t: f64) -> Polygon<f64> {
    let a_points = &a.exterior().0;
    let b_points = &b.exterior().0;
    assert_eq!(a_points.len(), b_points.len());
    let mut points = Vec::with_capacity(a_points.len());
    for i in 0..a_points.len() {
        points.push(lerp_coord(a_points[i], b_points[i], t));
    }
    Polygon::new(LineString(points), vec![])
}

pub trait Transform {
    fn translate(self, delta: &Coord) -> Self;
    fn scale(self, anchor: &Coord, scale_x: f64, scale_y: f64) -> Self;
    fn rotate(self, anchor: &Coord, degrees: f64) -> Self;
}

impl Transform for Coord {
    fn translate(self, delta: &Coord) -> Self {
        Self {
            x: self.x + delta.x,
            y: self.y + delta.y,
        }
    }
    fn scale(self, anchor: &Coord, scale_x: f64, scale_y: f64) -> Self {
        Self {
            x: (self.x - anchor.x).mul_add(scale_x, anchor.x),
            y: (self.y - anchor.y).mul_add(scale_y, anchor.y),
        }
    }
    fn rotate(self, anchor: &Coord, degrees: f64) -> Self {
        let radians = degrees.to_radians();
        let (sin, cos) = radians.sin_cos();
        let dx = self.x - anchor.x;
        let dy = self.y - anchor.y;
        let rotated_x = dx.mul_add(cos, -dy * sin);
        let rotated_y = dx.mul_add(sin, dy * cos);
        Self {
            x: anchor.x + rotated_x,
            y: anchor.y + rotated_y,
        }
    }
}
