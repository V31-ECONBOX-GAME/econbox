use bevy::prelude::*;

pub const LOOKS: usize = 28;
pub const DEFAULT: u8 = 0;
pub const DOWN: u8 = 1;
pub const UP: u8 = 2;
pub const HOLD: u8 = 6;
pub const DRAG: u8 = 7;
pub const PINKIE: u8 = 8;
pub const SPRINKLE: u8 = 13;
pub const DRAW: u8 = 17;
pub const ATTACK: u8 = 22;
pub const INSPECT: u8 = 26;
pub const OVER_UI: u8 = 27;

pub const ANIM_SPEED: i32 = 5;
pub const CLICK_FRAMES: i32 = 6;
pub const SPRINKLE_FRAMES: i32 = 4;
pub const DRAW_FRAMES: i32 = 5;
pub const PINKIE_LENGTH: i32 = 40;
pub const PRESS_DELAY: i32 = 3;
pub const DRAG_DELAY: i32 = 10;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HoldAnimation {
    #[default]
    Default,
    Sprinkle,
    Draw,
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CursorHints {
    pub over_button: bool,
    pub over_scroll: bool,
    pub dragging_item: bool,
    pub inspect: bool,
    pub attack: bool,
    pub hold: HoldAnimation,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LookInput {
    pub down_any: bool,
    pub down_right: bool,
    pub down_middle: bool,
    pub up_any: bool,
    pub up_right: bool,
    pub up_middle: bool,
    pub held_right: bool,
    pub held_middle: bool,
    pub any_key: bool,
    pub over_ui: bool,
    pub can_drag_map: bool,
    pub hints: CursorHints,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorLook {
    pub current: u8,
    counter: i32,
    pressed: bool,
    pressing: i32,
    dragged: i32,
    right: bool,
    middle: bool,
    anim_done: bool,
}

impl Default for CursorLook {
    fn default() -> Self {
        Self {
            current: DEFAULT,
            counter: 0,
            pressed: false,
            pressing: 0,
            dragged: 0,
            right: false,
            middle: false,
            anim_done: true,
        }
    }
}

impl CursorLook {
    pub fn pressed(&self) -> bool {
        self.pressed
    }

    fn rest(&mut self, hints: CursorHints) {
        self.current = if hints.over_button { OVER_UI } else { DEFAULT };
    }

    pub fn step(&mut self, input: LookInput) -> u8 {
        let hints = input.hints;

        if input.down_any {
            if !self.pressed {
                self.pressing = 0;
                self.dragged = 0;
                self.counter = 0;
            }
            self.pressed = true;
            self.anim_done = false;
            self.right = input.down_right;
            self.middle = input.down_middle;
        } else if input.up_any {
            if !input.any_key {
                self.pressed = false;
                self.pressing = 0;
                self.dragged = 0;
                self.right = input.up_right;
                self.middle = input.up_middle;
            } else {
                self.right = input.held_right;
                self.middle = input.held_middle;
            }
        } else if self.pressed {
            if !input.any_key {
                self.pressed = false;
                self.pressing = 0;
                self.dragged = 0;
                self.right = false;
                self.middle = false;
            } else {
                self.right = input.held_right;
                self.middle = input.held_middle;
                self.pressing += 1;
                if hints.dragging_item {
                    self.dragged += 1;
                }
            }
        }

        let can_drag = input.over_ui || input.can_drag_map;

        if hints.attack {
            self.current = ATTACK;
        } else if can_drag && hints.dragging_item && self.pressed && self.dragged > PRESS_DELAY {
            self.current = if self.dragged < DRAG_DELAY {
                HOLD
            } else {
                DRAG
            };
        } else if !self.pressed && self.anim_done {
            if hints.inspect {
                self.current = INSPECT;
            } else if self.right {
                self.current = PINKIE;
            } else {
                self.rest(hints);
            }
            self.counter = 0;
        } else if can_drag && self.pressed && self.pressing > PRESS_DELAY {
            if !input.over_ui || hints.over_scroll {
                self.current = if self.pressing < DRAG_DELAY {
                    HOLD
                } else {
                    DRAG
                };
            } else if self.right {
                self.current = PINKIE;
            } else {
                self.current = DOWN;
            }
        } else if !self.pressed && hints.inspect {
            self.current = INSPECT;
        } else if self.right {
            let frame = self.counter / ANIM_SPEED;
            self.current = if frame > 0 && frame < 5 {
                PINKIE + frame as u8
            } else {
                PINKIE
            };
            self.counter += 1;
            if self.counter > PINKIE_LENGTH {
                self.counter = 0;
                if !self.pressed {
                    self.anim_done = true;
                }
            }
        } else {
            let frames = match hints.hold {
                HoldAnimation::Sprinkle => {
                    let frame = (self.counter / ANIM_SPEED) % SPRINKLE_FRAMES;
                    self.current = SPRINKLE + frame as u8;
                    SPRINKLE_FRAMES
                }
                HoldAnimation::Draw => {
                    let frame = (self.counter / ANIM_SPEED) % DRAW_FRAMES;
                    self.current = DRAW + frame as u8;
                    DRAW_FRAMES
                }
                HoldAnimation::Default => {
                    let frame = self.counter / ANIM_SPEED;
                    if frame < CLICK_FRAMES {
                        self.current = frame as u8;
                    } else {
                        self.rest(hints);
                    }
                    CLICK_FRAMES
                }
            };
            self.counter += 1;
            if self.counter >= frames * ANIM_SPEED && !self.pressed {
                self.anim_done = true;
            }
        }

        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idle() -> LookInput {
        LookInput {
            can_drag_map: true,
            ..default()
        }
    }

    fn left_down() -> LookInput {
        LookInput {
            down_any: true,
            any_key: true,
            ..idle()
        }
    }

    fn left_held() -> LookInput {
        LookInput {
            any_key: true,
            ..idle()
        }
    }

    fn left_up() -> LookInput {
        LookInput {
            up_any: true,
            ..idle()
        }
    }

    fn run(look: &mut CursorLook, input: LookInput, frames: usize) -> Vec<u8> {
        (0..frames).map(|_| look.step(input)).collect()
    }

    #[test]
    fn an_idle_pointer_rests_on_the_plain_hand_or_the_button_hand() {
        let mut look = CursorLook::default();

        assert_eq!(look.step(idle()), DEFAULT);

        let over = LookInput {
            over_ui: true,
            hints: CursorHints {
                over_button: true,
                ..default()
            },
            ..idle()
        };
        assert_eq!(look.step(over), OVER_UI);
    }

    #[test]
    fn a_quick_click_plays_the_press_and_sparkle_frames_then_rests() {
        let mut look = CursorLook::default();
        let mut seen = vec![look.step(left_down())];
        seen.push(look.step(left_up()));
        seen.extend(run(&mut look, idle(), 40));

        assert_eq!(&seen[..5], &[DEFAULT; 5]);
        assert_eq!(&seen[5..10], &[DOWN; 5]);
        assert_eq!(&seen[10..15], &[UP; 5]);
        assert_eq!(&seen[25..30], &[UP + 3; 5]);
        assert!(seen[30..].iter().all(|look| *look == DEFAULT));
    }

    #[test]
    fn holding_the_button_on_open_map_turns_into_the_grab_hand() {
        let mut look = CursorLook::default();
        look.step(left_down());
        let held = run(&mut look, left_held(), 12);

        assert_eq!(&held[..3], &[DEFAULT, DEFAULT, DEFAULT]);
        assert!(held[3..9].iter().all(|look| *look == HOLD), "{held:?}");
        assert!(held[9..].iter().all(|look| *look == DRAG), "{held:?}");
    }

    #[test]
    fn a_painting_power_plays_its_own_loop_instead_of_grabbing() {
        let mut look = CursorLook::default();
        let draw = LookInput {
            can_drag_map: false,
            hints: CursorHints {
                hold: HoldAnimation::Draw,
                ..default()
            },
            ..left_held()
        };
        look.step(LookInput {
            down_any: true,
            ..draw
        });
        let frames = run(&mut look, draw, 30);

        assert!(frames.iter().all(|look| (DRAW..DRAW + 5).contains(look)));
        assert_eq!(frames[3], DRAW);
        assert_eq!(frames[4], DRAW + 1);
        assert_eq!(frames[24], DRAW);
    }

    #[test]
    fn a_right_click_wiggles_the_little_finger_then_keeps_it_up() {
        let mut look = CursorLook::default();
        let down = LookInput {
            down_any: true,
            down_right: true,
            any_key: true,
            can_drag_map: false,
            ..default()
        };
        let up = LookInput {
            up_any: true,
            up_right: true,
            can_drag_map: false,
            ..default()
        };
        let rest = LookInput {
            can_drag_map: false,
            ..default()
        };

        let mut seen = vec![look.step(down), look.step(up)];
        seen.extend(run(&mut look, rest, 60));

        assert_eq!(seen[0], PINKIE);
        assert_eq!(seen[5], PINKIE + 1);
        assert_eq!(seen[20], PINKIE + 4);
        assert_eq!(seen[25], PINKIE);
        assert_eq!(*seen.last().expect("frames"), PINKIE);
    }

    #[test]
    fn the_sword_and_the_magnifier_win_when_asked_for() {
        let mut look = CursorLook::default();
        let attack = LookInput {
            hints: CursorHints {
                attack: true,
                ..default()
            },
            ..idle()
        };
        assert_eq!(look.step(attack), ATTACK);

        let inspect = LookInput {
            hints: CursorHints {
                inspect: true,
                ..default()
            },
            ..idle()
        };
        assert_eq!(look.step(inspect), INSPECT);
    }

    #[test]
    fn dragging_a_toolbar_item_shows_the_grab_hand_even_over_the_bar() {
        let mut look = CursorLook::default();
        let dragging = LookInput {
            over_ui: true,
            can_drag_map: false,
            hints: CursorHints {
                dragging_item: true,
                ..default()
            },
            ..left_held()
        };

        look.step(LookInput {
            down_any: true,
            ..dragging
        });
        let frames = run(&mut look, dragging, 12);

        assert_eq!(frames[3], HOLD);
        assert_eq!(frames[11], DRAG);
    }
}
