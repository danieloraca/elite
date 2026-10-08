use crate::sound::SoundEvent;
use macroquad::prelude::*;

pub(crate) const WIDTH: f32 = 960.0;
pub(crate) const HEIGHT: f32 = 720.0;
const VIEW_HEIGHT: f32 = 530.0;
const FOCAL: f32 = 620.0;
const STATION: Vec3 = Vec3::new(0.0, 0.0, 2000.0);
const CYAN: Color = Color::new(0.35, 0.95, 0.95, 1.0);
const DIM: Color = Color::new(0.17, 0.46, 0.53, 1.0);
const GOLD: Color = Color::new(1.0, 0.82, 0.3, 1.0);
const DANGER: Color = Color::new(1.0, 0.32, 0.35, 1.0);
const BACK: Color = Color::new(0.015, 0.025, 0.06, 1.0);
const PANEL: Color = Color::new(0.03, 0.065, 0.11, 1.0);

struct System {
    name: &'static str,
    economy: i32,
    colour: Color,
}

const SYSTEMS: [System; 6] = [
    System {
        name: "ASTER",
        economy: 0,
        colour: BLUE,
    },
    System {
        name: "KESTREL",
        economy: 2,
        colour: PURPLE,
    },
    System {
        name: "ORION",
        economy: -2,
        colour: GREEN,
    },
    System {
        name: "VESTA",
        economy: 3,
        colour: ORANGE,
    },
    System {
        name: "NARA",
        economy: -1,
        colour: SKYBLUE,
    },
    System {
        name: "CIRRUS",
        economy: 1,
        colour: VIOLET,
    },
];

struct Good {
    name: &'static str,
    base: i32,
    economy_effect: i32,
}

const GOODS: [Good; 4] = [
    Good {
        name: "FOOD",
        base: 18,
        economy_effect: -4,
    },
    Good {
        name: "TEXTILES",
        base: 35,
        economy_effect: -5,
    },
    Good {
        name: "ALLOYS",
        base: 65,
        economy_effect: 7,
    },
    Good {
        name: "COMPUTERS",
        base: 110,
        economy_effect: 12,
    },
];

fn price(system: usize, good: usize) -> i32 {
    let variation = ((system * 17 + good * 7) % 9) as i32 - 4;
    (GOODS[good].base + SYSTEMS[system].economy * GOODS[good].economy_effect + variation).max(2)
}

struct Ship {
    pos: Vec3,
    orientation: Quat,
    throttle: f32,
    speed: f32,
    shields: f32,
}

impl Ship {
    fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            throttle: 0.0,
            speed: 0.0,
            shields: 100.0,
        }
    }

    fn local(&self, point: Vec3) -> Vec3 {
        self.orientation.conjugate() * (point - self.pos)
    }
}

struct Pirate {
    pos: Vec3,
    hull: u8,
    attack_timer: f32,
    spin: f32,
}

#[derive(PartialEq)]
enum Mode {
    Title,
    Flight,
    Docked,
    Destroyed,
}

pub(crate) struct Game {
    mode: Mode,
    paused: bool,
    ship: Ship,
    system: usize,
    destination: usize,
    selected_good: usize,
    cargo: [u32; 4],
    credits: i32,
    fuel: u8,
    kills: u32,
    pirate: Option<Pirate>,
    pirate_respawn: f32,
    laser_cooldown: f32,
    laser_flash: f32,
    station_spin: f32,
    stars: Vec<Vec3>,
    message: String,
    message_timer: f32,
    sounds: Vec<SoundEvent>,
}

impl Game {
    pub(crate) fn new() -> Self {
        let mut seed = 0x5eed_1984_u32;
        let stars = (0..300)
            .map(|_| {
                Vec3::new(
                    random_range(&mut seed, -4400.0, 4400.0),
                    random_range(&mut seed, -3300.0, 3300.0),
                    random_range(&mut seed, -2500.0, 7000.0),
                )
            })
            .collect();
        Self {
            mode: Mode::Title,
            paused: false,
            ship: Ship::new(),
            system: 0,
            destination: 1,
            selected_good: 0,
            cargo: [0; 4],
            credits: 1000,
            fuel: 7,
            kills: 0,
            pirate: None,
            pirate_respawn: 8.0,
            laser_cooldown: 0.0,
            laser_flash: 0.0,
            station_spin: 0.0,
            stars,
            message: "APPROACH THE STATION - SLOW DOWN - PRESS E TO DOCK".into(),
            message_timer: 8.0,
            sounds: Vec::new(),
        }
    }

    pub(crate) fn music_active(&self) -> bool {
        matches!(self.mode, Mode::Flight | Mode::Docked) && !self.paused
    }

    pub(crate) fn take_sounds(&mut self) -> Vec<SoundEvent> {
        std::mem::take(&mut self.sounds)
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn pause_for_hidden_tab(&mut self) {
        if matches!(self.mode, Mode::Flight | Mode::Docked) {
            self.paused = true;
        }
    }

    fn say(&mut self, text: impl Into<String>) {
        self.message = text.into();
        self.message_timer = 3.5;
    }

    fn cargo_load(&self) -> u32 {
        self.cargo.iter().sum()
    }

    fn dock_ready(&self) -> bool {
        let local = self.ship.local(STATION);
        local.length() < 135.0
            && local.z > 0.0
            && local.x.abs() < 95.0
            && local.y.abs() < 95.0
            && self.ship.speed < 30.0
    }

    pub(crate) fn update(&mut self, dt: f32) {
        let dt = dt.min(0.05);
        if self.mode == Mode::Title {
            self.station_spin += dt * 0.15;
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                self.mode = Mode::Flight;
                self.sounds.push(SoundEvent::Start);
            }
            return;
        }
        if self.mode == Mode::Destroyed {
            if is_key_pressed(KeyCode::R) {
                *self = Self::new();
            }
            return;
        }
        if is_key_pressed(KeyCode::P) {
            self.paused = !self.paused;
        }
        if self.paused {
            return;
        }
        self.message_timer = (self.message_timer - dt).max(0.0);
        self.laser_cooldown = (self.laser_cooldown - dt).max(0.0);
        self.laser_flash = (self.laser_flash - dt).max(0.0);
        self.station_spin += dt * 0.15;
        match self.mode {
            Mode::Flight => self.update_flight(dt),
            Mode::Docked => self.update_market(),
            Mode::Title | Mode::Destroyed => unreachable!(),
        }
    }

    fn update_flight(&mut self, dt: f32) {
        if is_key_down(KeyCode::W) {
            self.ship.throttle = (self.ship.throttle + dt * 0.5).min(1.0);
        }
        if is_key_down(KeyCode::S) {
            self.ship.throttle = (self.ship.throttle - dt * 0.65).max(0.0);
        }
        let turn = 1.4 * dt;
        let pitch =
            (is_key_down(KeyCode::Down) as i32 - is_key_down(KeyCode::Up) as i32) as f32 * turn;
        let yaw = (is_key_down(KeyCode::D) as i32 - is_key_down(KeyCode::A) as i32) as f32 * turn;
        let roll =
            (is_key_down(KeyCode::Right) as i32 - is_key_down(KeyCode::Left) as i32) as f32 * turn;
        self.ship.orientation = (self.ship.orientation
            * Quat::from_rotation_x(pitch)
            * Quat::from_rotation_y(yaw)
            * Quat::from_rotation_z(roll))
        .normalize();
        let target = self.ship.throttle * 220.0;
        let step = 85.0 * dt;
        self.ship.speed += (target - self.ship.speed).clamp(-step, step);
        self.ship.pos += self.ship.orientation * Vec3::Z * self.ship.speed * dt;

        if self.ship.pos.distance(STATION) < 72.0 && self.ship.speed > 30.0 {
            let away = (self.ship.pos - STATION).normalize_or_zero();
            self.ship.pos = STATION
                + if away.length_squared() > 0.0 {
                    away * 74.0
                } else {
                    -Vec3::Z * 74.0
                };
            self.ship.speed = 0.0;
            self.ship.throttle = 0.0;
            self.ship.shields -= 35.0;
            self.sounds.push(SoundEvent::Damage);
            self.say("STATION COLLISION - SHIELDS DAMAGED");
        }

        if is_key_pressed(KeyCode::E) {
            if self.dock_ready() {
                self.mode = Mode::Docked;
                self.ship.speed = 0.0;
                self.ship.throttle = 0.0;
                self.ship.shields = 100.0;
                self.pirate = None;
                self.sounds.push(SoundEvent::Dock);
                self.say("DOCKING COMPLETE - WELCOME TO THE MARKET");
            } else {
                self.sounds.push(SoundEvent::Error);
                self.say("DOCKING DENIED - ALIGN AND SLOW BELOW 30");
            }
        }
        if self.mode != Mode::Flight {
            return;
        }

        if is_key_pressed(KeyCode::Tab) {
            self.destination = (self.destination + 1) % SYSTEMS.len();
            if self.destination == self.system {
                self.destination = (self.destination + 1) % SYSTEMS.len();
            }
            self.sounds.push(SoundEvent::Nav);
            self.say(format!(
                "NAVIGATION SET: {}",
                SYSTEMS[self.destination].name
            ));
        }
        if is_key_pressed(KeyCode::J) {
            if self.fuel == 0 {
                self.sounds.push(SoundEvent::Error);
                self.say("HYPERSPACE DENIED - NO FUEL");
            } else if self.ship.pos.distance(STATION) < 300.0 {
                self.sounds.push(SoundEvent::Error);
                self.say("HYPERSPACE DENIED - CLEAR THE STATION");
            } else {
                self.sounds.push(SoundEvent::Jump);
                self.fuel -= 1;
                self.system = self.destination;
                self.destination = (self.system + 1) % SYSTEMS.len();
                self.ship = Ship::new();
                self.pirate = None;
                self.pirate_respawn = 8.0;
                self.say(format!("ARRIVED IN {} SYSTEM", SYSTEMS[self.system].name));
                return;
            }
        }

        if is_key_down(KeyCode::Space) && self.laser_cooldown <= 0.0 {
            self.sounds.push(SoundEvent::Laser);
            self.laser_cooldown = 0.24;
            self.laser_flash = 0.12;
            let mut hit = false;
            let mut destroyed = false;
            if let Some(pirate) = &mut self.pirate {
                let local = self.ship.local(pirate.pos);
                if local.z > 0.0
                    && local.z < 1800.0
                    && local.x.abs() < local.z * 0.045
                    && local.y.abs() < local.z * 0.045
                {
                    pirate.hull -= 1;
                    hit = true;
                    destroyed = pirate.hull == 0;
                }
            }
            if destroyed {
                self.pirate = None;
                self.pirate_respawn = 18.0;
                self.kills += 1;
                self.credits += 80;
                self.sounds.push(SoundEvent::Explosion);
                self.say("PIRATE DESTROYED - 80 CR BOUNTY");
            } else if hit {
                self.sounds.push(SoundEvent::Hit);
                self.say("TARGET HIT");
            }
        }
        self.update_pirate(dt);
        if self.ship.shields <= 0.0 {
            self.ship.shields = 0.0;
            self.mode = Mode::Destroyed;
            self.sounds.push(SoundEvent::GameOver);
        }
    }

    fn update_pirate(&mut self, dt: f32) {
        if self.pirate.is_none() {
            self.pirate_respawn -= dt;
            if self.pirate_respawn <= 0.0 && self.ship.pos.distance(STATION) > 400.0 {
                self.pirate = Some(Pirate {
                    pos: self.ship.pos + self.ship.orientation * Vec3::new(-320.0, 100.0, 1050.0),
                    hull: 2,
                    attack_timer: 3.0,
                    spin: 0.0,
                });
                self.sounds.push(SoundEvent::Alert);
                self.say("HOSTILE CONTACT DETECTED");
            }
        }
        let mut attacked = false;
        if let Some(pirate) = &mut self.pirate {
            let delta = self.ship.pos - pirate.pos;
            let distance = delta.length();
            pirate.spin += dt * 0.8;
            pirate.attack_timer -= dt;
            if distance > 220.0 {
                pirate.pos += delta.normalize_or_zero() * 52.0 * dt;
            }
            if distance < 700.0 && pirate.attack_timer <= 0.0 {
                pirate.attack_timer = 2.6;
                self.ship.shields -= 9.0;
                attacked = true;
            }
        }
        if attacked {
            self.sounds.push(SoundEvent::Damage);
            self.say("LASER IMPACT - SHIELDS DAMAGED");
        }
    }

    fn update_market(&mut self) {
        if is_key_pressed(KeyCode::Up) {
            self.selected_good = (self.selected_good + GOODS.len() - 1) % GOODS.len();
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected_good = (self.selected_good + 1) % GOODS.len();
        }
        if is_key_pressed(KeyCode::B) {
            let cost = price(self.system, self.selected_good);
            if self.cargo_load() >= 20 {
                self.sounds.push(SoundEvent::Error);
                self.say("CARGO HOLD FULL");
            } else if self.credits < cost {
                self.sounds.push(SoundEvent::Error);
                self.say("INSUFFICIENT CREDITS");
            } else {
                self.sounds.push(SoundEvent::Trade);
                self.credits -= cost;
                self.cargo[self.selected_good] += 1;
                self.say(format!(
                    "BOUGHT 1 {} FOR {} CR",
                    GOODS[self.selected_good].name, cost
                ));
            }
        }
        if is_key_pressed(KeyCode::V) {
            if self.cargo[self.selected_good] == 0 {
                self.sounds.push(SoundEvent::Error);
                self.say("NONE OF THAT COMMODITY IN HOLD");
            } else {
                self.sounds.push(SoundEvent::Trade);
                let revenue = price(self.system, self.selected_good);
                self.cargo[self.selected_good] -= 1;
                self.credits += revenue;
                self.say(format!(
                    "SOLD 1 {} FOR {} CR",
                    GOODS[self.selected_good].name, revenue
                ));
            }
        }
        if is_key_pressed(KeyCode::F) {
            if self.fuel >= 7 {
                self.sounds.push(SoundEvent::Error);
                self.say("FUEL TANK FULL");
            } else if self.credits < 25 {
                self.sounds.push(SoundEvent::Error);
                self.say("INSUFFICIENT CREDITS FOR FUEL");
            } else {
                self.sounds.push(SoundEvent::Trade);
                self.credits -= 25;
                self.fuel += 1;
                self.say("BOUGHT 1 FUEL FOR 25 CR");
            }
        }
        if is_key_pressed(KeyCode::U) {
            self.sounds.push(SoundEvent::Undock);
            self.mode = Mode::Flight;
            self.ship.pos = STATION - Vec3::Z * 230.0;
            self.ship.orientation = Quat::from_rotation_y(std::f32::consts::PI);
            self.ship.throttle = 0.25;
            self.ship.speed = 30.0;
            self.pirate_respawn = 8.0;
            self.say("UNDOCKED - CLEAR THE STATION");
        }
    }
}

mod render;

fn random_range(seed: &mut u32, min: f32, max: f32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    min + (*seed as f32 / u32::MAX as f32) * (max - min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markets_have_distinct_positive_prices() {
        for good in 0..GOODS.len() {
            let prices: Vec<_> = (0..SYSTEMS.len())
                .map(|system| price(system, good))
                .collect();
            assert!(prices.iter().all(|price| *price > 0));
            assert!(prices.iter().min() < prices.iter().max());
        }
    }

    #[test]
    fn docking_requires_alignment_distance_and_low_speed() {
        let mut game = Game::new();
        game.ship.pos = STATION - Vec3::Z * 120.0;
        assert!(game.dock_ready());
        game.ship.speed = 31.0;
        assert!(!game.dock_ready());
        game.ship.speed = 0.0;
        game.ship.pos.x = 100.0;
        assert!(!game.dock_ready());
    }
}
