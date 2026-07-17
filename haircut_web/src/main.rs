use hotaru::http::*;
use hotaru::prelude::*;

use std::sync::{Mutex, Once};

use haircut_core::{Simulation, SimulationConfig};

mod render_svg;
mod resource;
mod script;
mod ticker;

use script::MovementScript;

pub struct SimState {
    pub sim: Simulation,
    pub tick: u64,
    pub script: Box<dyn MovementScript + Send>,
}

pub static SIM_STATE: Lazy<Mutex<SimState>> = Lazy::new(|| {
    let sim = Simulation::new(SimulationConfig::default())
        .expect("default SimulationConfig is valid");
    Mutex::new(SimState {
        sim,
        tick: 0,
        script: script::from_env(),
    })
});

/// Gate the background ticker so the simulation only starts running when
/// someone actually visits the page. Subsequent visits are no-ops.
static TICKER_INIT: Once = Once::new();

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}

endpoint! {
    APP.url("/"),
    pub index <HTTP> {
        plain_template_response("scene.html")
    }
}

endpoint! {
    APP.url("/start"),
    pub start_cutting <HTTP> {
        // First call kicks off the simulation; later calls are no-ops.
        TICKER_INIT.call_once(|| {
            tokio::spawn(ticker::run());
        });
        text_response("started")
    }
}

endpoint! {
    APP.url("/state"),
    pub get_state <HTTP> {
        // Reports whether the background ticker has been spawned yet.
        // Lets a fresh page load reconcile its UI with server state.
        let s = if TICKER_INIT.is_completed() { "running" } else { "paused" };
        text_response(s)
    }
}

endpoint! {
    APP.url("/favicon.ico"),
    pub favicon <HTTP> {
        // Silence the browser's automatic /favicon.ico probe.
        normal_response(204u16, "")
    }
}

endpoint! {
    APP.url("/scene.svg"),
    pub scene_svg <HTTP> {
        let body = {
            let state = SIM_STATE.lock().expect("sim state poisoned");
            render_svg::render(&state.sim, state.tick)
        };
        html_response(body.into_bytes())
            .content_type(HttpContentType::ImageSvg())
    }
}

#[cfg(test)]
mod tests;
