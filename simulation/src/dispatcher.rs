use proposed_traits::service::{Interruptible, Service};

pub trait Dispatcher<S: Service + Interruptible> {
    /// Runs the dispatcher, handling both service requests and interrupts.
    ///
    /// `irq_mask` is a bitmask of notification bits this task is willing to receive as interrupts.
    fn run_with_interrupts(&mut self, service: S);
}
