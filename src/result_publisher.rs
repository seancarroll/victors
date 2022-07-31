use crate::experiment_result::ExperimentResult;

pub trait Publisher<R: Clone + PartialEq> {

    fn publish(result: ExperimentResult<R>);

}

pub struct Otel;
impl<R: Clone + PartialEq> Publisher<R> for Otel {
    fn publish(result: ExperimentResult<R>) {
        todo!()
    }
}
