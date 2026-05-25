use std::time::Duration;

use crate::{SIM_STATE, SimState};

pub async fn run() {
    let mut interval = tokio::time::interval(Duration::from_millis(33));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        interval.tick().await;
        let mut guard = SIM_STATE.lock().expect("sim state poisoned");
        let SimState { sim, tick, script } = &mut *guard;
        script.tick(*tick, sim);
        sim.step();
        *tick = tick.wrapping_add(1);
    }
}
