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
        Self {
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
        (self.x - self.init_x).hypot(self.y - self.init_y)
    }

    pub const fn reset_length(&mut self) {
        self.init_x = self.x;
        self.init_y = self.y;
    }

    pub const fn touch_down(&mut self, x: f32, y: f32) {
        self.enable = true;
        self.touch_valid = true;
        self.init_x = x;
        self.init_y = y;
        self.touch_move(x, y);
    }

    pub const fn touch_move(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub const fn touch_up(&mut self) {
        self.enable = false;
    }
}
