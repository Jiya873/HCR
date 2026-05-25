use haircut_core::{ClipperCommand, RuntimeCommand, Simulation};

/// A movement script drives the clipper. The server calls `tick` once per
/// physics step (~30 Hz). Implement this trait to describe a haircut.
pub trait MovementScript: Send {
    fn tick(&mut self, t: u64, sim: &mut Simulation);
}

/// Pick a script from the `HAIRCUT_SCRIPT` environment variable.
/// Default is `CutLine` — the simple template students extend.
pub fn from_env() -> Box<dyn MovementScript + Send> {
    match std::env::var("HAIRCUT_SCRIPT").as_deref() {
        Ok("raster") | Ok("raster_sweep") => Box::new(RasterSweep::default()),
        _ => Box::new(CutLine::default()),
    }
}

// =========================================================================
// STUDENT TEMPLATE
// =========================================================================
//
// This is the script students replace.  The goal is simple: drive the clipper
// across the head and cut a single horizontal line of hair.
//
// Each tick the server hands you:
//   * `t`   — the tick counter (0, 1, 2, ...), 30 ticks ≈ 1 second.
//   * `sim` — the live simulation; send it commands.
//
// Send commands with:
//     sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::<...>))
//
// Available `ClipperCommand` variants:
//   MoveLeft, MoveRight, MoveUp, MoveDown, MoveForward, MoveBackward
//     — nudge by one `move_speed` step on the named axis.
//   SetTargetXz { x, z }
//     — jump to (x, z) world coordinates (Y is left where it was).
//   ActivateCutting / DeactivateCutting
//     — turn the cutter on / off. Strands within the clipper sphere are cut
//       only while it is on.
//   Reset
//     — return the clipper to its starting position.
//
// The clipper starts at world (0, -1.7, 1.0): in front of the head, slightly
// above eye level. The head is centered at (0, 0, 0.8) with radii roughly
// (0.75, 0.85, 0.85). To reach hair, we usually want the clipper above the
// head (Z ≈ 2.0) and forward enough to touch strands (Y near 0).
// =========================================================================

#[derive(Default)]
pub struct CutLine;

impl MovementScript for CutLine {
    fn tick(&mut self, t: u64, sim: &mut Simulation) {
        // Phase 1: push the clipper forward until it is over the head.
        // 42 MoveForward calls × move_speed 0.04 ≈ 1.68 → Y moves from -1.7 to ~0.
        const APPROACH: u64 = 42;
        if t < APPROACH {
            let _ = sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::MoveForward));
            return;
        }

        // Phase 2: turn the cutter on once, the first tick we're in position.
        if t == APPROACH {
            let _ = sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::ActivateCutting));
        }

        // Phase 3: sweep the clipper from x = -1.0 to x = +1.0 over 200 ticks.
        //           Z is held constant — this draws a straight line cut.
        const SWEEP_TICKS: f32 = 200.0;
        let progress = ((t - APPROACH) as f32 / SWEEP_TICKS).clamp(0.0, 1.0);
        let x = -1.0 + 2.0 * progress * progress;
        let z = 2.0 - 1.0 * progress; // slight downward slope looks nicer 
        let _ = sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::SetTargetXz {
            x,
            z,
        }));

        // === Students: rewrite the body above to draw your own haircut. ===
    }
}

// =========================================================================
// Reference implementation: a back-and-forth raster sweep.
// Selected with `HAIRCUT_SCRIPT=raster`.
// =========================================================================

#[derive(Default)]
pub struct RasterSweep;

impl MovementScript for RasterSweep {
    fn tick(&mut self, t: u64, sim: &mut Simulation) {
        const APPROACH: u64 = 42;
        if t == 0 {
            let _ = sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::SetTargetXz {
                x: -1.0,
                z: 2.3,
            }));
        }
        if t < APPROACH {
            let _ = sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::MoveForward));
            return;
        }
        if t == APPROACH + 8 {
            let _ = sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::ActivateCutting));
        }
        const ROW_PERIOD: u64 = 200;
        let local = ((t - APPROACH) % ROW_PERIOD) as f32 / ROW_PERIOD as f32;
        let row = (t - APPROACH) / ROW_PERIOD;
        let x = if row % 2 == 0 {
            -1.0 + 2.0 * local
        } else {
            1.0 - 2.0 * local
        };
        let z = 2.30 - (row as f32) * 0.18;
        let _ = sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::SetTargetXz {
            x,
            z,
        }));
    }
}
