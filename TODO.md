# TODO

List of TODOS

1. include a flag for raising/returning errors. See what scientist does but initial thought is that we might always want to make sure "control" runs 
2. Controlled vs Uncontrolled Experiment. I think this would provide a better API
3. Review structure of Observations and ExperimentResults
4. How to capture variables in closure
5. Capture exceptions and durations for observations
6. Avoiding clone to build experiment results
7. Cleaning Values
8. Comparing values
9. Publishing Results
10. Parallel computation
11. OTEL publisher - will probably go into a separate lib dedicated to publishers
12. Should we just return serde json Value instead of a generic R from a behavior? It does need to technically be serializable so that it can be reported.
