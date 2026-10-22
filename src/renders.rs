#[repr(C, packed)]
pub struct RendLine {
    pub rend_type: i8,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub alpha: f32,
}

#[repr(C, packed)]
pub struct RendNote {
    pub rend_type: i8,
    pub note_type: i8,
    pub x: f32,
    pub y: f32,
    pub rotate: f32,
    pub height: f32,
    pub high_light: i8,
}

#[repr(C, packed)]
pub struct RendClickEffect {
    pub rend_type: i8,
    pub x: f32,
    pub y: f32,
    pub frame: i8,
    pub tint_type: i8,
}

#[repr(C, packed)]
pub struct RendPoint {
    pub rend_type: i8,
    pub x: f32,
    pub y: f32,
}

#[repr(C, packed)]
pub struct RendStatistics {
    pub rend_type: i8,
    pub combo: u32,
    pub max_combo: u32,
    pub score: f32,
    pub accurate: f32,
}

#[repr(C, packed)]
pub struct RendSplashEffect {
    pub rend_type: i8,
    pub x: f32,
    pub y: f32,
    pub frame: i8,
    pub tint_type: i8,
}

#[repr(C, packed)]
pub struct RendSound {
    pub rend_type: i8,
    pub tap_sound: i8,
    pub drag_sound: i8,
    pub flick_sound: i8,
}

#[repr(C, packed)]
pub struct RendBlockAreaPolygon {
    pub rend_type: i8,
    pub time_state: i8,
    pub vertex_element_count: u32,
    pub vertex_index_count: u32,
    pub ready_progress: f32,
    // [Fill to align 4 byte]
    // [Vertexes as f32]
    // [Vertex index as u16]
}

#[repr(C, packed)]
pub struct RendBlockAreaDebugPos {
    pub rend_type: i8,
    pub index: u32,
    pub x: f32,
    pub y: f32,
}

pub trait Dense {
    fn to_bytes(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe {
            std::slice::from_raw_parts(
                std::ptr::from_ref::<Self>(self).cast::<u8>(),
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl Dense for RendLine {}
impl Dense for RendNote {}
impl Dense for RendClickEffect {}
impl Dense for RendPoint {}
impl Dense for RendStatistics {}
impl Dense for RendSplashEffect {}
impl Dense for RendSound {}
impl Dense for RendBlockAreaPolygon {}
impl Dense for RendBlockAreaDebugPos {}
