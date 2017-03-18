#![feature(conservative_impl_trait)]

extern crate gtk;

use std::mem;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Instant, Duration, SystemTime};

use gtk::prelude::*;

mod then;
mod and_then;
pub use then::Then;
pub use and_then::AndThen;

const MAX_FPS: u32 = 60;

pub enum Repeat {
    Count(u32),
    Indefinite,
    Function(Box<Fn() -> bool>),
}

impl Repeat {
    pub fn function<F>(f: F) -> Repeat
        where F: Fn() -> bool + 'static
    {
        Repeat::Function(Box::new(f))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Running,
    Paused,
    Stoped,
}

impl State {
    fn is_running(&self) -> bool {
        match *self {
            State::Running => true,
            _ => false,
        }
    }
}

struct AnimatorInner {
    duration: Duration,
    started_time: SystemTime,
    running_time: Duration,
    state: State,
    reverse: bool,
    fps_instant: Instant,
    fps_counter: u32,
    fps_second: Instant,
    progress: Box<Fn(f64, f64)>,
    easing: Box<Fn(f64) -> f64>,
    repeat: Repeat,
    then: Option<Box<Fn()>>,
    and_then: Option<Box<Fn() -> Animator>>,
}

impl AnimatorInner {
    fn is_running(&self) -> bool {
        self.state.is_running()
    }

    fn is_reversing(&self) -> bool {
        self.reverse
    }

    fn reset(&mut self) {
        self.state = State::Stoped;
        self.running_time = Duration::from_millis(0);
        let p = if self.is_reversing() { 1.0 } else { 0.0 };
        (self.progress)((self.easing)(p), p);
    }

    fn finish(&mut self) {
        self.state = State::Stoped;
        self.running_time = self.duration;
        let p = if self.is_reversing() { 0.0 } else { 1.0 };
        (self.progress)((self.easing)(p), p);
    }

    fn continue_repeat(&mut self) -> bool {
        match self.repeat {
            Repeat::Count(ref mut n) => {
                if *n == 0 {
                    // animator is restart manually
                    return false;
                }
                *n -= 1;
                if *n == 0 { false } else { true }
            }
            Repeat::Indefinite => false,
            Repeat::Function(ref f) => f(),
        }
    }
}

pub trait Animate {
    fn start(&self);
    fn pause(&self);
    fn set_repeat(&self, repeat: Repeat);
    fn reset(&self);
    fn finish(&self);
    fn reverse(&self, on: bool);
    fn is_running(&self) -> bool;
    fn is_reversing(&self) -> bool;
    fn one_frame(&self) -> bool;
    fn then<F>(self, f: F) -> Then<Self, F>
        where F: Fn() + 'static,
              Self: Sized
    {
        Then::new(self, f)
    }
    fn and_then<F, B>(self, f: F) -> AndThen<Self, B, F>
        where F: Fn() -> B + 'static,
              B: Animate,
              Self: Sized
    {
        AndThen::new(self, f)
    }
}

pub struct Animator {
    inner: Rc<RefCell<AnimatorInner>>,
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
            inner: Rc::new(RefCell::new(AnimatorInner {
                duration: duration,
                started_time: SystemTime::now(),
                running_time: Duration::from_millis(0),
                state: State::Stoped,
                reverse: false,
                fps_instant: Instant::now(),
                fps_counter: 0,
                fps_second: Instant::now(),
                progress: Box::new(progress),
                easing: Box::new(easing),
                repeat: repeat,
                then: None,
                and_then: None,
            })),
        }
    }
}

impl Animate for Animator {
    fn start(&self) {
        let mut animator = self.inner.borrow_mut();
        if !animator.is_running() {
            animator.started_time = SystemTime::now() - animator.running_time;
            animator.fps_instant = Instant::now();
            animator.fps_counter = 0;
            animator.fps_second = Instant::now();
            animator.state = State::Running;

            mem::drop(animator);
            let s = self.clone();
            gtk::timeout_add(1, move || {
                let mut animator = s.inner.borrow_mut();
                if animator.fps_instant.elapsed().subsec_nanos() < 1_000_000_000 / MAX_FPS {
                    return Continue(true);
                } else {
                    animator.fps_instant = Instant::now();

                    animator.fps_counter += 1;
                    if animator.fps_second.elapsed().as_secs() > 0 {
                        println!("fps: {}", animator.fps_counter);
                        animator.fps_counter = 0;
                        animator.fps_second = Instant::now();
                    }
                }
                mem::drop(animator);
                let c = s.one_frame();
                if !c {
                    for f in &s.inner.borrow().then {
                        f();
                    }
                    for f in &s.inner.borrow().and_then {
                        f();
                    }
                }
                Continue(c)
            });
        }
    }

    fn pause(&self) {
        let mut animator = self.inner.borrow_mut();
        if animator.is_running() {
            animator.state = State::Paused;
        }
    }

    fn set_repeat(&self, repeat: Repeat) {
        self.inner.borrow_mut().repeat = repeat;
    }

    fn reset(&self) {
        self.inner.borrow_mut().reset()
    }

    fn finish(&self) {
        self.inner.borrow_mut().finish()
    }

    fn reverse(&self, on: bool) {
        if on != self.inner.borrow().is_reversing() {
            let mut need_start = false;
            if self.inner.borrow().is_running() {
                self.pause();
                need_start = true;
            }
            let mut animator = self.inner.borrow_mut();
            animator.reverse = on;
            animator.running_time = animator.duration - animator.running_time;
            mem::drop(animator);
            if need_start {
                self.start();
            }
        }
    }

    fn is_running(&self) -> bool {
        self.inner.borrow().state.is_running()
    }

    fn is_reversing(&self) -> bool {
        self.inner.borrow().reverse
    }

    fn one_frame(&self) -> bool {
        if !self.is_running() {
            return false;
        }
        let mut animator = self.inner.borrow_mut();

        let running_time = (SystemTime::now() - animator.started_time.elapsed().unwrap())
            .elapsed()
            .unwrap();
        animator.running_time = running_time;
        let p = to_millisecond(running_time) as f64 / to_millisecond(animator.duration) as f64;
        let p = if animator.reverse { 1.0 - p } else { p };
        let continue_animation;

        if p >= 0.0 && p < 1.0 {
            continue_animation = true;
            (animator.progress)((animator.easing)(p), p);
            mem::drop(animator);
        } else {
            continue_animation = animator.continue_repeat();

            if !continue_animation {
                animator.finish();
                mem::drop(animator);
            } else {
                animator.started_time = SystemTime::now();
                animator.reset();
                mem::drop(animator);
                self.start();
            }
        }

        continue_animation && self.inner.borrow().is_running()
    }
}

fn to_millisecond(t: Duration) -> u64 {
    (t.as_secs() * 1_000) + (t.subsec_nanos() / 1_000_000) as u64
}
