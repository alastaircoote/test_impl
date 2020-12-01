# test-impl

`test-impl` is a Rust macro for use in testing when you want to test multiple implementations of a trait to ensure they all return the same result.

## How do I use it?

An example of the macro in use:

    #[test_impl(ExampleTrait = ExampleStruct, ExampleStruct2)]
    #[test]
    fn example_test() {
        let bool_value = ExampleTrait::return_true();
        assert_eq!(bool_value, true)
    }

With the arguments provided here the macro will test both ExampleStruct and ExampleStruct2 against the code you've provided. The translated version looks something like this:

    #[test]
    fn example_test() {
        fn ExampleStruct() {
            type ExampleTrait = ExampleStruct;
            let bool_value = ExampleTrait::return_true();
            assert_eq!(bool_value, true)
        }
        ExampleStruct();

        fn ExampleStruct2() {
            type ExampleTrait = ExampleStruct2;
            let bool_value = ExampleTrait::return_true();
            assert_eq!(bool_value, true)
        }
        ExampleStruct2();
    }

Why lay it out this way? Becuse it allows us to preserve line numbers in test backtraces while also being able to detect which implementation failed. For example, when one of those implementations returns false:

    thread 'example_test' panicked at 'assertion failed: `(left == right)`
      left: `false`,
    right: `true`', tests/example.rs:32:5
    stack backtrace:
      0: rust_begin_unwind
                at /rustc/18bf6b4f01a6feaf7259ba7cdae58031af1b7b39/library/std/src/panicking.rs:475
      1: std::panicking::begin_panic_fmt
                at /rustc/18bf6b4f01a6feaf7259ba7cdae58031af1b7b39/library/std/src/panicking.rs:429
      2: example::example_test::impl_ExampleStruct2
                at ./tests/example.rs:32
      3: example::example_test
                at ./tests/example.rs:27
      4: example::example_test::{{closure}}
                at ./tests/example.rs:27

The backtrace indicates that the panic occurred inside `example::example_test::impl_ExampleStruct2`, telling us that it was `ExampleStruct2` causing the problem.
