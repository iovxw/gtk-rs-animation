extern crate gtk;

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Instant, Duration, SystemTime};

use gtk::prelude::*;

const MAX_FPS: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnimatorState {
    Running,
    Paused,
    Stoped,
}

impl AnimatorState {
    fn is_running(&self) -> bool {
        match *self {
            AnimatorState::Running => true,
            _ => false,
        }
    }
}

struct AnimatorInner {
    duration: Cell<Duration>,
    started_time: Cell<SystemTime>,
    running_time: Cell<Duration>,
    state: Cell<AnimatorState>,
    reverse: Cell<bool>,
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
    pub fn new<P, E>(duration: Duration, progress: P, easing: E) -> Animator
        where P: Fn(f64, f64) + 'static,
              E: Fn(f64) -> f64 + 'static
    {
        Animator {
            inner: Rc::new(AnimatorInner {
                duration: Cell::new(duration),
                started_time: Cell::new(SystemTime::now()),
                running_time: Cell::new(Duration::from_millis(0)),
                state: Cell::new(AnimatorState::Stoped),
                reverse: Cell::new(false),
                progress: Box::new(progress),
                easing: Box::new(easing),
            }),
        }
    }

    pub fn start(&self) {
        if !self.is_running() {
            self.inner.started_time.set(SystemTime::now() - self.inner.running_time.get());

            self.start_inner();
        }
    }

    pub fn pause(&self) {
        if self.is_running() {
            self.inner.state.set(AnimatorState::Paused);
        }
    }

    pub fn reverse(&self, on: bool) {
        if on != self.is_reversing() {
            let mut need_start = false;
            if self.is_running() {
                self.pause();
                need_start = true;
            }
            self.inner.reverse.set(on);
            let t = self.inner.running_time.get();
            let d = self.inner.duration.get();
            self.inner.running_time.set(d - t);
            if need_start {
                self.start();
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.inner.state.get().is_running()
    }

    pub fn is_reversing(&self) -> bool {
        self.inner.reverse.get()
    }

    pub fn reset(&self) {
        self.inner.state.set(AnimatorState::Stoped);
        self.inner.running_time.set(Duration::from_millis(0));
        let p = if self.is_reversing() { 1.0 } else { 0.0 };
        (self.inner.progress)((self.inner.easing)(p), p);
    }

    pub fn finish(&self) {
        self.inner.state.set(AnimatorState::Stoped);
        self.inner.running_time.set(self.inner.duration.get());
        let p = if self.is_reversing() { 0.0 } else { 1.0 };
        (self.inner.progress)((self.inner.easing)(p), p);
    }

    pub fn start_inner(&self) {
        self.inner.state.set(AnimatorState::Running);

        let mut fps_time = Instant::now();
        let animator = self.clone();

        let mut fps_counter = 0;
        let mut fps_sec = Instant::now();

        gtk::timeout_add(1, move || {
            if !animator.inner.state.get().is_running() {
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

            let inner = &animator.inner;
            let running_time = (SystemTime::now() - inner.started_time.get().elapsed().unwrap())
                .elapsed()
                .unwrap();
            inner.running_time.set(running_time);
            let p = to_millisecond(running_time) as f64 /
                    to_millisecond(inner.duration.get()) as f64;
            let p = if inner.reverse.get() { 1.0 - p } else { p };
            let continue_animation;

            if p >= 0.0 && p < 1.0 {
                continue_animation = true;
                (inner.progress)((inner.easing)(p), p);
            } else {
                // FIXME: need a event or callback to set `continue_animation`
                continue_animation = false;

                if !continue_animation {
                    animator.finish();
                } else {
                    animator.inner.started_time.set(SystemTime::now());
                    animator.reset();
                }
            }

            Continue(continue_animation && inner.state.get().is_running())
        });
    }
}

fn to_millisecond(t: Duration) -> u64 {
    (t.as_secs() * 1_000) + (t.subsec_nanos() / 1_000_000) as u64
}
