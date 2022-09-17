use std::any::Any;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use serde::Serialize;
use crate::{errors::VictorsResult, experiment::{Experiment, UncontrolledExperiment}, Publisher};
use crate::result_publisher::NoopPublisher;

pub trait Scientist<'a, R: Clone + PartialEq + Serialize> {
    type P: Publisher<R> + 'a;

    /// Define and run a controlled experiment.
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    /// * `experiment_block` - Function to configure the experiment
    ///
    /// # Return
    /// Returns the calculated value of the control experiment or error
    fn conduct<F>(name: &'static str, experiment_block: F) -> VictorsResult<R>
    where
        F: Fn(&mut Experiment<'_, R>) -> VictorsResult<()>,
    {
        let mut experiment = Experiment::new(name);
        experiment.result_publisher(Self::get_publisher());
        experiment_block(&mut experiment)?;
        return experiment.run();
    }

    /// Define and run an uncontrolled experiment.
    /// Uncontrolled experiments contain only candidates and do not have a control.
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    /// * `return_candidate_result` - name of the candidate's result to return
    /// * `experiment_block` - Function to configure the experiment
    ///
    /// # Return
    /// Returns the calculated value of the named experiment or error
    fn conduct_uncontrolled<F>(
        name: &'static str,
        return_candidate_result: &'static str,
        experiment_block: F,
    ) -> VictorsResult<R>
        where
            F: Fn(&mut UncontrolledExperiment<'_, R>) -> VictorsResult<()>,
    {
        let mut experiment = UncontrolledExperiment::new(name);
        experiment.result_publisher(Self::get_publisher());
        experiment_block(&mut experiment)?;
        return experiment.run(return_candidate_result);
    }

    fn get_publisher() -> Self::P;

    // TODO: should have err_on_mismatch fn?
}

pub struct Victor;

impl<'a, R: Clone + PartialEq + Serialize> Scientist<'a, R> for Victor {
    type P = NoopPublisher;

    fn get_publisher() -> Self::P {
        return NoopPublisher{};
    }
}

// /// Represents the globally configured [`Publisher`] instance for this application.
// #[derive(Clone)]
// pub struct GlobalResultPublisher<R: Clone + PartialEq> {
//     publisher: Arc<dyn Publisher<R>>,
// }
// impl<R: Clone + PartialEq> GlobalResultPublisher<R> {
//     /// Create a new GlobalResultPublisher instance from a struct that implements `Publisher`.
//     fn new<P>(publisher: P) -> Self
//         where
//             P: Publisher<R> + 'static,
//     {
//         GlobalResultPublisher {
//             publisher: Arc::new(publisher),
//         }
//     }
// }

// static GLOBAL_RESULT_PUBLISHER: once_cell::sync::Lazy<RwLock<GlobalResultPublisher<R>>> = once_cell::sync::Lazy::new(|| {
//     RwLock::new(GlobalResultPublisher::new(
//         NoopPublisher{},
//     ))
// });

pub trait ObjectSafePublisher {
}

impl<T> ObjectSafePublisher for T
where
    T: Clone + PartialEq + Serialize
{

}

pub struct BoxedResultPublisher(Box<dyn Publisher<_> + Clone + PartialEq + Serialize>);
impl BoxedResultPublisher {
    pub(crate) fn new<T>(publisher: T) -> Self
        where
            T: Clone + PartialEq + Serialize + 'static,
    {
        BoxedResultPublisher(Box::new(publisher))
    }
}


/// Represents the globally configured [`Publisher`] instance for this application.
#[derive(Clone)]
pub struct GlobalResultPublisherProvider {
    publisher: Arc<BoxedResultPublisher>,
}

impl GlobalResultPublisherProvider {
    /// Create a new GlobalResultPublisher instance from a struct that implements `Publisher`.
    fn new<R, P>(publisher: P) -> Self
        where
            R: Clone + PartialEq + Serialize,
            P: Publisher<R> + 'static,
    {
        GlobalResultPublisher {
            publisher: Arc::new(publisher),
        }
    }
}

static GLOBAL_RESULT_PUBLISHER: once_cell::sync::Lazy<RwLock<GlobalResultPublisher>> = once_cell::sync::Lazy::new(|| {
    RwLock::new(GlobalResultPublisher::new(
        NoopPublisher{},
    ))
});



// /// Returns an instance of the currently configured global [`TracerProvider`] through
// /// [`GlobalTracerProvider`].
// ///
// /// [`TracerProvider`]: crate::trace::TracerProvider
// /// [`GlobalTracerProvider`]: crate::global::GlobalTracerProvider
// pub fn tracer_provider() -> GlobalTracerProvider {
//     GLOBAL_TRACER_PROVIDER
//         .read()
//         .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned")
//         .clone()
// }

// /// Sets the given [`TracerProvider`] instance as the current global provider.
// ///
// /// It returns the [`TracerProvider`] instance that was previously mounted as global provider
// /// (e.g. [`NoopTracerProvider`] if a provider had not been set before).
// ///
// /// [`TracerProvider`]: crate::trace::TracerProvider
// pub fn set_tracer_provider<P, T, S>(new_provider: P) -> GlobalTracerProvider
//     where
//         S: trace::Span + Send + Sync + 'static,
//         T: trace::Tracer<Span = S> + Send + Sync + 'static,
//         P: trace::TracerProvider<Tracer = T> + Send + Sync + 'static,
// {
//     let mut tracer_provider = GLOBAL_TRACER_PROVIDER
//         .write()
//         .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned");
//     mem::replace(
//         &mut *tracer_provider,
//         GlobalTracerProvider::new(new_provider),
//     )
// }
