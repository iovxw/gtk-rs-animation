extern crate gtk;

use std::cell::RefMut;

use gtk::prelude::*;

mod animator;
mod then;
mod and_then;

pub use animator::*;
pub use then::Then;
pub use and_then::AndThen;

pub trait Animate: Clone + 'static {
    fn one_frame(&self) -> bool;
    fn get_state(&self) -> RefMut<State>;
    fn start(&self) {
        if !self.is_running() {
            self.get_state().timer.run();

            let s = self.clone();
            gtk::timeout_add(1, move || {
                if !s.get_state().fps.update() {
                    return Continue(true);
                }
                Continue(s.one_frame())
            });
        }
    }
    fn pause(&self) {
        self.get_state().timer.pause();
    }
    fn set_repeat(&self, repeat: Repeat) {
        self.get_state().repeat = repeat;
    }
    fn reset(&self) {
        self.get_state().timer.reset();
        self.one_frame();
    }
    fn finish(&self);
    fn reverse(&self, on: bool);
    fn is_running(&self) -> bool {
        self.get_state().timer.is_running()
    }
    fn is_reversing(&self) -> bool {
        self.get_state().reverse
    }
    fn then<F>(self, f: F) -> Then<Self, F>
        where F: Fn() + 'static,
              Self: Sized
    {
        Then::new(self, f)
    }
    fn and_then<B>(self, and_then: B) -> AndThen<Self, B>
        where B: Animate,
              Self: Sized
    {
        AndThen::new(self, and_then)
    }
}
