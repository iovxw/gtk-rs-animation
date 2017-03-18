use std::cell::RefCell;

use super::Repeat;
use super::Animate;

pub struct AndThen<A, B, F>
    where A: Animate,
          B: Animate,
          F: Fn() -> B + 'static
{
    animator: A,
    and_then: F,
    and_then_animator: RefCell<Option<B>>,
}

impl<A, B, F> AndThen<A, B, F>
    where A: Animate,
          B: Animate,
          F: Fn() -> B + 'static
{
    pub fn new(animate: A, and_then: F) -> AndThen<A, B, F> {
        AndThen {
            animator: animate,
            and_then: and_then,
            and_then_animator: RefCell::new(None),
        }
    }
}

impl<A, B, F> Animate for AndThen<A, B, F>
    where A: Animate,
          B: Animate,
          F: Fn() -> B + 'static
{
    fn start(&self) {
        self.animator.start();
    }
    fn pause(&self) {
        self.animator.pause();
    }
    fn set_repeat(&self, repeat: Repeat) {
        self.animator.set_repeat(repeat);
    }
    fn reset(&self) {
        self.animator.reset();
    }
    fn finish(&self) {
        self.animator.finish();
    }
    fn reverse(&self, on: bool) {
        self.animator.reverse(on);
    }
    fn is_running(&self) -> bool {
        self.animator.is_running()
    }
    fn is_reversing(&self) -> bool {
        self.animator.is_reversing()
    }
    fn one_frame(&self) -> bool {
        if self.and_then_animator.borrow().is_none() {
            let c = self.animator.one_frame();
            if !c {
                *self.and_then_animator.borrow_mut() = Some((self.and_then)());
                self.one_frame()
            } else {
                c
            }
        } else {
            self.and_then_animator.borrow().as_ref().unwrap().one_frame()
        }
    }
}
