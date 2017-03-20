extern crate gtk;

use std::cell::RefMut;

use gtk::prelude::*;

mod animator;
//mod then;
//mod and_then;

pub use animator::*;
//pub use then::Then;
//pub use and_then::AndThen;

pub trait Animate: Clone + 'static {
    fn one_frame(&self);
    fn get_state(&self) -> RefMut<State>;
    fn start(&self) {
        if !self.is_running() {
            self.get_state().timer.run();

            let s = self.clone();
            gtk::timeout_add(1, move || {
                if !s.get_state().fps.update() {
                    return Continue(true);
                }
                s.one_frame();
                let one_loop_finish = s.get_state().timer.get_rate() >= 1.0;
                if !s.is_running() {
                    Continue(false)
                } else if one_loop_finish {
                    if !s.get_state().repeat.shoud_continue() {
                        s.finish();
                        Continue(false)
                    } else {
                        s.get_state().timer.restart();
                        Continue(true)
                    }
                } else {
                    Continue(true)
                }
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
    fn finish(&self) {
        self.get_state().timer.finish();
        self.one_frame();
    }
    fn reverse(&self, on: bool) {
        if on != self.is_reversing() {
            self.get_state().timer.reverse();
            self.get_state().reverse = on;
        }
    }
    fn is_running(&self) -> bool {
        self.get_state().timer.is_running()
    }
    fn is_finished(&self) -> bool {
        self.get_state().timer.is_finished()
    }
    fn is_reversing(&self) -> bool {
        self.get_state().reverse
    }
    //fn then<F>(self, f: F) -> Then<Self, F>
    //    where F: Fn() + 'static,
    //          Self: Sized
    //{
    //    Then::new(self, f)
    //}
    //fn and_then<B>(self, and_then: B) -> AndThen<Self, B>
    //    where B: Animate,
    //          Self: Sized
    //{
    //    AndThen::new(self, and_then)
    //}
}
