//! Easing curves and the small tweens that move, pop and tilt bar pieces.

use bevy::prelude::*;
use std::f32::consts::PI;

pub fn in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
    }
}

pub fn out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

const OVERSHOOT: f32 = 1.70158;

pub fn in_back(t: f32) -> f32 {
    (OVERSHOOT + 1.0) * t * t * t - OVERSHOOT * t * t
}

pub fn out_back(t: f32) -> f32 {
    let u = t - 1.0;
    1.0 + (OVERSHOOT + 1.0) * u * u * u + OVERSHOOT * u * u
}

pub fn in_out_back(t: f32) -> f32 {
    let c = OVERSHOOT * 1.525;

    if t < 0.5 {
        ((2.0 * t).powi(2) * ((c + 1.0) * 2.0 * t - c)) * 0.5
    } else {
        ((2.0 * t - 2.0).powi(2) * ((c + 1.0) * (t * 2.0 - 2.0) + c) + 2.0) * 0.5
    }
}

pub fn in_out_circ(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - (1.0 - (2.0 * t).powi(2)).max(0.0).sqrt()) * 0.5
    } else {
        ((1.0 - (-2.0 * t + 2.0).powi(2)).max(0.0).sqrt() + 1.0) * 0.5
    }
}

pub fn smooth_damp(current: f32, target: f32, speed: &mut f32, time: f32, dt: f32) -> f32 {
    let time = time.max(0.0001);
    let omega = 2.0 / time;
    let x = omega * dt;
    let decay = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let change = current - target;
    let temp = (*speed + omega * change) * dt;
    *speed = (*speed - omega * temp) * decay;
    let mut next = target + (change + temp) * decay;

    if (target - current > 0.0) == (next > target) {
        next = target;
        *speed = (next - target) / dt.max(f32::EPSILON);
    }

    next
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Curve {
    OutCubic,
    InBack,
    OutBack,
    InOutBack,
}

impl Curve {
    pub fn at(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::OutCubic => out_cubic(t),
            Self::InBack => in_back(t),
            Self::OutBack => out_back(t),
            Self::InOutBack => in_out_back(t),
        }
    }
}

pub const MOVE_TIME: f32 = 0.35;
pub const MOVE_DELAY: f32 = 0.02;

#[derive(Component, Debug, Clone, Copy)]
pub struct Mover {
    pub hidden: Vec2,
    pub visible: bool,
    from: Vec2,
    to: Vec2,
    clock: f32,
}

impl Mover {
    pub fn new(hidden: Vec2, visible: bool) -> Self {
        let at = if visible { Vec2::ZERO } else { hidden };

        Self {
            hidden,
            visible,
            from: at,
            to: at,
            clock: MOVE_TIME + MOVE_DELAY,
        }
    }

    pub fn show(&mut self, visible: bool) {
        if self.visible == visible {
            return;
        }

        self.from = self.current();
        self.to = if visible { Vec2::ZERO } else { self.hidden };
        self.visible = visible;
        self.clock = 0.0;
    }

    pub fn current(&self) -> Vec2 {
        let t = ((self.clock - MOVE_DELAY) / MOVE_TIME).clamp(0.0, 1.0);
        self.from.lerp(self.to, in_out_cubic(t))
    }

    pub fn settled(&self) -> bool {
        self.clock >= MOVE_TIME + MOVE_DELAY
    }

    pub fn tick(&mut self, dt: f32) {
        self.clock = (self.clock + dt).min(MOVE_TIME + MOVE_DELAY);
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Pop {
    from: Vec2,
    clock: f32,
    length: f32,
    curve: Curve,
}

impl Pop {
    pub fn new(from: Vec2, length: f32, curve: Curve) -> Self {
        Self {
            from,
            clock: 0.0,
            length,
            curve,
        }
    }

    pub fn scale(&self) -> Vec2 {
        let t = self.clock / self.length;
        self.from.lerp(Vec2::ONE, self.curve.at(t))
    }

    pub fn done(&self) -> bool {
        self.clock >= self.length
    }
}

pub const TILT: f32 = 20.0;
pub const TILT_SPEED: f32 = 100.0;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Tilt {
    pub target: f32,
    pub angle: f32,
}

pub fn movers(time: Res<Time>, mut movers: Query<(&mut Mover, &mut UiTransform)>) {
    let dt = time.delta_secs();

    for (mut mover, mut transform) in &mut movers {
        if !mover.settled() {
            mover.tick(dt);
        }

        let at = mover.current();
        let wanted = Val2::px(at.x, -at.y);

        if transform.translation != wanted {
            transform.translation = wanted;
        }
    }
}

pub fn pops(
    mut commands: Commands,
    time: Res<Time>,
    mut pops: Query<(Entity, &mut Pop, &mut UiTransform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut pop, mut transform) in &mut pops {
        pop.clock += dt;
        transform.scale = pop.scale();

        if pop.done() {
            transform.scale = Vec2::ONE;
            commands.entity(entity).remove::<Pop>();
        }
    }
}

pub fn tilts(time: Res<Time>, mut tilts: Query<(&mut Tilt, &mut UiTransform)>) {
    let step = TILT_SPEED * time.delta_secs();

    for (mut tilt, mut transform) in &mut tilts {
        if tilt.angle == tilt.target {
            continue;
        }

        let gap = tilt.target - tilt.angle;
        tilt.angle += gap.clamp(-step, step);
        transform.rotation = Rot2::radians(-tilt.angle * PI / 180.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_curve_starts_at_zero_and_ends_at_one() {
        for curve in [
            Curve::OutCubic,
            Curve::InBack,
            Curve::OutBack,
            Curve::InOutBack,
        ] {
            assert!(curve.at(0.0).abs() < 1e-5, "{curve:?}");
            assert!((curve.at(1.0) - 1.0).abs() < 1e-5, "{curve:?}");
        }

        assert!(in_out_circ(0.0).abs() < 1e-5);
        assert!((in_out_circ(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn back_curves_overshoot_and_cubic_curves_do_not() {
        assert!(in_back(0.3) < 0.0);
        assert!(out_back(0.7) > 1.0);
        assert!((0..=10).all(|step| (0.0..=1.0).contains(&in_out_cubic(step as f32 / 10.0))));
    }

    #[test]
    fn a_mover_takes_its_full_time_to_slide_out_and_back() {
        let mut mover = Mover::new(Vec2::new(-175.0, 0.0), true);
        mover.show(false);
        mover.tick(MOVE_DELAY);
        assert_eq!(mover.current(), Vec2::ZERO);

        mover.tick(MOVE_TIME * 0.5);
        assert!(mover.current().x < 0.0 && mover.current().x > -175.0);

        mover.tick(MOVE_TIME);
        assert_eq!(mover.current(), Vec2::new(-175.0, 0.0));
        assert!(mover.settled());
    }

    #[test]
    fn a_mover_reverses_from_where_it_is() {
        let mut mover = Mover::new(Vec2::new(0.0, -300.0), true);
        mover.show(false);
        mover.tick(MOVE_DELAY + MOVE_TIME * 0.5);
        let midway = mover.current();

        mover.show(true);
        assert_eq!(mover.current(), midway);
    }

    #[test]
    fn smooth_damp_settles_on_its_target() {
        let mut speed = 0.0;
        let mut at = 100.0;

        for _ in 0..200 {
            at = smooth_damp(at, 0.0, &mut speed, 0.1, 1.0 / 60.0);
        }

        assert!(at.abs() < 0.01, "{at}");
    }

    #[test]
    fn a_pop_springs_back_to_full_size() {
        let mut pop = Pop::new(Vec2::splat(0.9), 0.1, Curve::InOutBack);
        assert_eq!(pop.scale(), Vec2::splat(0.9));

        pop.clock = 0.1;
        assert_eq!(pop.scale(), Vec2::ONE);
        assert!(pop.done());
    }
}
