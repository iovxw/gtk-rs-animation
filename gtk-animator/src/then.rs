use std::rc::Rc;
use std::cell::RefMut;

use super::*;

pub struct ThenInner<A, F> {
    animator: A,
    then: F,
}

pub struct Then<A, F> {
    inner: Rc<ThenInner<A, F>>,
}

impl<A, F> Clone for Then<A, F> {
    fn clone(&self) -> Self {
        Then { inner: self.inner.clone() }
    }
}

impl<A, F> Then<A, F>
    where A: Animate,
          F: Fn() + 'static
{
    pub fn new(animate: A, then: F) -> Then<A, F> {
        Then {
            inner: Rc::new(ThenInner {
                animator: animate,
                then: then,
            }),
        }
    }
}

impl<A, F> Animate for Then<A, F>
    where A: Animate,
          F: Fn() + 'static
{
    fn finish(&self) {
        self.inner.animator.finish();
    }
    fn reverse(&self, on: bool) {
        self.inner.animator.reverse(on);
    }
    fn get_state(&self) -> RefMut<State> {
        self.inner.animator.get_state()
    }
    fn one_frame(&self) -> bool {
        let c = self.inner.animator.one_frame();
        if !c {
            (self.inner.then)();
        }
        c
    }
}
