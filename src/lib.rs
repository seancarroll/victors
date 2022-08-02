#![doc(html_root_url = "https://docs.rs/victors")]
#![doc(issue_tracker_base_url = "https://github.com/seancarroll/victors/issues/")]
#![cfg_attr(docsrs, deny(broken_intra_doc_links))]
#![feature(backtrace)]
#![deny(elided_lifetimes_in_paths)]

mod errors;
mod experiment;
mod experiment_result;
mod observation;
mod victor;
mod result_publisher;

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::cell::RefCell;
    use std::pin::Pin;
    use std::rc::Rc;
    use crate::errors::{BehaviorNotUnique, VictorsErrors, VictorsResult};
    use crate::experiment::Experiment;
    use crate::experiment_result::ExperimentResult;
    use crate::observation::Observation;
    use crate::result_publisher::InMemoryPublisher;
    use crate::victor::Victor;

    #[test]
    fn it_works() {
        let mut experiment = Experiment::default();
        experiment.control(|| { println!("control...")}).expect("control shouldnt fail");
        experiment.candidate("", || { println!("candidate...") }).expect("candidate shouldnt fail");

        let result = experiment.run();
    }

    #[test]
    fn should_be_able_to_specify_control_name() {
        let mut experiment = Experiment::new("custom control");
        experiment.control(|| { println!("control...")}).expect("control shouldnt fail");

        assert_eq!("custom control", experiment.name);
    }

    // TODO: custom enabled fn
    // TODO: pass context
    // TODO: publish
    // TODO: clean values


    // TODO: run_if - run "experiment" means candidates but not sure if we really need to change
    // the language or not
    #[test]
    fn should_not_run_candidates_when_run_if_returns_false() {
        let mut experiment = Experiment::default();
        experiment.control(|| { 1 }).expect("control shouldnt fail");
        experiment.candidate("candidate", || { 1 }).expect("candidate shouldnt fail");
        experiment.run_if(|| { false });
        let result = experiment.run().unwrap();

        assert_eq!(1, result);
        // TODO: how to confirm candidate
    }

    fn should_run_candidates_when_run_if_returns_true() {
        let mut experiment = Experiment::default();
        experiment.control(|| { 1 }).expect("control shouldnt fail");
        experiment.candidate("candidate", || { 1 }).expect("candidate shouldnt fail");
        experiment.run_if(|| { true });
        let result = experiment.run().unwrap();

        assert_eq!(1, result);
        // TODO: how to confirm candidate
    }


    // TODO: parallel
    // TODO: before_run setup


    #[test]
    fn should_be_able_to_create_and_run_experiment_via_victor() {
        let r = Victor::conduct("conduct test", |experiment| {
            // To fix the error below means we need to return Ok(()) at the end
            // cannot use the `?` operator in a closure that returns `()`
            // I dont love it but I suppose thats part of the rust idioms
            experiment.control(|| { 1 })?;
            experiment.candidate("candidate", || { 2 })?;
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
            experiment.candidate("first", || { 1 })?;
            experiment.candidate("second", || { 2 })?;
            Ok(())
        });

        assert_eq!(Some(2), r.ok());
    }

    #[test]
    fn should_return_non_unique_error_when_multiple_candidates_are_registered_with_same_name() {
        let mut experiment = Experiment::default();
        experiment.control(|| { println!("control...")}).expect("control shouldnt fail");
        experiment.candidate("candidate", || { println!("candidate...") }).expect("candidate shouldnt fail");
        let result = experiment.candidate("candidate", || { println!("second candidate...") });
        let expected = VictorsErrors::BehaviorNotUnique(BehaviorNotUnique {
            experiment_name: "experiment".to_string(),
            name: "candidate".to_string()
        });
        assert_eq!(expected, result.unwrap_err());
    }


    // ignore ignore_mismatched_observation tests
    // TODO: does not ignore an observation if no ignores are configured
    // TODO: calls a configured ignore block with the given observed values
    #[test]
    fn should_call_ignore_blocks_until_match() {
        // TODO: I wish we could capture a variable in the closure to test that specific ignores
        // were called but I cant get FnOnce to work in a loop and not sure how else to accomplish it
        let mut experiment = Experiment::default();
        experiment.add_ignore(|a, b| { false });
        experiment.add_ignore(|a, b| { true });
        experiment.add_ignore(|a, b| { false });

        let a = create_observation("a");
        let b = create_observation("b");

        assert!(experiment.ignore_mismatch_observation(&a, &b));
    }

    // TODO: can't be run without a control behavior  --> this is at least one behavior
    // TODO: runs other behaviors but always returns the control
    // TODO: complains about duplicate behavior names
    // TODO: swallows exceptions raised by candidate behaviors
    // TODO: shuffles behaviors before running
    // TODO: re-raises exceptions raised during publish by default
    // TODO: reports publishing errors
    // TODO: publishes results
    // TODO: does not publish results when there is only a control value
    // TODO: compares results with a comparator block if provided
    // TODO: compares errors with an error comparator block if provided
    #[test]
    fn should_compare_results_with_comparator_when_provided() {
        #[derive(Clone, PartialEq)]
        struct TestResult {
            count: u8,
            message: &'static str
        }

        let r: RefCell<Option<ExperimentResult<TestResult>>> = RefCell::new(None);

        let mut experiment = Experiment::default();
        experiment.control(|| { TestResult { count: 1, message: "control msg"} }).unwrap();
        experiment.candidate("candidate", || TestResult { count: 1, message: "candidate msg"}).unwrap();
        experiment.comparator(|a, b| { a.count == b.count });
        experiment.result_publisher(InMemoryPublisher::new(|result| {
            r.swap(&RefCell::new(Some(result.clone())));
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


    // TODO: can be marshaled

    // TODO: raise_on_mismatches
    // TODO: raises when there is a mismatch if the experiment instance's raise on mismatches is enabled
    // TODO: doesn't raise when there is a mismatch if the experiment instance's raise on mismatches is disabled


    // TODO: MismatchError
    // TODO: has the name of the experiment
    // TODO: includes the experiments' results
    // TODO: formats nicely as a string
    // TODO: includes the backtrace when an observation raises

    // TODO: before run block
    // TODO: does not run when an experiment is disabled



    fn create_observation(name: &'static str) -> Observation<u8> {
        return Observation::new(
            name.to_string(),
            "experiment".to_string(),
            1,
            None,
            1);
    }
}
