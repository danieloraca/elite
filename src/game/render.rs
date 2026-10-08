use super::*;

impl Game {
    fn project(&self, point: Vec3) -> Option<Vec2> {
        let p = self.ship.local(point);
        (p.z > 5.0).then(|| {
            vec2(
                WIDTH * 0.5 + p.x * FOCAL / p.z,
                VIEW_HEIGHT * 0.5 - p.y * FOCAL / p.z,
            )
        })
    }

    fn line3(&self, a: Vec3, b: Vec3, colour: Color, thickness: f32) {
        if let (Some(a), Some(b)) = (self.project(a), self.project(b)) {
            draw_line(a.x, a.y, b.x, b.y, thickness, colour);
        }
    }

    pub(crate) fn render(&self, muted: bool) {
        clear_background(BACK);
        match self.mode {
            Mode::Title => self.render_title(muted),
            Mode::Flight => {
                self.render_space();
                self.render_cockpit(muted);
            }
            Mode::Docked => self.render_market(muted),
            Mode::Destroyed => {
                label("SHIP DESTROYED", 245.0, 290.0, 47.0, DANGER);
                label(
                    &format!("BOUNTIES COLLECTED: {}", self.kills),
                    305.0,
                    350.0,
                    25.0,
                    CYAN,
                );
                label("PRESS R TO START AGAIN", 313.0, 407.0, 25.0, GOLD);
                label(
                    if muted { "M SOUND OFF" } else { "M SOUND ON" },
                    394.0,
                    452.0,
                    18.0,
                    DIM,
                );
            }
        }
        if self.paused {
            draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, Color::new(0.0, 0.0, 0.0, 0.75));
            label("PAUSED", 376.0, 325.0, 54.0, GOLD);
            label("PRESS P TO CONTINUE", 346.0, 378.0, 24.0, CYAN);
        }
    }

    fn render_title(&self, muted: bool) {
        self.render_space();
        draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, Color::new(0.0, 0.0, 0.0, 0.65));
        draw_rectangle(
            60.0,
            100.0,
            840.0,
            520.0,
            Color::new(0.015, 0.025, 0.06, 0.96),
        );
        draw_rectangle_lines(60.0, 100.0, 840.0, 520.0, 2.0, CYAN);
        label("WIRESPACE", 235.0, 226.0, 78.0, CYAN);
        label("TRADE. FIGHT. SURVIVE.", 328.0, 272.0, 24.0, GOLD);
        draw_line(200.0, 307.0, 760.0, 307.0, 1.0, DIM);
        label(
            "FLY TO THE STATION. TRADE ACROSS SIX SYSTEMS.",
            222.0,
            361.0,
            21.0,
            CYAN,
        );
        label(
            "DEFEND YOUR SHIP. EARN YOUR PLACE AMONG THE STARS.",
            184.0,
            398.0,
            21.0,
            CYAN,
        );
        label("ENTER OR SPACE TO START", 318.0, 490.0, 29.0, GOLD);
        label(
            if muted {
                "M SOUND OFF  |  ARROWS TURN  |  W/S THRUST"
            } else {
                "M SOUND ON  |  ARROWS TURN  |  W/S THRUST"
            },
            315.0,
            568.0,
            18.0,
            DIM,
        );
    }

    fn render_space(&self) {
        for (i, star) in self.stars.iter().enumerate() {
            if let Some(p) = self.project(*star) {
                if (0.0..WIDTH).contains(&p.x) && (0.0..VIEW_HEIGHT).contains(&p.y) {
                    draw_circle(
                        p.x,
                        p.y,
                        if i % 11 == 0 { 1.4 } else { 0.8 },
                        if i % 9 == 0 { WHITE } else { GRAY },
                    );
                }
            }
        }
        let planet = Vec3::new(1150.0, -520.0, 6100.0);
        if let Some(p) = self.project(planet) {
            let radius = 520.0 * FOCAL / self.ship.local(planet).z;
            if radius < 2000.0 {
                let colour = SYSTEMS[self.system].colour;
                draw_circle(
                    p.x,
                    p.y,
                    radius,
                    Color::new(colour.r * 0.28, colour.g * 0.28, colour.b * 0.28, 1.0),
                );
                draw_circle_lines(p.x, p.y, radius, 1.5, colour);
                draw_circle_lines(p.x, p.y, radius * 0.72, 1.0, colour);
                draw_line(p.x - radius, p.y, p.x + radius, p.y, 1.0, colour);
            }
        }
        self.render_station();
        self.render_pirate();
    }

    fn render_station(&self) {
        let spin = Quat::from_rotation_z(self.station_spin);
        let vertex = |ring: usize, i: usize| {
            let a = i as f32 * std::f32::consts::TAU / 8.0;
            STATION
                + spin
                    * Vec3::new(
                        a.cos() * 100.0,
                        a.sin() * 100.0,
                        if ring == 0 { -55.0 } else { 55.0 },
                    )
        };
        for i in 0..8 {
            self.line3(vertex(0, i), vertex(0, (i + 1) % 8), CYAN, 1.5);
            self.line3(vertex(1, i), vertex(1, (i + 1) % 8), DIM, 1.0);
            self.line3(vertex(0, i), vertex(1, i), CYAN, 1.2);
        }
        let port = [
            Vec3::new(-28.0, -28.0, -57.0),
            Vec3::new(28.0, -28.0, -57.0),
            Vec3::new(28.0, 28.0, -57.0),
            Vec3::new(-28.0, 28.0, -57.0),
        ];
        for i in 0..4 {
            self.line3(
                STATION + spin * port[i],
                STATION + spin * port[(i + 1) % 4],
                GOLD,
                2.0,
            );
        }
        if let Some(p) = self.project(STATION) {
            if (0.0..WIDTH).contains(&p.x) && (0.0..VIEW_HEIGHT).contains(&p.y) {
                label("STATION", p.x + 24.0, p.y - 18.0, 16.0, CYAN);
            }
        }
    }

    fn render_pirate(&self) {
        let Some(pirate) = &self.pirate else { return };
        let points = [
            Vec3::new(0.0, 4.0, -33.0),
            Vec3::new(-42.0, 0.0, 24.0),
            Vec3::new(42.0, 0.0, 24.0),
            Vec3::new(0.0, 12.0, 16.0),
            Vec3::new(0.0, -10.0, 28.0),
        ];
        let rotation = Quat::from_rotation_y(pirate.spin);
        for (a, b) in [(0, 1), (0, 2), (1, 2), (0, 3), (1, 4), (2, 4), (3, 4)] {
            self.line3(
                pirate.pos + rotation * points[a],
                pirate.pos + rotation * points[b],
                DANGER,
                1.5,
            );
        }
        if let Some(p) = self.project(pirate.pos) {
            if (0.0..WIDTH).contains(&p.x) && (0.0..VIEW_HEIGHT).contains(&p.y) {
                label("HOSTILE", p.x + 20.0, p.y - 12.0, 15.0, DANGER);
            }
        }
    }

    fn render_cockpit(&self, muted: bool) {
        for (x1, y1, x2, y2) in [
            (467.0, 265.0, 478.0, 265.0),
            (482.0, 265.0, 493.0, 265.0),
            (480.0, 252.0, 480.0, 263.0),
            (480.0, 267.0, 480.0, 278.0),
        ] {
            draw_line(x1, y1, x2, y2, 1.2, GOLD);
        }
        if self.laser_flash > 0.0 {
            draw_line(285.0, VIEW_HEIGHT, 480.0, 265.0, 3.0, DANGER);
            draw_line(675.0, VIEW_HEIGHT, 480.0, 265.0, 3.0, DANGER);
        }
        draw_rectangle(0.0, VIEW_HEIGHT, WIDTH, HEIGHT - VIEW_HEIGHT, PANEL);
        draw_line(0.0, VIEW_HEIGHT, WIDTH, VIEW_HEIGHT, 2.0, CYAN);
        for x in [308.0, 652.0] {
            draw_line(x, VIEW_HEIGHT, x, HEIGHT, 1.0, DIM);
        }
        label(SYSTEMS[self.system].name, 24.0, 562.0, 27.0, CYAN);
        label(
            &format!("CREDITS {:04}", self.credits),
            24.0,
            592.0,
            22.0,
            GOLD,
        );
        label(
            &format!("CARGO {:02}/20", self.cargo_load()),
            24.0,
            620.0,
            19.0,
            CYAN,
        );
        label(&format!("KILLS {:02}", self.kills), 24.0, 646.0, 19.0, CYAN);
        label(
            if muted { "M SOUND OFF" } else { "M SOUND ON" },
            24.0,
            676.0,
            16.0,
            DIM,
        );
        label(
            &format!("SPEED {:03}", self.ship.speed as i32),
            682.0,
            562.0,
            22.0,
            CYAN,
        );
        label(
            &format!("THROTTLE {:03}%", (self.ship.throttle * 100.0) as i32),
            682.0,
            590.0,
            19.0,
            CYAN,
        );
        label(
            &format!("SHIELDS {:03}%", self.ship.shields.max(0.0) as i32),
            682.0,
            616.0,
            19.0,
            if self.ship.shields < 35.0 {
                DANGER
            } else {
                GOLD
            },
        );
        label(&format!("FUEL {}/7", self.fuel), 682.0, 642.0, 19.0, CYAN);
        label(
            &format!("STATION {:04} M", self.ship.pos.distance(STATION) as i32),
            350.0,
            562.0,
            18.0,
            CYAN,
        );
        label(
            if self.dock_ready() {
                "E - DOCK NOW"
            } else {
                "ALIGN + SLOW TO DOCK"
            },
            350.0,
            587.0,
            18.0,
            if self.dock_ready() { GOLD } else { DIM },
        );
        label(
            &format!("JUMP TO {}  [TAB]", SYSTEMS[self.destination].name),
            350.0,
            614.0,
            17.0,
            CYAN,
        );
        label("W/S THRUST  ARROWS TURN  A/D YAW", 350.0, 640.0, 15.0, DIM);
        label("SPACE LASER  E DOCK  J JUMP", 350.0, 666.0, 15.0, DIM);
        if self.message_timer > 0.0 {
            draw_rectangle(12.0, 12.0, 650.0, 32.0, PANEL);
            label(&self.message, 20.0, 35.0, 18.0, GOLD);
        }
    }

    fn render_market(&self, muted: bool) {
        label("STATION MARKET", 44.0, 73.0, 40.0, CYAN);
        label(
            &format!("{} SYSTEM", SYSTEMS[self.system].name),
            46.0,
            107.0,
            22.0,
            GOLD,
        );
        draw_line(44.0, 125.0, 916.0, 125.0, 2.0, CYAN);
        label("COMMODITY", 72.0, 172.0, 20.0, DIM);
        label("PRICE", 477.0, 172.0, 20.0, DIM);
        label("IN HOLD", 685.0, 172.0, 20.0, DIM);
        for (i, good) in GOODS.iter().enumerate() {
            let y = 225.0 + i as f32 * 62.0;
            if i == self.selected_good {
                draw_rectangle(54.0, y - 30.0, 850.0, 48.0, PANEL);
                draw_rectangle_lines(54.0, y - 30.0, 850.0, 48.0, 1.0, CYAN);
                label(">", 65.0, y + 2.0, 28.0, GOLD);
            }
            let colour = if i == self.selected_good { GOLD } else { CYAN };
            label(good.name, 100.0, y, 27.0, colour);
            label(
                &format!("{} CR", price(self.system, i)),
                477.0,
                y,
                25.0,
                colour,
            );
            label(&format!("{} T", self.cargo[i]), 685.0, y, 25.0, colour);
        }
        draw_line(44.0, 470.0, 916.0, 470.0, 1.0, CYAN);
        label(
            &format!("CREDITS {} CR", self.credits),
            54.0,
            515.0,
            25.0,
            GOLD,
        );
        label(
            &format!("HOLD {}/20 T", self.cargo_load()),
            420.0,
            515.0,
            25.0,
            CYAN,
        );
        label(&format!("FUEL {}/7", self.fuel), 730.0, 515.0, 25.0, CYAN);
        label(
            "UP/DOWN SELECT   B BUY   V SELL   F REFUEL (25 CR)   U UNDOCK",
            54.0,
            574.0,
            20.0,
            CYAN,
        );
        label(
            "Trade where prices differ. Each hyperspace jump uses one fuel unit.",
            54.0,
            612.0,
            18.0,
            DIM,
        );
        if self.message_timer > 0.0 {
            label(&self.message, 54.0, 665.0, 21.0, GOLD);
        }
        label(
            if muted { "M SOUND OFF" } else { "M SOUND ON" },
            54.0,
            700.0,
            16.0,
            DIM,
        );
    }
}

fn label(text: &str, x: f32, y: f32, size: f32, colour: Color) {
    draw_text(text, x, y, size, colour);
}
