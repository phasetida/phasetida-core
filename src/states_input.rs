use crate::TOUCH_STATES;

/// Set a touch point as enabled
pub fn set_touch_down(id: usize, x: f32, y: f32) {
    TOUCH_STATES.with_borrow_mut(|it| {
        if let Some(touch) = it.get_mut(id) {
            touch.touch_down(x, y);
        }
    });
}

/// Move a touch point
pub fn set_touch_move(id: usize, x: f32, y: f32) {
    TOUCH_STATES.with_borrow_mut(|it| {
        if let Some(touch) = it.get_mut(id) {
            touch.touch_move(x, y);
        }
    });
}

/// Set a touch point as disabled
pub fn set_touch_up(id: usize) {
    TOUCH_STATES.with_borrow_mut(|it| {
        if let Some(touch) = it.get_mut(id) {
            touch.touch_up();
        }
    });
}

/// Clear the state of touch
pub fn clear_touch() {
    TOUCH_STATES.with_borrow_mut(|it| {
        for touch in it.iter_mut() {
            touch.enable = false;
        }
    });
}
