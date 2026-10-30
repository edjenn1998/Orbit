use macroquad::prelude::*;
use std::collections::VecDeque;

pub const G: f32 = 1400.0;
pub const PRIMARY_MASS: f32 = 2000.0;
pub const PRIMARY_RADIUS: f32 = 38.0;
pub const ESCAPE_RADIUS: f32 = 1800.0;
pub const STEP: f32 = 1.0 / 120.0;
pub const MAX_BODIES: usize = 180;
const SOFTENING: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MassClass {
    Light,
    Medium,
    Heavy,
}

impl MassClass {
    pub fn mass(self) -> f32 {
        match self {
            // Keep even light payloads influential during a close encounter.
            // At a 250-unit orbit and 50-unit separation, a light neighbor's
            // pull is about one third of the primary's acceleration.
            Self::Light => 24.0,
            Self::Medium => 72.0,
            Self::Heavy => 180.0,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Self::Light => 5.0,
            Self::Medium => 8.0,
            Self::Heavy => 12.0,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Light => Color::from_rgba(85, 233, 206, 255),
            Self::Medium => Color::from_rgba(112, 165, 255, 255),
            Self::Heavy => Color::from_rgba(255, 179, 98, 255),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Light => "LIGHT",
            Self::Medium => "MEDIUM",
            Self::Heavy => "HEAVY",
        }
    }
}

#[derive(Clone)]
pub struct Body {
    pub position: Vec2,
    pub velocity: Vec2,
    pub class: MassClass,
    pub angle_traveled: f32,
    pub age: f32,
    pub trail: VecDeque<Vec2>,
    trail_clock: f32,
}

impl Body {
    pub fn new(position: Vec2, velocity: Vec2, class: MassClass) -> Self {
        Self {
            position,
            velocity,
            class,
            angle_traveled: 0.0,
            age: 0.0,
            trail: VecDeque::from([position]),
            trail_clock: 0.0,
        }
    }

    pub fn is_bound(&self) -> bool {
        self.velocity.length_squared() * 0.5
            < G * PRIMARY_MASS / self.position.length().max(PRIMARY_RADIUS)
    }

    pub fn laps(&self) -> u32 {
        (self.angle_traveled.abs() / std::f32::consts::TAU) as u32
    }

    pub fn orbiting(&self) -> bool {
        self.laps() > 0 && self.is_bound()
    }
}

pub struct Impact {
    pub position: Vec2,
    pub class: MassClass,
}

#[derive(Default)]
pub struct Simulation {
    pub bodies: Vec<Body>,
    pub launched: u32,
    pub destroyed: u32,
    pub escaped: u32,
    pub best: usize,
    pub time: f32,
}

/// Plummer-softened Newtonian acceleration. Pair forces are equal and opposite.
fn acceleration(delta: Vec2, source_mass: f32) -> Vec2 {
    let d2 = delta.length_squared() + SOFTENING * SOFTENING;
    delta * (G * source_mass / (d2 * d2.sqrt()))
}

pub fn primary_acceleration(position: Vec2) -> Vec2 {
    acceleration(-position, PRIMARY_MASS)
}

pub fn circular_speed(radius: f32) -> f32 {
    // Exactly matches the softened force used by the simulation.
    let d2 = radius * radius + SOFTENING * SOFTENING;
    (G * PRIMARY_MASS * radius * radius / d2.powf(1.5)).sqrt()
}

/// Minimum distance to the origin during a linear movement segment.
fn segment_distance(start: Vec2, end: Vec2) -> f32 {
    let delta = end - start;
    let t = (-start.dot(delta) / delta.length_squared().max(1e-12)).clamp(0.0, 1.0);
    (start + t * delta).length()
}

impl Simulation {
    pub fn launch(&mut self, position: Vec2, velocity: Vec2, class: MassClass) -> bool {
        if self.bodies.len() >= MAX_BODIES
            || position.length() <= PRIMARY_RADIUS + class.radius() + 2.0
        {
            return false;
        }
        self.bodies.push(Body::new(position, velocity, class));
        self.launched += 1;
        true
    }

    pub fn orbiting(&self) -> usize {
        self.bodies.iter().filter(|b| b.orbiting()).count()
    }

    fn accelerations(&self) -> Vec<Vec2> {
        let mut result: Vec<Vec2> = self
            .bodies
            .iter()
            .map(|b| primary_acceleration(b.position))
            .collect();
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                let delta = self.bodies[j].position - self.bodies[i].position;
                result[i] += acceleration(delta, self.bodies[j].class.mass());
                result[j] -= acceleration(delta, self.bodies[i].class.mass());
            }
        }
        result
    }

    pub fn step(&mut self, dt: f32) -> Vec<Impact> {
        self.time += dt;
        let old: Vec<Vec2> = self.bodies.iter().map(|b| b.position).collect();
        // Kick-drift-kick (velocity Verlet): stable orbital energy at a fixed step.
        let forces = self.accelerations();
        for (body, force) in self.bodies.iter_mut().zip(forces) {
            body.velocity += force * (dt * 0.5);
            body.position += body.velocity * dt;
        }
        let forces = self.accelerations();
        for (body, force) in self.bodies.iter_mut().zip(forces) {
            body.velocity += force * (dt * 0.5);
        }

        let mut dead = vec![false; self.bodies.len()];
        for (i, body) in self.bodies.iter().enumerate() {
            dead[i] |=
                segment_distance(old[i], body.position) <= PRIMARY_RADIUS + body.class.radius();
            for j in (i + 1)..self.bodies.len() {
                let other = &self.bodies[j];
                if segment_distance(old[i] - old[j], body.position - other.position)
                    <= body.class.radius() + other.class.radius()
                {
                    dead[i] = true;
                    dead[j] = true;
                }
            }
        }

        let mut impacts = Vec::new();
        let mut index = 0;
        self.bodies.retain_mut(|body| {
            let i = index;
            index += 1;
            if dead[i] {
                self.destroyed += 1;
                impacts.push(Impact {
                    position: body.position,
                    class: body.class,
                });
                return false;
            }
            if body.position.length() > ESCAPE_RADIUS {
                self.escaped += 1;
                return false;
            }
            body.age += dt;
            if body.is_bound() {
                let previous = old[i];
                body.angle_traveled += previous
                    .perp_dot(body.position)
                    .atan2(previous.dot(body.position));
            } else {
                body.angle_traveled = 0.0;
            }
            body.trail_clock += dt;
            if body.trail_clock >= 0.055 {
                body.trail_clock = 0.0;
                body.trail.push_back(body.position);
                if body.trail.len() > 160 {
                    body.trail.pop_front();
                }
            }
            true
        });
        self.best = self.best.max(self.orbiting());
        impacts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_orbit_survives_and_is_counted_in_either_direction() {
        for direction in [-1.0, 1.0] {
            let mut sim = Simulation::default();
            let radius = 250.0;
            let speed = circular_speed(radius);
            sim.launch(
                vec2(radius, 0.0),
                vec2(0.0, speed * direction),
                MassClass::Light,
            );
            let period = std::f32::consts::TAU * radius / speed;
            for _ in 0..(period * 2.1 / STEP) as usize {
                sim.step(STEP);
            }
            assert_eq!(sim.orbiting(), 1);
            assert_eq!(sim.bodies[0].laps(), 2);
            assert!((sim.bodies[0].position.length() - radius).abs() < 0.1);
        }
    }

    #[test]
    fn mutual_gravity_conserves_pair_momentum_and_mass_changes_acceleration() {
        let mut sim = Simulation::default();
        sim.launch(vec2(-200.0, 100.0), Vec2::ZERO, MassClass::Light);
        sim.launch(vec2(200.0, 100.0), Vec2::ZERO, MassClass::Heavy);
        let forces = sim.accelerations();
        let a = forces[0] - primary_acceleration(sim.bodies[0].position);
        let b = forces[1] - primary_acceleration(sim.bodies[1].position);
        let light_mass = MassClass::Light.mass();
        let heavy_mass = MassClass::Heavy.mass();
        assert!((a * light_mass + b * heavy_mass).length() < 0.001);
        assert!((a.length() / b.length() - heavy_mass / light_mass).abs() < 0.01);
    }

    #[test]
    fn nearby_light_objects_visibly_deflect_each_others_paths() {
        let mut pair = Simulation::default();
        let mut left_alone = Simulation::default();
        let mut right_alone = Simulation::default();
        let left = vec2(-35.0, 300.0);
        let right = vec2(35.0, 300.0);
        let velocity = vec2(-circular_speed(300.0), 0.0);
        pair.launch(left, velocity, MassClass::Light);
        pair.launch(right, velocity, MassClass::Light);
        left_alone.launch(left, velocity, MassClass::Light);
        right_alone.launch(right, velocity, MassClass::Light);

        // Compare a real second of mutual gravity with the same launches under
        // primary-only gravity. This catches a force that is present but too
        // weak to produce an observable change in the actual integration.
        for _ in 0..120 {
            pair.step(STEP);
            left_alone.step(STEP);
            right_alone.step(STEP);
        }
        assert_eq!(pair.bodies.len(), 2);
        let left_deflection = pair.bodies[0].position - left_alone.bodies[0].position;
        let right_deflection = pair.bodies[1].position - right_alone.bodies[0].position;
        assert!(
            left_deflection.x > 3.0,
            "left deflection: {left_deflection:?}"
        );
        assert!(
            right_deflection.x < -3.0,
            "right deflection: {right_deflection:?}"
        );
    }

    #[test]
    fn impact_destroys_both_bodies() {
        let mut sim = Simulation::default();
        sim.launch(vec2(200.0, 0.0), vec2(20.0, 0.0), MassClass::Light);
        sim.launch(vec2(209.0, 0.0), vec2(-20.0, 0.0), MassClass::Light);
        assert_eq!(sim.step(STEP).len(), 2);
        assert!(sim.bodies.is_empty());
        assert_eq!(sim.destroyed, 2);
    }

    #[test]
    fn fast_projectile_cannot_tunnel_through_indestructible_primary() {
        let mut sim = Simulation::default();
        sim.launch(vec2(-100.0, 0.0), vec2(24000.0, 0.0), MassClass::Light);
        sim.step(STEP);
        assert!(sim.bodies.is_empty());
        assert_eq!(sim.destroyed, 1);
        assert_eq!(PRIMARY_MASS, 2000.0);
    }

    #[test]
    fn escape_is_not_scored_as_an_orbit() {
        let mut sim = Simulation::default();
        sim.launch(vec2(1799.0, 0.0), vec2(500.0, 0.0), MassClass::Light);
        sim.step(STEP);
        assert_eq!(sim.escaped, 1);
        assert_eq!(sim.orbiting(), 0);
        assert_eq!(sim.destroyed, 0);
    }
}
