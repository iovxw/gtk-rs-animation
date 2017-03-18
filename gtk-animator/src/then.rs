use super::Repeat;
use super::Animate;

pub struct Then<A, F>
    where A: Animate,
          F: Fn() + 'static
{
    animator: A,
    then: F,
}

impl<A, F> Then<A, F>
    where A: Animate,
          F: Fn() + 'static
{
    pub fn new(animate: A, then: F) -> Then<A, F> {
        Then {
            animator: animate,
            then: then,
        }
    }
}

impl<A, F> Animate for Then<A, F>
    where A: Animate,
          F: Fn() + 'static
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
        let c = self.animator.one_frame();
        if !c {
            (self.then)();
        }
        c
    }
}
