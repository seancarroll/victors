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

// #[derive(Clone)]
pub struct InMemoryPublisher<R: Clone + PartialEq, CB>
    where CB: FnOnce(&ExperimentResult<R>) + Copy
{
    phantom: PhantomData<R>,
    // pub result: RefCell<Option<ExperimentResult<R>>>,
    pub cb: CB
}

impl<R: Clone + PartialEq, CB> InMemoryPublisher<R, CB>
    where CB: FnOnce(&ExperimentResult<R>) + Copy
{
    pub fn new(block: CB) -> Self {
        Self {
            // result: RefCell::new(None),
            phantom: PhantomData,
            cb: block
        }
    }
}

// impl<R: Clone + PartialEq> Default for InMemoryPublisher<R> {
//     fn default() -> Self {
//         Self {
//             result: RefCell::new(None)
//         }
//     }
// }

impl<R: Clone + PartialEq, CB> Publisher<R> for InMemoryPublisher<R, CB>
    where CB: FnOnce(&ExperimentResult<R>) + Copy
{
    fn publish(&self, result: &ExperimentResult<R>) {
        // TODO: see if there is a way to remove this clone
        // self.result.swap(&RefCell::new(Some(result.clone())));
        (self.cb)(result);
    }
}

// pub(crate) struct PassthroughPublisher<'a, R: Clone + PartialEq> {
//     pub(crate) published_result: Box<ExperimentResult<'a, R>>
// }
// impl<R: Clone + PartialEq> Publisher<R> for PassthroughPublisher< R> {
//     fn publish(&mut self, result: ExperimentResult<R>) {
//         self.published_result = Box::from(result);
//     }
// }
