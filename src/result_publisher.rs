use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::experiment_result::ExperimentResult;

pub trait Publisher<R: Clone + PartialEq> {

    fn publish(&self, result: &ExperimentResult<R>);

}

// #[derive(Clone)]
pub struct NoopPublisher;
impl<R: Clone + PartialEq> Publisher<R> for NoopPublisher {
    fn publish(&self, _result: &ExperimentResult<R>) {
    }
}

pub struct InMemoryPublisher<R: Clone + PartialEq, CB>
    where CB: FnOnce(&ExperimentResult<R>) + Copy
{
    phantom: PhantomData<R>,
    pub cb: CB
}

impl<R: Clone + PartialEq, CB> InMemoryPublisher<R, CB>
    where CB: FnOnce(&ExperimentResult<R>) + Copy
{
    pub fn new(block: CB) -> Self {
        Self {
            phantom: PhantomData,
            cb: block
        }
    }
}

impl<R: Clone + PartialEq, CB> Publisher<R> for InMemoryPublisher<R, CB>
    where CB: FnOnce(&ExperimentResult<R>) + Copy
{
    fn publish(&self, result: &ExperimentResult<R>) {
        // TODO: see if there is a way to remove this clone
        // self.result.swap(&RefCell::new(Some(result.clone())));
        (self.cb)(result);
    }
}
