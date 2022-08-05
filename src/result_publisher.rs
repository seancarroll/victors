use std::marker::PhantomData;

use crate::experiment_result::ExperimentResult;

pub trait Publisher<R: Clone + PartialEq> {
    fn publish(&self, result: &ExperimentResult<R>);
}

pub struct NoopPublisher;
impl<R: Clone + PartialEq> Publisher<R> for NoopPublisher {
    fn publish(&self, _result: &ExperimentResult<R>) {}
}

pub(crate) struct InMemoryPublisher<R: Clone + PartialEq, CB>
where
    CB: FnOnce(&ExperimentResult<R>) + Copy,
{
    phantom: PhantomData<R>,
    pub cb: CB,
}

impl<R: Clone + PartialEq, CB> InMemoryPublisher<R, CB>
where
    CB: FnOnce(&ExperimentResult<R>) + Copy,
{
    pub fn new(block: CB) -> Self {
        Self {
            phantom: PhantomData,
            cb: block,
        }
    }
}

impl<R: Clone + PartialEq, CB> Publisher<R> for InMemoryPublisher<R, CB>
where
    CB: FnOnce(&ExperimentResult<R>) + Copy,
{
    fn publish(&self, result: &ExperimentResult<R>) {
        (self.cb)(result);
    }
}
