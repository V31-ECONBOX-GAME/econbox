//! World time: whether it runs, how fast, and how much of it has passed.

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Speed {
    pub id: &'static str,
    pub label: &'static str,
    pub multiplier: f32,
}

pub const SPEEDS: [Speed; 9] = [
    Speed {
        id: "slow_mo",
        label: "x0.5",
        multiplier: 0.5,
    },
    Speed {
        id: "x1",
        label: "x1",
        multiplier: 1.0,
    },
    Speed {
        id: "x2",
        label: "x2",
        multiplier: 2.0,
    },
    Speed {
        id: "x3",
        label: "x3",
        multiplier: 3.0,
    },
    Speed {
        id: "x4",
        label: "x4",
        multiplier: 4.0,
    },
    Speed {
        id: "x5",
        label: "x5",
        multiplier: 5.0,
    },
    Speed {
        id: "x10",
        label: "x10",
        multiplier: 10.0,
    },
    Speed {
        id: "x15",
        label: "x15",
        multiplier: 15.0,
    },
    Speed {
        id: "x20",
        label: "x20",
        multiplier: 20.0,
    },
];

pub const NORMAL: usize = 1;
pub const ON_PANEL: usize = 6;
pub const TICK: f32 = 0.2;
pub const FRAME: f32 = 1.0 / 60.0;
pub const STEP: f32 = 0.02;
pub const CATCH_UP: u32 = 4;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct WorldClock {
    pub paused: bool,
    pub hold: bool,
    pub rung: usize,
    pub elapsed: f32,
    pub ticks: u64,
}

impl Default for WorldClock {
    fn default() -> Self {
        Self {
            paused: false,
            hold: false,
            rung: NORMAL,
            elapsed: 0.0,
            ticks: 0,
        }
    }
}

impl WorldClock {
    pub fn speed(&self) -> Speed {
        SPEEDS[self.rung.min(SPEEDS.len() - 1)]
    }

    pub fn rate(&self) -> f32 {
        if self.paused || self.hold {
            0.0
        } else {
            self.speed().multiplier
        }
    }

    pub fn toggle(&mut self) {
        self.paused = !self.paused;
    }

    pub fn faster(&mut self) {
        self.rung = (self.rung + 1).min(SPEEDS.len() - 1);
    }

    pub fn slower(&mut self) {
        self.rung = self.rung.saturating_sub(1);
    }

    pub fn cycle(&mut self) {
        self.rung = (self.rung + 1) % SPEEDS.len();
    }

    pub fn select(&mut self, id: &str) -> bool {
        let Some(rung) = SPEEDS.iter().position(|speed| speed.id == id) else {
            return false;
        };

        self.rung = rung;
        true
    }

    pub fn advance(&mut self, seconds: f32) {
        self.elapsed += seconds * self.rate();
        self.ticks = (self.elapsed / TICK) as u64;
    }

    pub fn step(&self) -> f32 {
        STEP * self.speed().multiplier
    }

    pub fn running(&self) -> bool {
        self.rate() > 0.0
    }

    pub fn label(&self) -> String {
        if self.paused {
            "paused".to_string()
        } else {
            self.speed().label.to_string()
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Frames {
    pub lag: f32,
    pub due: u32,
}

impl Frames {
    pub fn advance(&mut self, seconds: f32) -> u32 {
        self.lag += seconds;
        let due = (self.lag / FRAME) as u32;
        self.lag -= due as f32 * FRAME;

        if due > CATCH_UP {
            self.lag = 0.0;
        }

        self.due = due.min(CATCH_UP);
        self.due
    }
}

pub struct ClockPlugin;

impl Plugin for ClockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldClock>()
            .init_resource::<Frames>()
            .add_systems(PreUpdate, frame)
            .add_systems(Update, advance);
    }
}

fn frame(time: Res<Time>, mut frames: ResMut<Frames>) {
    frames.advance(time.delta_secs());
}

fn advance(time: Res<Time>, mut clock: ResMut<WorldClock>) {
    if clock.rate() == 0.0 {
        return;
    }

    clock.advance(time.delta_secs());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_clock_runs_at_one_times_speed() {
        let clock = WorldClock::default();

        assert!(!clock.paused);
        assert_eq!(clock.speed().id, "x1");
        assert_eq!(clock.rate(), 1.0);
        assert_eq!(clock.label(), "x1");
    }

    #[test]
    fn a_paused_clock_does_not_move() {
        let mut clock = WorldClock::default();
        clock.toggle();
        clock.advance(10.0);

        assert_eq!(clock.rate(), 0.0);
        assert_eq!(clock.elapsed, 0.0);
        assert_eq!(clock.ticks, 0);
        assert_eq!(clock.label(), "paused");
    }

    #[test]
    fn a_held_clock_stands_still_without_counting_as_paused() {
        let mut clock = WorldClock {
            hold: true,
            ..Default::default()
        };
        clock.advance(3.0);

        assert_eq!(clock.elapsed, 0.0);
        assert!(!clock.paused);
        assert_eq!(clock.label(), "x1");
    }

    #[test]
    fn stepping_faster_stops_at_the_top_speed() {
        let mut clock = WorldClock::default();

        for _ in 0..30 {
            clock.faster();
        }

        assert_eq!(clock.speed().id, "x20");
    }

    #[test]
    fn stepping_slower_stops_at_slow_motion() {
        let mut clock = WorldClock::default();

        for _ in 0..30 {
            clock.slower();
        }

        assert_eq!(clock.speed().id, "slow_mo");
        assert_eq!(clock.rate(), 0.5);
        assert_eq!(clock.label(), "x0.5");
    }

    #[test]
    fn cycling_wraps_from_the_top_back_to_slow_motion() {
        let mut clock = WorldClock::default();
        let seen: Vec<&str> = (0..SPEEDS.len())
            .map(|_| {
                clock.cycle();
                clock.speed().id
            })
            .collect();

        assert_eq!(seen.last(), Some(&"x1"));
        assert!(seen.contains(&"slow_mo"));
        assert!(seen.contains(&"x20"));
    }

    #[test]
    fn a_speed_is_picked_by_its_id() {
        let mut clock = WorldClock::default();

        assert!(clock.select("x4"));
        assert_eq!(clock.rate(), 4.0);
        assert!(!clock.select("x40"));
        assert_eq!(clock.speed().id, "x4");
    }

    #[test]
    fn the_panel_offers_slow_motion_up_to_five_times() {
        let offered: Vec<&str> = SPEEDS[..ON_PANEL].iter().map(|speed| speed.id).collect();

        assert_eq!(offered, ["slow_mo", "x1", "x2", "x3", "x4", "x5"]);
    }

    #[test]
    fn faster_time_runs_further_in_the_same_second() {
        let mut slow = WorldClock::default();
        let mut fast = WorldClock::default();
        fast.faster();

        slow.advance(1.0);
        fast.advance(1.0);

        assert!(fast.elapsed > slow.elapsed);
        assert_eq!(fast.elapsed, slow.elapsed * fast.rate());
    }

    #[test]
    fn ticks_count_the_elapsed_world_seconds() {
        let mut clock = WorldClock::default();
        clock.advance(1.0);

        assert_eq!(clock.ticks, (1.0 / TICK) as u64);

        clock.advance(TICK);
        assert_eq!(clock.ticks, (1.0 / TICK) as u64 + 1);
    }

    #[test]
    fn many_small_steps_add_up_to_the_same_stretch_of_time() {
        let mut clock = WorldClock::default();

        for _ in 0..100 {
            clock.advance(0.01);
        }

        assert!((clock.elapsed - 1.0).abs() < 1e-3, "{}", clock.elapsed);
    }

    #[test]
    fn worldbox_frames_come_sixty_a_second_and_each_moves_the_world_by_a_fiftieth() {
        let mut frames = Frames::default();
        let stepped: u32 = (0..600).map(|_| frames.advance(FRAME)).sum();
        let clock = WorldClock::default();

        assert_eq!(stepped, 600);
        assert!((stepped as f32 * clock.step() - 12.0).abs() < 1e-3);
    }

    #[test]
    fn a_stalled_frame_catches_up_only_a_few_worldbox_frames() {
        let mut frames = Frames::default();

        assert_eq!(frames.advance(2.0), CATCH_UP);
        assert_eq!(frames.lag, 0.0);
    }

    #[test]
    fn a_world_frame_scales_with_the_speed_and_keeps_running_while_paused() {
        let mut clock = WorldClock::default();
        clock.select("x5");
        clock.toggle();

        assert!((clock.step() - 0.1).abs() < 1e-6);
        assert!(!clock.running());
    }

    #[test]
    fn unpausing_picks_up_where_it_stopped() {
        let mut clock = WorldClock::default();
        clock.advance(1.0);
        let held = clock.elapsed;

        clock.toggle();
        clock.advance(5.0);
        assert_eq!(clock.elapsed, held);

        clock.toggle();
        clock.advance(1.0);
        assert!(clock.elapsed > held);
    }
}
