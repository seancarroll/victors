use crate::experiment_result::ExperimentResult;

pub trait Publisher<R: Clone> {

    fn publish(result: ExperimentResult<R>);

}

pub struct Otel;
impl<R: Clone> Publisher<R> for Otel {
    fn publish(result: ExperimentResult<R>) {
        todo!()
    }
}
