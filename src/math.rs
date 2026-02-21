#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub cx: f64,
    pub cy: f64,
    pub width: f64,
    pub height: f64,
    pub rotate: f64,
}

enum Gx {
    I,
    Ii,
    Iii,
    Iv,
}

pub static WORLD_WIDTH: f64 = 1920.0;
pub static WORLD_HEIGHT: f64 = 1080.0;
pub static UNIT_WIDTH: f64 = WORLD_WIDTH / 18.0;
pub static UNIT_HEIGHT: f64 = WORLD_HEIGHT * 0.6;

pub static WORLD_RECT: Rect = Rect {
    cx: WORLD_WIDTH / 2.0,
    cy: WORLD_HEIGHT / 2.0,
    width: WORLD_WIDTH,
    height: WORLD_HEIGHT,
    rotate: 0.0,
};

fn get_gx(valid_degree: f64) -> Gx {
    match valid_degree {
        315.0..=360.0 | 0.0..=45.0 => Gx::I,
        45.0..=135.0 => Gx::Ii,
        135.0..=225.0 => Gx::Iii,
        225.0..=315.0 => Gx::Iv,
        _ => panic!(""),
    }
}

pub fn get_cross_point_with_screen(line_x: f64, line_y: f64, valid_degree: f64) -> Point {
    let gx = get_gx(valid_degree);
    let rad = valid_degree.to_radians();
    let sin = rad.sin();
    let cos = rad.cos();
    let tan_cot = match gx {
        Gx::I | Gx::Iii => sin / cos,
        Gx::Ii | Gx::Iv => cos / sin,
    };
    match gx {
        Gx::I => Point {
            x: WORLD_WIDTH,
            y: line_y + (WORLD_WIDTH - line_x) * tan_cot,
        },
        Gx::Ii => Point {
            x: line_x + tan_cot * (WORLD_HEIGHT - line_y),
            y: WORLD_HEIGHT,
        },
        Gx::Iii => Point {
            x: 0.0,
            y: line_y - line_x * tan_cot,
        },
        Gx::Iv => Point {
            x: line_x - line_y * tan_cot,
            y: 0.0,
        },
    }
}

pub fn get_pos_out_of_line(line_x: f64, line_y: f64, any_degree: f64, distance: f64) -> Point {
    let rad = any_degree.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    let da = cos * distance;
    let db = sin * distance;
    Point {
        x: line_x + da,
        y: line_y + db,
    }
}

pub fn fix_degree(any_degree: f64) -> f64 {
    match any_degree {
        f if f < 0.0 => fix_degree(f + 360.0),
        f if f > 360.0 => fix_degree(f - 360.0),
        f => f,
    }
}

pub fn is_point_in_judge_range(
    line_x: f64,
    line_y: f64,
    valid_degree: f64,
    point_x: f64,
    point_y: f64,
    judge_width: f64,
) -> bool {
    let gx = get_gx(valid_degree);
    let rad = valid_degree.to_radians();
    let sin = rad.sin();
    let cos = rad.cos();
    let (p1, p2) = match gx {
        Gx::I | Gx::Iii => {
            let cot_or_tan = sin / cos;
            let ld1 = judge_width / cos;
            let d = point_y - line_y;
            let ld2 = d * cot_or_tan;
            let p1 = line_x - (ld2 + ld1);
            let p2 = line_x - (ld2 - ld1);
            (p1, p2)
        }
        Gx::Ii | Gx::Iv => {
            let cot_or_tan = cos / sin;
            let ld1 = judge_width / sin;
            let d = point_y - line_y;
            let ld2 = d * cot_or_tan;
            let p1 = line_y + (ld2 + ld1);
            let p2 = line_y + (ld2 - ld1);
            (p1, p2)
        }
    };
    match gx {
        Gx::I => point_x >= p1 && point_x <= p2,
        Gx::Iii => point_x >= p2 && point_x <= p1,
        Gx::Ii => point_y >= p2 && point_y <= p1,
        Gx::Iv => point_y >= p1 && point_y <= p2,
    }
}

pub fn get_pos_point_vertical_in_line(
    line_x: f64,
    line_y: f64,
    degree: f64,
    point_x: f64,
    point_y: f64,
) -> Point {
    let gx = get_gx(degree);
    let rad = degree.to_radians();
    let sin = rad.sin();
    let cos = rad.cos();
    match gx {
        Gx::I | Gx::Iii => {
            let tan = sin / cos;
            let tmp = point_y - line_y - (point_x - line_x) * tan;
            Point {
                x: point_x + tmp * cos * sin,
                y: point_y - tmp * cos * cos,
            }
        }
        Gx::Ii | Gx::Iv => {
            let cot = cos / sin;
            let tmp = point_x - line_x - (point_y - line_y) * cot;
            Point {
                x: point_x - tmp * sin * sin,
                y: point_y + tmp * sin * cos,
            }
        }
    }
}

fn dot_product(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    x1 * x2 + y1 * y2
}

fn get_projection_interval(rect: &Rect, axis_x: f64, axis_y: f64) -> (f64, f64) {
    let center_proj = rect.cx * axis_x + rect.cy * axis_y;
    let ux = rect.rotate.cos();
    let uy = rect.rotate.sin();
    let vx = -uy;
    let vy = ux;
    let half_w = rect.width / 2.0;
    let half_h = rect.height / 2.0;
    let r = half_w * dot_product(axis_x, axis_y, ux, uy).abs()
        + half_h * dot_product(axis_x, axis_y, vx, vy).abs();
    (center_proj - r, center_proj + r)
}

fn intervals_overlap(min1: f64, max1: f64, min2: f64, max2: f64) -> bool {
    !(max1 < min2 || max2 < min1)
}

pub fn check_rectangles_overlap(rect1: &Rect, rect2: &Rect) -> bool {
    let u1x = rect1.rotate.cos();
    let u1y = rect1.rotate.sin();
    let v1x = -u1y;
    let v1y = u1x;
    let u2x = rect2.rotate.cos();
    let u2y = rect2.rotate.sin();
    let v2x = -u2y;
    let v2y = u2x;
    let axes = [(u1x, u1y), (v1x, v1y), (u2x, u2y), (v2x, v2y)];
    for (axis_x, axis_y) in axes {
        if axis_x == 0.0 && axis_y == 0.0 {
            continue;
        }
        let (min1, max1) = get_projection_interval(rect1, axis_x, axis_y);
        let (min2, max2) = get_projection_interval(rect2, axis_x, axis_y);
        if !intervals_overlap(min1, max1, min2, max2) {
            return false;
        }
    }
    true
}
