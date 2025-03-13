pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

pub fn sub(left: i32, right: i32) -> i32 {
    left - right
}

use mockall::{predicate::*, *};
#[automock]
trait MyTrait {
    fn my_method(&self, x: u32) -> u32;
}

fn trait_fn(x: &dyn MyTrait, v: u32) -> u32 {
    x.my_method(v)
}

fn main() {
    let mut mock = MockMyTrait::new();
    mock.expect_my_method().returning(|x| x + 1);
    assert_eq!(10, trait_fn(&mock, 9))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[rstest]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[fixture]
    fn sample_fixture() -> i32 {
        42
    }
    #[rstest]
    fn sample_fixture_usage(sample_fixture: i32) {
        assert_eq!(sample_fixture / 6, 7);
    }

    #[rstest]
    #[case(10, 0, 10)]
    #[case(100, 5, 95)]
    fn test_sub(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
        assert_eq!(sub(a, b), expected);
    }
}
