extern crate gtk;

use std::mem;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Instant, Duration, SystemTime};

use gtk::prelude::*;

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
    progress: Box<Fn(f64, f64)>,
    easing: Box<Fn(f64) -> f64>,
    repeat: Repeat,
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
                progress: Box::new(progress),
                easing: Box::new(easing),
                repeat: repeat,
            })),
        }
    }

    pub fn start(&self) {
        let mut animator = self.inner.borrow_mut();
        if !animator.is_running() {
            animator.started_time = SystemTime::now() - animator.running_time;

            mem::drop(animator);
            self.start_inner();
        }
    }

    pub fn pause(&self) {
        let mut animator = self.inner.borrow_mut();
        if animator.is_running() {
            animator.state = State::Paused;
        }
    }

    pub fn set_repeat(&self, repeat: Repeat) {
        self.inner.borrow_mut().repeat = repeat;
    }

    pub fn reset(&self) {
        self.inner.borrow_mut().reset()
    }

    pub fn finish(&self) {
        self.inner.borrow_mut().finish()
    }

    pub fn reverse(&self, on: bool) {
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

    pub fn start_inner(&self) {
        self.inner.borrow_mut().state = State::Running;

        let mut fps_time = Instant::now();
        let s = self.clone();

        let mut fps_counter = 0;
        let mut fps_sec = Instant::now();

        gtk::timeout_add(1, move || {
            let mut animator = s.inner.borrow_mut();
            if !animator.is_running() {
                return Continue(false);
            }
            if fps_time.elapsed().subsec_nanos() < 1_000_000_000 / MAX_FPS {
                return Continue(true);
            } else {
                fps_time = Instant::now();

                fps_counter += 1;
                if fps_sec.elapsed().as_secs() > 0 {
                    println!("fps: {}", fps_counter);
                    fps_counter = 0;
                    fps_sec = Instant::now();
                }
            }

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
                    s.start();
                }
            }

            Continue(continue_animation && s.inner.borrow().is_running())
        });
    }

    pub fn is_running(&self) -> bool {
        self.inner.borrow().state.is_running()
    }

    pub fn is_reversing(&self) -> bool {
        self.inner.borrow().reverse
    }
}

fn to_millisecond(t: Duration) -> u64 {
    (t.as_secs() * 1_000) + (t.subsec_nanos() / 1_000_000) as u64
}
