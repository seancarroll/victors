use std::marker::PhantomData;
use crate::experiment_result::ExperimentResult;

pub trait Publisher<R: Clone + PartialEq> {

    fn publish(&mut self, result: ExperimentResult<R>);

}

pub struct NoopPublisher;
impl<R: Clone + PartialEq> Publisher<R> for NoopPublisher {
    fn publish(&mut self, _result: ExperimentResult<R>) {
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
