use rand::Rng;

pub mod cartpole;
pub mod pcgrng;

/// The [Score] is the value representing the performances of
/// a [Program] on an episode (or various episodes).
pub type Score = f32;

/// The [State] are 4 values that represent the current state
/// of an environment.
pub type State = [f32; 4];

pub trait Environment {
    fn step<R: Rng>(&mut self, action: f32, rng: &mut R) -> (State, Score, bool);
    fn reset<R: Rng>(&mut self, rng: &mut R) -> State;
}
