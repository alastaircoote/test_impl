use std::fmt::Debug;

use test_impl::test_impl;

trait ExampleTrait: Debug {
    fn return_true() -> bool;
}

#[derive(Debug)]
struct ExampleStruct;

impl ExampleTrait for ExampleStruct {
    fn return_true() -> bool {
        true
    }
}

#[derive(Debug)]
struct ExampleStruct2;

impl ExampleTrait for ExampleStruct2 {
    fn return_true() -> bool {
        false
    }
}

#[test_impl(ExampleTrait = ExampleStruct, ExampleStruct2)]
#[test]
fn example_test() {
    println!("woah")
    // println!("Going to test {:?}", ExampleTrait {});
    // let bool_value = ExampleTrait::return_true();
    // assert_eq!(bool_value, true);
}
