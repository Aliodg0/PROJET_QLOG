use std::f32::consts::PI;

use rand::Rng;
use rand_distr::Distribution;
use rand_distr::Uniform;

use crate::{Environment, Score, State, pcgrng::PCGRng};

pub struct CartPole {
    /// Gravity force
    g: f32,
    /// Cart mass
    mass_cart: f32,
    /// Pole mass
    mass_pole: f32,
    /// Half length of the pole
    half_length: f32,
    /// Seconds between state update
    tau: f32,
    force_mag: f32,

    // State
    x: f32,
    x_dot: f32,
    theta: f32,
    theta_dot: f32,

    // Threshold indicating when to stop the episode
    threshold_x: f32,
    threshold_theta: f32,
    episode_length: usize,
    episode_done: bool,
}

impl Environment for CartPole {
    fn step<R>(&mut self, action: f32, _: &mut R) -> (State, Score, bool)
    where
        R: Rng,
    {
        if self.episode_done {
            return ([self.x, self.x_dot, self.theta, self.theta_dot], 0., true);
        }

        let force = if action > 0. {
            self.force_mag
        } else {
            -self.force_mag
        };

        let sin = self.theta.sin();
        let cos = self.theta.cos();
        let mp = self.mass_pole;
        let mc = self.mass_cart;
        let l = self.half_length;
        let t_d = self.theta_dot;
        let g = self.g;

        let temp = (force + mp * l * t_d * t_d * sin) / (mc + mp);
        let t_dd = (g * sin - cos * temp) / (l * (4. / 3. - (mp * cos * cos) / (mc + mp)));
        let x_dd = temp - mp * t_dd * cos / (mc + mp);

        self.x += self.tau * self.x_dot;
        self.x_dot += self.tau * x_dd;
        self.theta += self.tau * self.theta_dot;
        self.theta_dot += self.tau * t_dd;

        self.episode_length += 1;

        let done_x = self.x < -self.threshold_x || self.x > self.threshold_x;
        let done_theta = self.theta < -self.threshold_theta || self.theta > self.threshold_theta;
        let done = done_x || done_theta || self.episode_length >= 500;
        self.episode_done = done;

        ([self.x, self.x_dot, self.theta, self.theta_dot], 1., done)
    }

    fn reset<R>(&mut self, mut rng: &mut R) -> State
    where
        R: Rng,
    {
        let dist = Uniform::new(-0.05f32, 0.05f32).unwrap();
        self.x = dist.sample(&mut rng);
        self.x_dot = dist.sample(&mut rng);
        self.theta = dist.sample(&mut rng);
        self.theta_dot = dist.sample(&mut rng);
        self.episode_length = 0;
        self.episode_done = false;
        [self.x, self.x_dot, self.theta, self.theta_dot]
    }
}

impl Default for CartPole {
    fn default() -> Self {
        let dist = Uniform::new(-0.05f32, 0.05f32).unwrap();
        let mut rng = PCGRng::seed(0);
        CartPole {
            g: 9.8,
            mass_cart: 1.,
            mass_pole: 0.1,
            half_length: 0.5,
            tau: 0.02,
            force_mag: 10.,
            x: dist.sample(&mut rng),
            x_dot: dist.sample(&mut rng),
            theta: dist.sample(&mut rng),
            theta_dot: dist.sample(&mut rng),
            threshold_x: 2.4,
            threshold_theta: 12. * 2. * PI / 360.,
            episode_done: false,
            episode_length: 0,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pcgrng::PCGRng;

    #[test]
    fn reset_initialises_bounded_state() {
        let mut rng = PCGRng::seed(42);
        let mut env = CartPole::default();
        let state = env.reset(&mut rng);
        for value in state {
            assert!(value.is_finite());
            assert!(value.abs() < 1.0);
        }
    }

    #[test]
    fn episodes_are_deterministic_for_a_seed() {
        let mut first_rng = PCGRng::seed(9);
        let mut env1 = CartPole::default();
        let _ = env1.reset(&mut first_rng);
        let mut total1 = 0.0f32;
        loop {
            let (_next, reward, done) = env1.step(1.0, &mut first_rng);
            total1 += reward;
            if done {
                break;
            }
        }

        let mut second_rng = PCGRng::seed(9);
        let mut env2 = CartPole::default();
        let _ = env2.reset(&mut second_rng);
        let mut total2 = 0.0f32;
        loop {
            let (_next, reward, done) = env2.step(1.0, &mut second_rng);
            total2 += reward;
            if done {
                break;
            }
        }
        assert_eq!(total1, total2);
    }

    #[test]
    fn episode_ends_after_limit() {
        let mut rng = PCGRng::seed(1);
        let mut env = CartPole::default();
        env.reset(&mut rng);
        let mut steps = 0usize;
        loop {
            let (_, _, done) = env.step(0.0, &mut rng);
            steps += 1;
            if done {
                break;
            }
        }
        assert!(steps <= 500);
    }

    #[test]
    fn terminal_state_is_stable() {
        let mut rng = PCGRng::seed(11);
        let mut env = CartPole::default();
        env.reset(&mut rng);

        // Move until the episode ends.
        while !env.episode_done {
            let _ = env.step(0.0, &mut rng);
        }

        let state = [env.x, env.x_dot, env.theta, env.theta_dot];
        let (_, reward, done) = env.step(1.0, &mut rng);
        assert!(done);
        assert_eq!(reward, 0.0);
        assert_eq!([env.x, env.x_dot, env.theta, env.theta_dot], state);
    }

    #[test]
    fn reset_restarts_after_terminal() {
        let mut rng = PCGRng::seed(21);
        let mut env = CartPole::default();
        env.reset(&mut rng);

        while !env.episode_done {
            let _ = env.step(0.0, &mut rng);
        }

        let restarted = env.reset(&mut rng);
        assert!(!env.episode_done);
        assert_eq!(restarted, [env.x, env.x_dot, env.theta, env.theta_dot]);
    }
}
