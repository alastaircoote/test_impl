use std::fmt::Debug;

use test_impl::test_impl;

trait Whaa {}

trait ExampleTrait<'a, 'b>: Debug {
    fn return_true() -> bool;
}

#[derive(Debug)]
struct ExampleStruct;

impl<'a, 'b> ExampleTrait<'a, 'b> for ExampleStruct {
    fn return_true() -> bool {
        true
    }
}

#[derive(Debug)]
struct ExampleStruct2 {}

impl<'a, 'b> ExampleTrait<'a, 'b> for ExampleStruct2 {
    fn return_true() -> bool {
        false
    }
}

#[test_impl(ExampleTrait<'avs, 'bvs> = ExampleStruct, ExampleStruct2)]
#[test]
fn example_test_with_lifetimes() {
    println!("Going to test {:?}", ExampleTrait {});
    let bool_value = ExampleTrait::return_true();
    assert_eq!(bool_value, true);
}
