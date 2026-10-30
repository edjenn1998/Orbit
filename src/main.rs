mod physics;

use macroquad::prelude::*;
use physics::*;

const BG: Color = Color::new(0.025, 0.041, 0.075, 1.0);
const PANEL: Color = Color::new(0.043, 0.064, 0.106, 1.0);
const TEXT: Color = Color::new(0.85, 0.90, 0.96, 1.0);
const MUTED: Color = Color::new(0.43, 0.52, 0.64, 1.0);
const TEAL: Color = Color::new(0.33, 0.91, 0.81, 1.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Orbit | Gravity playground".into(),
        window_width: 1360,
        window_height: 900,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

struct View {
    center: Vec2,
    scale: f32,
    width: f32,
    panel: f32,
}

impl View {
    fn new() -> Self {
        let panel = 280.0;
        let width = (screen_width() - panel).max(200.0);
        Self {
            center: vec2(width * 0.5, (screen_height() + 12.0) * 0.5),
            scale: (width / 1040.0)
                .min((screen_height() - 150.0) / 760.0)
                .max(0.2),
            width,
            panel,
        }
    }

    fn screen(&self, world: Vec2) -> Vec2 {
        self.center + world * self.scale
    }

    fn world(&self, screen: Vec2) -> Vec2 {
        (screen - self.center) / self.scale
    }

    fn in_field(&self, p: Vec2) -> bool {
        p.x > 0.0 && p.x < self.width && p.y > 80.0 && p.y < screen_height() - 68.0
    }
}

struct Particle {
    position: Vec2,
    velocity: Vec2,
    life: f32,
    color: Color,
}

struct Game {
    sim: Simulation,
    launcher: Vec2,
    class: MassClass,
    speed: f32,
    assist: bool,
    clockwise: bool,
    paused: bool,
    help: bool,
    trails: bool,
    warp: usize,
    accumulator: f32,
    cooldown: f32,
    particles: Vec<Particle>,
    notice: String,
    notice_time: f32,
    record: usize,
}

impl Game {
    fn new() -> Self {
        Self {
            sim: Simulation::default(),
            launcher: vec2(0.0, 265.0),
            class: MassClass::Light,
            speed: 103.0,
            assist: false,
            clockwise: true,
            paused: false,
            help: false,
            trails: true,
            warp: 0,
            accumulator: 0.0,
            cooldown: 0.0,
            particles: Vec::new(),
            notice: "TIP: Press T for circular-orbit assist, then click to launch.".into(),
            notice_time: 12.0,
            record: 0,
        }
    }

    fn velocity(&self, view: &View) -> Vec2 {
        if self.assist {
            let radial = self.launcher.normalize_or_zero();
            let tangent = vec2(-radial.y, radial.x);
            tangent
                * circular_speed(self.launcher.length())
                * if self.clockwise { 1.0 } else { -1.0 }
        } else {
            let (x, y) = mouse_position();
            (view.world(vec2(x, y)) - self.launcher).normalize_or_zero() * self.speed
        }
    }

    fn notify(&mut self, text: &str) {
        self.notice = text.into();
        self.notice_time = 4.0;
    }

    fn update(&mut self, dt: f32, view: &View) {
        self.notice_time = (self.notice_time - dt).max(0.0);
        self.cooldown = (self.cooldown - dt).max(0.0);
        if is_key_pressed(KeyCode::H) || is_key_pressed(KeyCode::F1) {
            self.help = !self.help;
        }
        if is_key_pressed(KeyCode::Escape) {
            if self.help {
                self.help = false;
            } else {
                self.paused = !self.paused;
            }
        }
        if is_key_pressed(KeyCode::P) {
            self.paused = !self.paused;
        }
        if is_key_pressed(KeyCode::R) && !self.help {
            let record = self.record.max(self.sim.best);
            *self = Self::new();
            self.record = record;
            self.notify("Fresh sky. Session record retained.");
        }
        if self.help {
            return;
        }
        for (key, class) in [
            (KeyCode::Key1, MassClass::Light),
            (KeyCode::Key2, MassClass::Medium),
            (KeyCode::Key3, MassClass::Heavy),
        ] {
            if is_key_pressed(key) {
                self.class = class;
            }
        }
        if is_key_pressed(KeyCode::T) {
            self.assist = !self.assist;
        }
        if is_key_pressed(KeyCode::C) {
            self.clockwise = !self.clockwise;
        }
        if is_key_pressed(KeyCode::V) {
            self.trails = !self.trails;
        }
        if is_key_pressed(KeyCode::Tab) {
            self.warp = (self.warp + 1) % 3;
        }

        let mut movement = Vec2::ZERO;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            movement.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            movement.x += 1.0;
        }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            movement.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            movement.y += 1.0;
        }
        let boost = if is_key_down(KeyCode::LeftShift) {
            2.0
        } else {
            1.0
        };
        self.launcher += movement.normalize_or_zero() * 210.0 * boost * dt;
        let (mx, my) = mouse_position();
        let mouse = vec2(mx, my);
        if is_mouse_button_pressed(MouseButton::Right) && view.in_field(mouse) {
            self.launcher = view.world(mouse);
        }
        let x_limit = (view.width * 0.5 - 22.0).max(20.0) / view.scale;
        let y_min = (88.0 - view.center.y) / view.scale;
        let y_max = (screen_height() - 84.0 - view.center.y) / view.scale;
        self.launcher.x = self.launcher.x.clamp(-x_limit, x_limit);
        self.launcher.y = self.launcher.y.clamp(y_min.min(y_max), y_max.max(y_min));
        let mut speed_delta = mouse_wheel().1 * 3.0;
        if is_key_down(KeyCode::Equal) || is_key_down(KeyCode::KpAdd) {
            speed_delta += dt * 70.0;
        }
        if is_key_down(KeyCode::Minus) || is_key_down(KeyCode::KpSubtract) {
            speed_delta -= dt * 70.0;
        }
        self.speed = (self.speed + speed_delta).clamp(10.0, 320.0);
        if !self.paused
            && self.cooldown <= 0.0
            && ((is_mouse_button_pressed(MouseButton::Left) && view.in_field(mouse))
                || is_key_pressed(KeyCode::Space))
        {
            let velocity = self.velocity(view);
            if velocity.length() < 1.0 {
                self.notify("Aim away from the launcher to choose a direction.");
            } else if self.sim.launch(self.launcher, velocity, self.class) {
                self.cooldown = 0.18;
            } else {
                self.notify(if self.sim.bodies.len() >= MAX_BODIES {
                    "Sky full: 180 active objects. Wait for a clear lane."
                } else {
                    "Move the launcher away from the primary."
                });
            }
        }
        if !self.paused {
            self.accumulator += dt.min(0.1) * [1.0, 2.0, 4.0][self.warp];
            while self.accumulator >= STEP {
                for impact in self.sim.step(STEP) {
                    for i in 0..16 {
                        let angle = i as f32 * 2.39996 + self.sim.time;
                        self.particles.push(Particle {
                            position: impact.position,
                            velocity: vec2(angle.cos(), angle.sin())
                                * (25.0 + (i % 5) as f32 * 13.0),
                            life: 1.0,
                            color: impact.class.color(),
                        });
                    }
                }
                self.accumulator -= STEP;
            }
            self.record = self.record.max(self.sim.best);
            for particle in &mut self.particles {
                particle.position += particle.velocity * dt;
                particle.life -= dt * 1.4;
            }
            self.particles.retain(|p| p.life > 0.0);
        }
    }

    fn draw(&self, view: &View) {
        clear_background(BG);
        self.draw_field(view);
        self.draw_panel(view);
        draw_rectangle(0.0, 0.0, view.width, 76.0, BG);
        label("O R B I T", 28.0, 39.0, 29.0, TEXT);
        label("A GRAVITATIONAL PLAYGROUND", 29.0, 61.0, 13.0, MUTED);
        let status = if self.help {
            "HELP"
        } else if self.paused {
            "PAUSED"
        } else {
            "LIVE SIMULATION"
        };
        dot(
            view.width - 188.0,
            31.0,
            3.0,
            if self.paused || self.help {
                ORANGE
            } else {
                TEAL
            },
        );
        label(status, view.width - 176.0, 36.0, 15.0, TEXT);
        label(
            &format!(
                "{:02}:{:02}  /  {}x",
                self.sim.time as u32 / 60,
                self.sim.time as u32 % 60,
                [1, 2, 4][self.warp]
            ),
            view.width - 176.0,
            58.0,
            14.0,
            MUTED,
        );
        let bottom = screen_height() - 64.0;
        draw_rectangle(0.0, bottom, view.width, 64.0, BG);
        draw_line(
            24.0,
            bottom,
            view.width - 24.0,
            bottom,
            1.0,
            alpha(MUTED, 0.22),
        );
        label(
            "WASD  move     MOUSE  aim     CLICK / SPACE  fire",
            27.0,
            bottom + 24.0,
            16.0,
            TEXT,
        );
        label(
            "1 / 2 / 3  mass     WHEEL  speed     T  assist     H  help",
            27.0,
            bottom + 47.0,
            15.0,
            MUTED,
        );
        if self.notice_time > 0.0 {
            let text = &self.notice;
            let size = 17.0_f32.min((view.width - 90.0) / text.len().max(1) as f32 * 1.9);
            let width = measure_text(text, None, size as u16, 1.0).width;
            draw_rectangle(
                view.width * 0.5 - width * 0.5 - 16.0,
                bottom - 49.0,
                width + 32.0,
                32.0,
                alpha(PANEL, 0.96),
            );
            label(text, (view.width - width) * 0.5, bottom - 28.0, size, TEAL);
        }
        if self.paused && !self.help {
            centered("PAUSED", view.width * 0.5, 116.0, 28.0, TEXT);
            centered("P or Esc to resume", view.width * 0.5, 140.0, 16.0, MUTED);
        }
        if self.help {
            self.draw_help();
        }
    }

    fn draw_field(&self, view: &View) {
        for i in 0..210 {
            let x = ((i * 7919 + 17) % 10007) as f32 / 10007.0 * view.width;
            let y = ((i * 3571 + 73) % 9973) as f32 / 9973.0 * screen_height();
            dot(
                x,
                y,
                if i % 9 == 0 { 1.2 } else { 0.65 },
                alpha(MUTED, 0.22 + (i % 4) as f32 * 0.12),
            );
        }
        for r in [125.0, 250.0, 375.0, 500.0, 650.0] {
            draw_circle_lines(
                view.center.x,
                view.center.y,
                r * view.scale,
                1.0,
                alpha(MUTED, 0.12),
            );
        }
        draw_line(
            view.center.x,
            80.0,
            view.center.x,
            screen_height() - 68.0,
            1.0,
            alpha(MUTED, 0.08),
        );
        draw_line(
            0.0,
            view.center.y,
            view.width,
            view.center.y,
            1.0,
            alpha(MUTED, 0.08),
        );

        if self.trails {
            for body in &self.sim.bodies {
                for i in 1..body.trail.len() {
                    let a = view.screen(body.trail[i - 1]);
                    let b = view.screen(body.trail[i]);
                    let fade = i as f32 / body.trail.len() as f32;
                    draw_line(
                        a.x,
                        a.y,
                        b.x,
                        b.y,
                        1.3 * view.scale,
                        alpha(body.class.color(), fade * 0.48),
                    );
                }
            }
        }

        // This is deliberately a primary-only guide, not a promised trajectory.
        let mut p = self.launcher;
        let mut v = self.velocity(view);
        for i in 0..420 {
            v += primary_acceleration(p) * 0.025;
            p += v * 0.05;
            v += primary_acceleration(p) * 0.025;
            if p.length() < PRIMARY_RADIUS + self.class.radius() || p.length() > ESCAPE_RADIUS {
                break;
            }
            if i % 4 == 0 {
                let s = view.screen(p);
                dot(
                    s.x,
                    s.y,
                    1.5,
                    alpha(self.class.color(), 0.5 * (1.0 - i as f32 / 440.0)),
                );
            }
        }

        let c = view.center;
        let r = PRIMARY_RADIUS * view.scale;
        for (extra, opacity) in [(30.0, 0.02), (19.0, 0.04), (10.0, 0.07)] {
            draw_circle(c.x, c.y, r + extra * view.scale, alpha(TEAL, opacity));
        }
        draw_circle(c.x, c.y, r, Color::from_rgba(17, 50, 62, 255));
        draw_circle_lines(c.x, c.y, r, 2.0, TEAL);
        draw_circle_lines(c.x, c.y, r * 0.79, 1.0, alpha(TEAL, 0.22));
        draw_circle(c.x - r * 0.22, c.y - r * 0.25, r * 0.25, alpha(TEAL, 0.12));
        centered("PRIMARY", c.x, c.y + r + 26.0, 13.0, MUTED);
        centered("2,000 M", c.x, c.y + r + 44.0, 12.0, alpha(MUTED, 0.75));

        for body in &self.sim.bodies {
            let s = view.screen(body.position);
            let r = body.class.radius() * view.scale;
            let color = body.class.color();
            draw_circle(s.x, s.y, r + 4.0, alpha(color, 0.08));
            draw_circle(s.x, s.y, r, color);
            draw_circle(s.x - r * 0.25, s.y - r * 0.25, r * 0.32, alpha(WHITE, 0.55));
            if body.orbiting() {
                draw_circle_lines(s.x, s.y, r + 3.0, 1.0, alpha(TEAL, 0.75));
            }
        }
        for particle in &self.particles {
            let s = view.screen(particle.position);
            dot(
                s.x,
                s.y,
                particle.life * 2.5 * view.scale,
                alpha(particle.color, particle.life),
            );
        }

        let s = view.screen(self.launcher);
        let direction = self.velocity(view).normalize_or_zero();
        let tip = s + direction * 28.0;
        let normal = vec2(-direction.y, direction.x);
        let valid = self.launcher.length() > PRIMARY_RADIUS + self.class.radius() + 2.0;
        let color = if valid { self.class.color() } else { RED };
        draw_circle_lines(s.x, s.y, 16.0, 1.5, alpha(color, 0.7));
        draw_line(s.x, s.y, tip.x, tip.y, 4.0, color);
        draw_triangle(
            tip + direction * 4.0,
            tip - direction * 5.0 + normal * 5.0,
            tip - direction * 5.0 - normal * 5.0,
            color,
        );
        draw_circle(s.x, s.y, 5.0, color);
        centered("LAUNCHER", s.x, s.y + 36.0, 12.0, color);

        let offscreen = self
            .sim
            .bodies
            .iter()
            .filter(|b| {
                let s = view.screen(b.position);
                !view.in_field(s)
            })
            .count();
        if offscreen > 0 {
            label(
                &format!("{} objects beyond view", offscreen),
                26.0,
                104.0,
                14.0,
                MUTED,
            );
        }
    }

    fn draw_panel(&self, view: &View) {
        let x = view.width;
        let left = x + 24.0;
        draw_rectangle(x, 0.0, view.panel, screen_height(), PANEL);
        draw_line(x, 0.0, x, screen_height(), 1.0, alpha(MUTED, 0.2));
        label("MISSION / ORBITAL SURVIVAL", left, 37.0, 13.0, MUTED);
        label("Keep them flying.", left, 70.0, 25.0, TEXT);
        label(
            "Complete a full revolution to score.",
            left,
            94.0,
            15.0,
            MUTED,
        );

        label("IN ORBIT", left, 143.0, 15.0, TEAL);
        label(
            &format!("{:02}", self.sim.orbiting()),
            left - 2.0,
            207.0,
            65.0,
            TEXT,
        );
        label(
            &format!("SESSION BEST   {:02}", self.record),
            left,
            236.0,
            15.0,
            TEAL,
        );
        divider(left, 258.0, view.panel - 48.0);
        stat("Active objects", self.sim.bodies.len(), left, 284.0);
        stat("Launched", self.sim.launched as usize, left, 310.0);
        stat("Destroyed", self.sim.destroyed as usize, left, 336.0);
        stat("Escaped", self.sim.escaped as usize, left, 362.0);
        divider(left, 382.0, view.panel - 48.0);
        label("PAYLOAD", left, 411.0, 13.0, MUTED);
        for (i, class) in [MassClass::Light, MassClass::Medium, MassClass::Heavy]
            .iter()
            .enumerate()
        {
            let y = 427.0 + i as f32 * 40.0;
            let selected = *class == self.class;
            if selected {
                draw_rectangle(left - 8.0, y - 1.0, 240.0, 36.0, alpha(class.color(), 0.09));
            }
            dot(left + 5.0, y + 17.0, class.radius() * 0.65, class.color());
            label(
                &format!("{}  {}", i + 1, class.name()),
                left + 23.0,
                y + 23.0,
                16.0,
                if selected { TEXT } else { MUTED },
            );
            label(
                &format!("{} M", class.mass() as u32),
                left + 181.0,
                y + 23.0,
                16.0,
                class.color(),
            );
        }
        let speed = self.velocity(view).length();
        label("LAUNCH SPEED", left, 578.0, 13.0, MUTED);
        label(&format!("{:05.1}", speed), left, 613.0, 32.0, TEXT);
        label("units / s", left + 119.0, 611.0, 15.0, MUTED);
        draw_rectangle(left, 629.0, 230.0, 4.0, alpha(MUTED, 0.2));
        draw_rectangle(
            left,
            629.0,
            (speed / 320.0).min(1.0) * 230.0,
            4.0,
            self.class.color(),
        );
        label(
            if self.assist {
                "T  CIRCULAR ASSIST: ON"
            } else {
                "T  CIRCULAR ASSIST: OFF"
            },
            left,
            661.0,
            15.0,
            if self.assist { TEAL } else { MUTED },
        );
        label(
            if self.assist {
                "C  reverse orbit direction"
            } else {
                "Wheel or +/- adjusts speed"
            },
            left,
            684.0,
            14.0,
            MUTED,
        );
        label("Guide: primary gravity only.", left, 721.0, 14.0, MUTED);
        label("Every live object adds gravity.", left, 743.0, 14.0, MUTED);
        if screen_height() >= 850.0 {
            divider(left, screen_height() - 125.0, view.panel - 48.0);
            label(
                "P  pause     TAB  time speed",
                left,
                screen_height() - 94.0,
                15.0,
                TEXT,
            );
            label(
                "R  restart    V  trails    H  help",
                left,
                screen_height() - 68.0,
                15.0,
                MUTED,
            );
            label(
                "THE PRIMARY IS INDESTRUCTIBLE",
                left,
                screen_height() - 30.0,
                12.0,
                MUTED,
            );
        }
    }

    fn draw_help(&self) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            alpha(BLACK, 0.85),
        );
        let width = 710.0_f32.min(screen_width() - 32.0);
        let x = (screen_width() - width) * 0.5;
        let y = ((screen_height() - 650.0) * 0.5).max(10.0);
        draw_rectangle(x, y, width, 650.0, PANEL);
        label("FIND THE BALANCE", x + 32.0, y + 47.0, 30.0, TEAL);
        let lines = [
            "Launch objects into orbit around the indestructible primary.",
            "Each survivor scores after one complete, bound revolution.",
            "Your score is the number still orbiting, not total launches.",
            "",
            "WASD / arrows       Move launcher (Shift = faster)",
            "Right-click         Place launcher at cursor",
            "Mouse               Aim toward cursor",
            "Left-click / Space  Launch selected mass",
            "1 / 2 / 3           Light / medium / heavy payload",
            "Wheel / + / -       Adjust manual launch speed",
            "T                   Toggle circular-orbit assist",
            "C                   Reverse assisted orbit direction",
            "P / Esc             Pause / resume",
            "Tab                 Cycle 1x / 2x / 4x simulation speed",
            "V                   Toggle trails",
            "R                   Restart (keeps this session's best)",
            "",
            "START: Press T, place launcher away from the primary, fire.",
            "Move to different radii before adding more objects.",
            "Heavier objects disturb their neighbors more strongly.",
            "Any collision destroys the player objects involved.",
            "The guide and assist use only the primary's gravity.",
            "Objects beyond 1,800 units escape. Maximum: 180 live objects.",
        ];
        for (i, line) in lines.iter().enumerate() {
            label(
                line,
                x + 32.0,
                y + 84.0 + i as f32 * 22.0,
                16.0,
                if i < 3 { TEXT } else { MUTED },
            );
        }
        label(
            "H / F1 / Esc   close help and resume",
            x + 32.0,
            y + 618.0,
            17.0,
            TEAL,
        );
    }
}

fn alpha(mut color: Color, opacity: f32) -> Color {
    color.a = opacity;
    color
}
fn label(text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text(text, x, y, size, color);
}
fn centered(text: &str, x: f32, y: f32, size: f32, color: Color) {
    let width = measure_text(text, None, size as u16, 1.0).width;
    label(text, x - width * 0.5, y, size, color);
}
fn dot(x: f32, y: f32, radius: f32, color: Color) {
    draw_circle(x, y, radius, color);
}
fn divider(x: f32, y: f32, width: f32) {
    draw_line(x, y, x + width, y, 1.0, alpha(MUTED, 0.22));
}
fn stat(name: &str, value: usize, x: f32, y: f32) {
    label(name, x, y, 17.0, MUTED);
    label(&format!("{value:03}"), x + 190.0, y, 18.0, TEXT);
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    loop {
        let view = View::new();
        game.update(get_frame_time().min(0.1), &view);
        game.draw(&view);
        next_frame().await;
    }
}
