use std::sync::Mutex;

use crate::core::init_renderer;

use lazy_static::lazy_static;

pub const INOCHI2D_SPEC_VERSION: &str = "1.0-alpha";

pub mod core;
pub mod math;
pub mod misc;
pub mod phys;

#[derive(Debug)]
struct InoxTime {
    pub current_time: f64,
    pub last_time: f64,
    pub delta_time: f64,
    pub time_func: fn() -> f64,
}

impl Default for InoxTime {
    fn default() -> Self {
        Self {
            current_time: Default::default(),
            last_time: Default::default(),
            delta_time: Default::default(),
            time_func: || 0.,
        }
    }
}

lazy_static! {
    static ref INOX_TIME: Mutex<InoxTime> = Mutex::new(InoxTime::default());
}

pub fn in_init(time_func: fn() -> f64) {
    init_renderer();
    let mut guard = INOX_TIME.lock().unwrap();
    guard.time_func = time_func;
}

pub fn in_update() {
    let mut guard = INOX_TIME.lock().unwrap();
    guard.current_time = (guard.time_func)();
    guard.delta_time = guard.current_time - guard.last_time;
    guard.last_time = guard.current_time;
}
