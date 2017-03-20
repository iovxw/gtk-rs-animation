use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::time::{Instant, Duration};

use super::*;

const MAX_FPS: u32 = 60;

pub struct FPS {
    max: u32,
    instant: Instant,
    counter: u32,
    second_timer: Instant,
}

impl FPS {
    pub fn new(max: u32) -> FPS {
        FPS {
            max: max,
            instant: Instant::now(),
            counter: 0,
            second_timer: Instant::now(),
        }
    }

    pub fn update(&mut self) -> bool {
        if self.instant.elapsed().subsec_nanos() < (1_000_000_000 / self.max) {
            false
        } else {
            self.counter += 1;
            self.instant = Instant::now();
            if self.second_timer.elapsed().as_secs() > 0 {
                println!("fps: {}", self.counter);
                self.counter = 0;
                self.second_timer = Instant::now();
            }
            true
        }
    }
}

pub enum Repeat {
    Count(u32, u32),
    Indefinite,
    Function(Box<Fn() -> bool>),
}

impl Repeat {
    pub fn count(n: u32) -> Repeat {
        Repeat::Count(n, n)
    }

    pub fn function<F>(f: F) -> Repeat
        where F: Fn() -> bool + 'static
    {
        Repeat::Function(Box::new(f))
    }

    pub fn shoud_continue(&mut self) -> bool {
        match *self {
            Repeat::Count(ref mut n, ref mut current) => {
                if *current == 0 {
                    *current = *n;
                }
                *current -= 1;
                if *current == 0 { false } else { true }
            }
            Repeat::Indefinite => false,
            Repeat::Function(ref f) => f(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerState {
    Running(Instant),
    Paused(Duration),
    Unstart,
    Finish,
}

pub struct Timer {
    state: TimerState,
    duration: Duration,
}

impl Timer {
    pub fn new(duration: Duration) -> Timer {
        Timer {
            state: TimerState::Unstart,
            duration: duration,
        }
    }

    pub fn run(&mut self) {
        match self.state {
            TimerState::Paused(d) => {
                self.state = TimerState::Running(Instant::now() - d);
            }
            _ => {
                self.state = TimerState::Running(Instant::now());
            }
        }
    }

    pub fn pause(&mut self) {
        match self.state {
            TimerState::Running(t) => {
                self.state = TimerState::Paused(t.elapsed());
            }
            _ => (),
        }
    }

    pub fn reset(&mut self) {
        self.state = TimerState::Unstart;
    }

    pub fn finish(&mut self) {
        self.state = TimerState::Finish;
    }

    pub fn reverse(&mut self) {
        match self.state {
            TimerState::Paused(ref mut d) => {
                *d = self.duration - *d;
            }
            TimerState::Running(ref mut t) => {
                *t = Instant::now() - (self.duration - t.elapsed());
            }
            TimerState::Finish => self.reset(),
            TimerState::Unstart => self.finish(),
        }
    }

    pub fn restart(&mut self) {
        self.state = TimerState::Running(Instant::now());
    }

    pub fn get_duration(&self) -> Duration {
        match self.state {
            TimerState::Running(t) => t.elapsed(),
            TimerState::Paused(d) => d,
            TimerState::Unstart => Duration::from_secs(0),
            TimerState::Finish => self.duration,
        }
    }

    pub fn get_target_duration(&self) -> Duration {
        self.duration
    }

    pub fn get_rate(&self) -> f64 {
        to_millisecond(self.get_duration()) as f64 /
        to_millisecond(self.get_target_duration()) as f64
    }

    pub fn is_running(&self) -> bool {
        match self.state {
            TimerState::Running(_) => true,
            _ => false,
        }
    }

    pub fn is_finished(&mut self) -> bool {
        match self.state {
            TimerState::Finish => true,
            _ => false,
        }
    }
}

pub struct State {
    pub timer: Timer,
    pub fps: FPS,
    pub repeat: Repeat,
    pub reverse: bool,
}

impl State {
    pub fn new(repeat: Repeat, duation: Duration) -> State {
        State {
            timer: Timer::new(duation),
            fps: FPS::new(MAX_FPS),
            repeat: repeat,
            reverse: false,
        }
    }
}

struct AnimatorInner {
    state: Rc<RefCell<State>>,
    progress: Box<Fn(f64, f64)>,
    easing: Box<Fn(f64) -> f64>,
}

pub struct Animator {
    inner: Rc<AnimatorInner>,
}

// Must implement `Clone` manually, see:
// http://stackoverflow.com/a/31676926
impl Clone for Animator {
    fn clone(&self) -> Self {
        Animator { inner: self.inner.clone() }
    }
}

impl Animator {
    pub fn new<P, E>(duration: Duration, repeat: Repeat, progress: P, easing: E) -> Animator
        where P: Fn(f64, f64) + 'static,
              E: Fn(f64) -> f64 + 'static
    {
        Animator {
            inner: Rc::new(AnimatorInner {
                state: Rc::new(RefCell::new(State::new(repeat, duration))),
                progress: Box::new(progress),
                easing: Box::new(easing),
            }),
        }
    }
}

impl Animate for Animator {
    fn get_state(&self) -> RefMut<State> {
        self.inner.state.borrow_mut()
    }

    fn one_frame(&self) {
        let animator = &self.inner;

        let p = self.get_state().timer.get_rate();
        let p = if self.get_state().reverse { 1.0 - p } else { p };

        (animator.progress)((animator.easing)(p), p);
    }
}

fn to_millisecond(t: Duration) -> u64 {
    (t.as_secs() * 1_000) + (t.subsec_nanos() / 1_000_000) as u64
}
