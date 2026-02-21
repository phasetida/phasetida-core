use crate::{
    LINE_STATES,
    chart::{self, TimeState, WithTimeRange, WithValue},
    math,
    states::LineState,
};

pub(crate) fn tick_lines(time_in_second: f64) {
    LINE_STATES.with_borrow_mut(|x| {
        for state in x.iter_mut() {
            tick_line_state(time_in_second, state);
        }
    });
}

fn get_line_y(tick_time: f64, line: &LineState) -> f64 {
    let mut t = 0.0;
    let seconds_per_tick = 60.0 / line.bpm / 32.0;
    let speed_events = &line.speed_events;
    for event in speed_events {
        if event.end_time > tick_time && event.start_time > tick_time {
            break;
        }
        if event.start_time < tick_time && tick_time < event.end_time {
            let duration = event.end_time - event.start_time;
            let percent = (tick_time - event.start_time) / duration;
            t += duration * percent * event.value;
            break;
        }
        if event.end_time < tick_time {
            t += (event.end_time - event.start_time) * event.value;
        }
    }
    t * seconds_per_tick
}

#[allow(clippy::similar_names)]
fn tick_line_state(time_in_second: f64, state: &mut LineState) {
    let seconds_per_tick = 60.0 / state.bpm / 32.0;
    let tick_time = time_in_second / seconds_per_tick;
    let ((speed_value, _), _, speed_new_index) = get_current_value_for_event(
        tick_time,
        &state.speed_events,
        state.event_speed_index_cache,
    );
    state.event_speed_index_cache = speed_new_index;
    state.speed = speed_value;
    let ((alpha_start, alpha_end), alpha_percent, alpha_new_index) = get_current_value_for_event(
        tick_time,
        &state.alpha_events,
        state.event_alpha_index_cache,
    );
    state.alpha = alpha_start + (alpha_end - alpha_start) * alpha_percent;
    state.event_alpha_index_cache = alpha_new_index;
    let ((rotate_start, rotate_end), rotate_percent, rotate_new_index) =
        get_current_value_for_event(
            tick_time,
            &state.rotate_events,
            state.event_rotate_index_cache,
        );
    state.rotate =
        math::fix_degree(360.0 - (rotate_start + (rotate_end - rotate_start) * rotate_percent));
    state.event_rotate_index_cache = rotate_new_index;
    let (((line_x_start, line_x_end), (line_y_start, line_y_end)), line_percent, line_new_index) =
        get_current_value_for_event(tick_time, &state.move_events, state.event_move_index_cache);
    state.x = math::WORLD_WIDTH * (line_x_start + (line_x_end - line_x_start) * line_percent);
    state.y =
        math::WORLD_HEIGHT * (1.0 - (line_y_start + (line_y_end - line_y_start) * line_percent));
    state.event_move_index_cache = line_new_index;
    state.line_y = get_line_y(tick_time, state);
    state.tick_time = tick_time;
}

fn get_current_value_for_event<T, U>(
    tick_time: f64,
    events: &[U],
    cache_index: i64,
) -> ((T, T), f64, i64)
where
    U: WithValue<T> + WithTimeRange,
{
    if events.is_empty() {
        return (U::zero(), 0.0, 0);
    }
    let event_result = find_current_event(tick_time, events, cache_index);
    if let Some((event, index, percent)) = event_result {
        (event.get_value(), percent, index)
    } else {
        (U::zero(), 0.0, 0)
    }
}

fn find_current_event<T>(tick_time: f64, events: &[T], cache_index: i64) -> Option<(&T, i64, f64)>
where
    T: WithTimeRange,
{
    let mut i = cache_index.clamp(0, i64::from(events.len() as u32));
    let mut last_result: TimeState = TimeState::During(0.0);
    loop {
        let op = events.get(i.max(0).unsigned_abs() as usize);
        if let Some(event) = op {
            let result = event.check_time(tick_time);
            match result {
                chart::TimeState::Early => {
                    if last_result == chart::TimeState::Late {
                        return Some((event, i, 1.0));
                    }
                    i -= 1;
                }
                chart::TimeState::Late => {
                    if last_result == chart::TimeState::Early {
                        return Some((event, i, 1.0));
                    }
                    i += 1;
                }
                chart::TimeState::During(percent) => return Some((event, i, percent)),
            }
            last_result = result;
        } else {
            if i <= 0 {
                return None;
            }
            return if let Some(x) = events.last() {
                Some((x, i64::from(events.len() as u32) - 1, 1.0))
            } else {
                None
            };
        }
    }
}
