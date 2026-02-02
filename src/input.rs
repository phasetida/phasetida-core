pub struct TouchInfo {
    pub enable: bool,
    pub x: f32,
    pub y: f32,
    pub touch_valid: bool,
    pub init_x: f32,
    pub init_y: f32,
}

impl Default for TouchInfo {
    fn default() -> Self {
        TouchInfo {
            enable: false,
            x: 0.0,
            y: 0.0,
            touch_valid: true,
            init_x: 0.0,
            init_y: 0.0,
        }
    }
}

impl TouchInfo {
    pub fn length(&self) -> f32 {
        ((self.x - self.init_x).powi(2) + (self.y - self.init_y).powi(2)).sqrt()
    }

    pub fn reset_length(&mut self) {
        self.init_x = self.x;
        self.init_y = self.y;
    }
}