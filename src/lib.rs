#![doc(html_root_url = "https://docs.rs/victors")]
#![doc(issue_tracker_base_url = "https://github.com/seancarroll/victors/issues/")]
#![cfg_attr(docsrs, deny(broken_intra_doc_links))]
#![feature(backtrace)]
#![deny(elided_lifetimes_in_paths)]

pub mod context;
pub mod errors;
pub mod experiment;
pub mod experiment_result;
pub mod observation;
pub mod result_publisher;
pub mod victor;

// TODO: can i use *?
// https://github.com/SeaQL/sea-orm/blob/master/src/lib.rs
pub use crate::{
    context::Context,
    experiment::{Experiment, UncontrolledExperiment},
    experiment_result::ExperimentResult,
    observation::Observation,
    result_publisher::Publisher,
};

#[cfg(test)]
mod tests {
    use bincode;
    use std::{
        cell::{RefCell},
    };
    use std::cell::Ref;
    use std::collections::HashSet;
    use serde::Serialize;

    use serde_json::{json, Value};

    use crate::{
        context::Context,
        errors::{
            BehaviorMissing, BehaviorNotUnique, VictorsErrors, VictorsResult
        },
        experiment::Experiment,
        experiment_result::ExperimentResult,
        observation::Observation,
        Publisher,
        result_publisher::InMemoryPublisher,
        UncontrolledExperiment,
        victor::Victor
    };
    use crate::victor::Scientist;

    #[test]
    fn try_new_dr() {

        pub struct PrintPublisher;
        impl<R: Clone + PartialEq + Serialize> Publisher<R> for PrintPublisher {
            fn publish(&self, result: &ExperimentResult<R>) {
                println!("{}", serde_json::to_string(result).unwrap());
            }
        }

        struct Reed;
        impl<'a, R: Clone + PartialEq + Serialize> Scientist<'a, R> for Reed {
            type P = PrintPublisher;

            fn get_publisher() -> Self::P {
                return PrintPublisher{};
            }
        }

        let r = Reed::conduct("conduct test", |experiment| {
            experiment.control(|| 1)?;
            experiment.candidate(|| 2)?;
            Ok(())
        });

        assert_eq!(Some(1), r.ok());

    }


    #[test]
    fn should_be_able_to_run_experiment_with_only_control() {
        let mut experiment = Experiment::default();
        experiment.control(|| "control").unwrap();

        let result = experiment.run().unwrap();

        assert_eq!("control", result);
    }

    #[test]
    fn should_always_return_control_result() {
        let mut experiment = Experiment::default();
        experiment.control(|| "control").unwrap();
        experiment.candidate(|| "candidate").unwrap();

        let result = experiment.run().unwrap();

        assert_eq!("control", result);
    }

    #[test]
    fn should_be_able_to_specify_control_name() {
        let mut experiment = Experiment::new("custom control");
        experiment.control(|| "value").unwrap();

        assert_eq!("custom control", experiment.name);
    }

    #[test]
    fn should_return_error_when_controlled_experiment_has_no_control_defined() {
        let mut experiment = Experiment::default();
        experiment.candidate(|| 1).unwrap();

        let result = experiment.run();

        let expected = VictorsErrors::BehaviorMissing(BehaviorMissing {
            experiment_name: "experiment".to_string(),
            name: "control".to_string(),
        });
        assert_eq!(expected, result.unwrap_err());
    }

    #[test]
    fn should_return_error_when_uncontrolled_experiment_primary_behavior_is_missing() {
        let mut experiment = UncontrolledExperiment::default();
        experiment.candidate("candidate", || { "value" }).unwrap();

        let result = experiment.run("candidont");

        let expected = VictorsErrors::BehaviorMissing(BehaviorMissing {
            experiment_name: "experiment".to_string(),
            name: "candidont".to_string(),
        });
        assert_eq!(expected, result.unwrap_err());
    }

    #[test]
    fn should_not_run_experiment_when_disabled() {
        let called = RefCell::new(false);

        let mut experiment = Experiment::default();
        experiment.control(|| 1).unwrap();
        experiment.candidate(|| {
            called.replace(true);
            1
        }).unwrap();
        experiment.enabled(|| { false });

        let result = experiment.run().unwrap();

        assert_eq!(1, result);
        assert!(!called.take());
    }

    #[test]
    fn should_allow_to_init_experiment_with_context() {
        let r: RefCell<Option<ExperimentResult<u8>>> = RefCell::new(None);

        let mut experiment = Experiment::new_with_context(
            "experiment",
            Context::from_value(json!({"message": "hello world"})).unwrap(),
        );
        experiment.control(|| { 1 }).unwrap();

        assert_eq!(Context::from_value(json!({"message": "hello world"})).unwrap(), experiment.context);
    }

    #[test]
    fn should_allow_custom_context_to_be_passed_in() {
        let r: RefCell<Option<ExperimentResult<u8>>> = RefCell::new(None);

        let mut experiment = Experiment::default();
        experiment.add_context(Context::from_value(json!({"message": "hello world"})).unwrap());
        experiment.control(|| 1).unwrap();
        experiment.candidate(|| 1).unwrap();
        experiment.result_publisher(InMemoryPublisher::new(|result| {
            r.replace(Some(result.clone()));
        }));

        experiment.run().unwrap();

        assert_eq!(
            Context::from_value(json!({"message": "hello world"})).unwrap(),
            experiment.context
        );
    }

    #[test]
    fn should_shuffle_behaviors_execution_order() {
        let last = RefCell::new("");
        let mut runs = HashSet::new();

        let mut experiment = Experiment::default();
        experiment.control(|| { last.replace("control"); }).unwrap();
        experiment.candidate(|| { last.replace("candidate"); }).unwrap();
        for i in 0..1000 {
            experiment.run().unwrap();
            runs.insert(last.take());
        }

        assert!(runs.len() > 1);
    }


    // TODO: publish
    // TODO: clean values

    // TODO: run_if - run "experiment" means candidates but not sure if we really need to change
    // the language or not
    #[test]
    fn should_not_run_candidates_when_run_if_returns_false() {
        let called = RefCell::new(false);

        let mut experiment = Experiment::default();
        experiment.control(|| 1).unwrap();
        experiment.candidate(|| {
            called.replace(true);
            1
        }).unwrap();
        experiment.run_if(|| false);

        experiment.run().unwrap();

        assert!(!called.take());
    }

    #[test]
    fn should_run_candidates_when_run_if_returns_true() {
        let called = RefCell::new(false);

        let mut experiment = Experiment::default();
        experiment.control(|| 1).unwrap();
        experiment.candidate(|| {
            called.replace(true);
            1
        }).unwrap();
        experiment.run_if(|| true);

        experiment.run().unwrap();

        assert!(called.take());
    }

    // TODO: parallel

    #[test]
    fn should_be_able_to_create_and_run_experiment_via_victor() {
        let r = Victor::conduct("conduct test", |experiment| {
            // To fix the error below means we need to return Ok(()) at the end
            // cannot use the `?` operator in a closure that returns `()`
            // I dont love it but I suppose thats part of the rust idioms
            experiment.control(|| 1)?;
            experiment.candidate(|| 2)?;
            Ok(())
        });

        assert_eq!(Some(1), r.ok());
    }

    #[test]
    fn should_be_able_to_create_and_run_uncontrolled_experiment_via_victor() {
        let r = Victor::conduct_uncontrolled("uncontrolled test", "second", |experiment| {
            // To fix the error below means we need to return Ok(()) at the end
            // cannot use the `?` operator in a closure that returns `()`
            // I dont love it but I suppose thats part of the rust idioms
            experiment.candidate("first", || 1)?;
            experiment.candidate("second", || 2)?;
            Ok(())
        });

        assert_eq!(Some(2), r.ok());
    }

    // ignore ignore_mismatched_observation tests
    // TODO: does not ignore an observation if no ignores are configured
    // TODO: calls a configured ignore block with the given observed values
    #[test]
    fn should_call_ignore_blocks_until_match() {
        // TODO: using RefCell seems like a hack. I can't figure out how to just mutate a boolean
        // captured variable. All three types (Fn, FnOnce, FnMut) are problematic for various reasons
        let (called_one, called_two, called_three) = (RefCell::new(false), RefCell::new(false), RefCell::new(false));
        // let mut called_one = false;

        let mut experiment = Experiment::default();
        experiment.add_ignore(|_a, _b| {
            called_one.replace(true);
            false
        });
        experiment.add_ignore(|_a, _b| {
            called_two.replace(true);
            true
        });
        experiment.add_ignore(|_a, _b| {
            called_three.replace(true);
            false
        });

        let a = create_observation("a");
        let b = create_observation("b");

        assert!(experiment.ignore_mismatch_observation(&a, &b));
        assert!(called_one.take());
        assert!(called_two.take());
        assert!(!called_three.take());
    }

    #[test]
    fn should_return_non_unique_error_with_multiple_controls() {
        let mut experiment = Experiment::default();
        experiment.control(|| "control").unwrap();
        let result = experiment.control(|| "control again");

        let expected = VictorsErrors::BehaviorNotUnique(BehaviorNotUnique {
            experiment_name: "experiment".to_string(),
            name: "control".to_string(),
        });
        assert_eq!(expected, result.unwrap_err());
    }

    #[test]
    fn should_return_non_unique_error_when_multiple_candidates_are_registered_with_same_name() {
        let mut experiment = Experiment::default();
        experiment.control(|| "value").unwrap();
        experiment.candidate(|| "candidate").unwrap();
        let result = experiment.candidate(|| "candidate");

        let expected = VictorsErrors::BehaviorNotUnique(BehaviorNotUnique {
            experiment_name: "experiment".to_string(),
            name: "candidate".to_string(),
        });
        assert_eq!(expected, result.unwrap_err());
    }

    // TODO: swallows exceptions raised by candidate behaviors

    // TODO: re-raises exceptions raised during publish by default
    // TODO: reports publishing errors
    // TODO: publishes results


    #[test]
    fn should_not_publish_results_when_there_is_only_a_control_value() {
        let r: RefCell<Option<ExperimentResult<u8>>> = RefCell::new(None);

        let mut experiment = Experiment::new_with_context(
            "experiment",
            Context::from_value(json!({"message": "hello world"})).unwrap(),
        );
        experiment.control(|| 1).unwrap();
        experiment.result_publisher(InMemoryPublisher::new(|result| {
            r.replace(Some(result.clone()));
        }));

        experiment.run().unwrap();

        assert!(r.take().is_none());
    }

    // TODO: compares results with a comparator block if provided
    // TODO: compares errors with an error comparator block if provided
    #[test]
    fn should_compare_results_with_comparator_when_provided() {
        #[derive(Clone, PartialEq, Serialize)]
        struct TestResult {
            count: u8,
            message: &'static str,
        }

        let r: RefCell<Option<ExperimentResult<TestResult>>> = RefCell::new(None);

        let mut experiment = Experiment::default();
        experiment.control(|| TestResult {
            count: 1,
            message: "control msg",
        }).unwrap();
        experiment.candidate(|| TestResult {
            count: 1,
            message: "candidate msg",
        }).unwrap();
        experiment.comparator(|a, b| a.count == b.count);
        experiment.result_publisher(InMemoryPublisher::new(|result| {
            r.replace(Some(result.clone()));
        }));

        let value = experiment.run().unwrap();
        assert_eq!(1, value.count);
        assert_eq!("control msg", value.message);
        assert!(r.take().unwrap().matched());
    }


    // TODO: knows how to compare two experiments
    // TODO: uses a compare block to determine if observations are equivalent
    // TODO: reports errors in a compare block
    // TODO: reports errors in the enabled? method
    // TODO: reports errors in a run_if block
    // TODO: returns the given value when no clean block is configured
    // TODO: calls the configured clean block with a value when configured
    // TODO: reports an error and returns the original value when an error is raised in a clean block

    // TODO: calls multiple ignore blocks to see if any match
    // TODO: only calls ignore blocks until one matches
    // TODO: reports exceptions raised in an ignore block and returns false
    // TODO: skips ignore blocks that raise and tests any remaining blocks if an exception is swallowed

    // TODO: raising on mismatches
    // TODO: "raises when there is a mismatch if raise on mismatches is enabled"
    // TODO: "cleans values when raising on observation mismatch"
    // TODO: "doesn't raise when there is a mismatch if raise on mismatches is disabled"
    // TODO: "raises a mismatch error if the control raises and candidate doesn't"
    // TODO: "raises a mismatch error if the candidate raises and the control doesn't"



    // TODO: raise_on_mismatches
    // TODO: raises when there is a mismatch if the experiment instance's raise on mismatches is enabled
    // TODO: doesn't raise when there is a mismatch if the experiment instance's raise on mismatches is disabled

    // TODO: MismatchError
    // TODO: has the name of the experiment
    // TODO: includes the experiments' results
    // TODO: formats nicely as a string
    // TODO: includes the backtrace when an observation raises

    #[test]
    fn should_execute_before_run_when_experiment_is_enabled() {
        let (control_ok, candidate_ok) = (RefCell::new(false), RefCell::new(false));
        let before_run_called = RefCell::new(false);

        let mut experiment = Experiment::default();
        experiment.before_run(|| { before_run_called.replace(true); });
        experiment.control(|| { control_ok.replace(*before_run_called.borrow()); }).unwrap();
        experiment.candidate(|| { candidate_ok.replace(*before_run_called.borrow()); }).unwrap();

        let value = experiment.run().unwrap();

        assert!(before_run_called.take());
        assert!(control_ok.take());
        assert!(candidate_ok.take());
    }

    #[test]
    fn should_not_execute_before_run_when_experiment_is_disabled() {
        let before_run_called = RefCell::new(false);

        let mut experiment = Experiment::default();
        experiment.enabled(|| { false });
        experiment.before_run(|| { before_run_called.replace(true); });
        experiment.control(|| { 1 }).unwrap();
        experiment.candidate(|| { 1 }).unwrap();

        let value = experiment.run().unwrap();

        assert!(!before_run_called.take());
    }

    #[test]
    fn should_be_able_to_serialize_experiment_result() {
        let r: RefCell<Option<ExperimentResult<String>>> = RefCell::new(None);

        let mut experiment = Experiment::default();
        experiment.add_ignore(|_a, b| {
            b.value == "ignored"
        });
        experiment.control(|| "control".to_string()).unwrap();
        experiment.candidate(|| "candidate".to_string()).unwrap();
        experiment.candidate_with_name("second", || "ignored".to_string()).unwrap();
        experiment.result_publisher(InMemoryPublisher::new(|result| {
            r.replace(Some(result.clone()));
        }));

        experiment.run().unwrap();

        let experiment_result = r.take().unwrap();
        assert!(experiment_result.has_mismatches());
        assert!(experiment_result.has_ignores());

        let binary = bincode::serialize(&experiment_result);
        assert!(binary.is_ok());

        let json = serde_json::to_string(&experiment_result);
        assert!(json.is_ok())
    }

    fn create_observation(name: &'static str) -> Observation<u8> {
        return Observation::new(
            name.to_string(),
            "experiment".to_string(),
            1,
            None,
            1
        );
    }
}
