use std::rc::Rc;
use std::cell::RefMut;

use std::cell::Cell;

use super::*;

pub struct AndThenInner<A, B> {
    animator: A,
    and_then: B,
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
                then: Cell::new(false),
            }),
        }
    }
}

impl<A, B> Animate for AndThen<A, B>
    where A: Animate,
          B: Animate,
{
    fn finish(&self) {
        self.inner.animator.finish();
    }
    fn reverse(&self, on: bool) {
        self.inner.animator.reverse(on);
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
            if !c {
                self.inner.then.set(true);
                self.one_frame()
            } else {
                c
            }
        } else {
            self.inner.and_then.one_frame()
        }
    }
}
