use std::rc::Rc;
use std::cell::RefMut;

use std::cell::Cell;

use super::*;

pub struct AndThenInner<A, B> {
    animator: A,
    and_then: B,
    reverse: Cell<bool>,
    then: Cell<bool>,
}

pub struct AndThen<A, B> {
    inner: Rc<AndThenInner<A, B>>,
}

impl<A, B> Clone for AndThen<A, B> {
    fn clone(&self) -> Self {
        AndThen { inner: self.inner.clone() }
    }
}

impl<A, B> AndThen<A, B>
    where A: Animate,
          B: Animate
{
    pub fn new(animate: A, and_then: B) -> AndThen<A, B> {
        AndThen {
            inner: Rc::new(AndThenInner {
                animator: animate,
                and_then: and_then,
                reverse: Cell::new(false),
                then: Cell::new(false),
            }),
        }
    }
}

// FIXME: 独立实现一个 State
impl<A, B> Animate for AndThen<A, B>
    where A: Animate,
          B: Animate,
{
    fn start(&self) {
        if !self.is_running() {
            if self.is_finished() {
                self.inner.then.set(false);
            }

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
    fn reset(&self) {
        // FIXME: state.reset()
        if self.inner.then.get() {
            self.get_state().timer.reset();
            self.one_frame();
            self.inner.then.set(false);
        }
        self.get_state().timer.reset();
        self.one_frame();
    }
    fn finish(&self) {
        self.inner.animator.finish();
        self.inner.then.set(true);
        self.inner.and_then.finish();
    }
    fn reverse(&self, on: bool) {
        self.inner.reverse.set(on);
        self.inner.animator.reverse(on);
        self.inner.and_then.reverse(on);
    }
    fn get_state(&self) -> RefMut<State> {
        if !self.inner.then.get() {
            self.inner.animator.get_state()
        } else {
            self.inner.and_then.get_state()
        }
    }
    fn one_frame(&self) -> bool {
        if !self.inner.then.get() {
            let c = self.inner.animator.one_frame();
            if !c && !self.inner.reverse.get() {
                self.inner.then.set(true);
                self.get_state().timer.run();
                self.one_frame()
            } else {
                c
            }
        } else {
            let c = self.inner.and_then.one_frame();
            if !c && self.inner.reverse.get() {
                self.inner.then.set(false);
                self.get_state().timer.run();
                self.one_frame()
            } else {
                c
            }
        }
    }
}
