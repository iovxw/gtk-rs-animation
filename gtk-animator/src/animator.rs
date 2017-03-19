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

    fn shoud_continue(&mut self) -> bool {
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
pub enum Timer {
    Running(Instant),
    Paused(Duration),
    Unstart,
    Finish(Duration),
}

impl Timer {
    pub fn new() -> Timer {
        Timer::Unstart
    }

    pub fn run(&mut self) {
        match *self {
            Timer::Paused(d) => {
                *self = Timer::Running(Instant::now() - d);
            }
            _ => {
                *self = Timer::Running(Instant::now());
            }
        }
    }

    pub fn pause(&mut self) {
        match *self {
            Timer::Running(t) => {
                *self = Timer::Paused(t.elapsed());
            }
            _ => (),
        }
    }

    pub fn reset(&mut self) {
        *self = Timer::Unstart;
    }

    pub fn finish(&mut self, d: Duration) {
        *self = Timer::Finish(d);
    }

    pub fn reverse(&mut self, finish: Duration) {
        match *self {
            Timer::Paused(ref mut d) => {
                *d = finish - *d;
            }
            Timer::Running(ref mut t) => {
                *t = Instant::now() - (finish - t.elapsed());
            }
            Timer::Finish(_) => self.reset(),
            Timer::Unstart => self.finish(finish),
        }
    }

    pub fn restart(&mut self) {
        *self = Timer::Running(Instant::now());
    }

    pub fn get_duration(&self) -> Duration {
        match *self {
            Timer::Running(t) => t.elapsed(),
            Timer::Paused(d) => d,
            Timer::Unstart => Duration::from_secs(0),
            Timer::Finish(d) => d,
        }
    }

    pub fn is_running(&self) -> bool {
        match *self {
            Timer::Running(_) => true,
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
    pub fn new(repeat: Repeat) -> State {
        State {
            timer: Timer::new(),
            fps: FPS::new(MAX_FPS),
            repeat: repeat,
            reverse: false,
        }
    }
}

struct AnimatorInner {
    duration: Duration,
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
                duration: duration,
                state: Rc::new(RefCell::new(State::new(repeat))),
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

    fn one_frame(&self) -> bool {
        let animator = &self.inner;

        let p = to_millisecond(self.get_state().timer.get_duration()) as f64 /
                to_millisecond(animator.duration) as f64;
        let p = if self.get_state().reverse { 1.0 - p } else { p };

        if p >= 0.0 && p <= 1.0 {
            (animator.progress)((animator.easing)(p), p);
            if !self.get_state().timer.is_running() {
                false
            } else {
                true
            }
        } else {
            if !self.get_state().repeat.shoud_continue() {
                self.finish();
                false
            } else {
                self.get_state().timer.restart();
                true
            }
        }
    }

    fn finish(&self) {
        if !self.is_running() {
            self.get_state().timer.finish(self.inner.duration);
            self.one_frame();
        } else {
            self.get_state().timer.finish(self.inner.duration);
        }
    }

    fn reverse(&self, on: bool) {
        if on != self.is_reversing() {
            self.get_state().timer.reverse(self.inner.duration);
            self.get_state().reverse = on;
        }
    }
}

fn to_millisecond(t: Duration) -> u64 {
    (t.as_secs() * 1_000) + (t.subsec_nanos() / 1_000_000) as u64
}
