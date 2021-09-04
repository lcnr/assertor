use std::borrow::Borrow;
use std::collections::HashSet;

use crate::assertions::iterator::IteratorAssertion;
use crate::base::{AssertionApi, AssertionResult, Fact, ReturnStrategy, Subject};
use crate::testing::CheckThatResult;

pub trait AssertionResultAssertion<'a, R> {
    fn facts_are<B: Borrow<Vec<Fact>>>(&self, facts: B) -> R;
    fn facts_are_at_least<B: Borrow<Vec<Fact>>>(&self, facts: B) -> R;
    // Returns subject of fact value for the first matched key.
    fn fact_value_for_key<I: Into<String>>(&self, key: I) -> Subject<String, (), R>;
    fn fact_keys(&self) -> Subject<'a, HashSet<&String>, (), R>;
}

fn get_assertion_result<'a, 'o, R>(
    subject: &'o Subject<'a, CheckThatResult, (), R>,
) -> &'o AssertionResult {
    subject
        .actual()
        .as_ref()
        .expect_err("Because this is assertion for error message.")
}

impl<'a, R> AssertionResultAssertion<'a, R> for Subject<'a, CheckThatResult, (), R>
where
    AssertionResult: ReturnStrategy<R>,
{
    fn facts_are<B: Borrow<Vec<Fact>>>(&self, expected: B) -> R {
        self.new_owned_subject(
            get_assertion_result(&self).facts().iter(),
            Some(format!("{}.facts()", self.description_or_expr())),
            (),
        )
        .contains_exactly_in_order(expected.borrow().iter())
    }

    fn facts_are_at_least<B: Borrow<Vec<Fact>>>(&self, facts: B) -> R {
        self.new_owned_subject(
            get_assertion_result(&self).facts().iter(),
            Some(format!("{}.facts()", self.description_or_expr())),
            (),
        )
        .contains_at_least_in_order(facts.borrow().iter())
    }

    fn fact_value_for_key<I: Into<String>>(&self, key: I) -> Subject<String, (), R> {
        let key_str = key.into();
        let assertion_result = get_assertion_result(&self);
        let value = assertion_result
            .facts()
            .iter()
            .flat_map(|fact| match fact {
                Fact::KeyValue { key: k, value } if k.eq(&key_str) => Some(value),
                _ => None,
            })
            .next()
            .expect(&format!(
                "key `{}` not found in assertion result.\n{:?}",
                key_str,
                assertion_result.generate_message()
            ))
            .clone();
        self.new_owned_subject(
            value,
            Some(format!("{}.[key={}]", self.description_or_expr(), key_str)),
            (),
        )
    }

    fn fact_keys(&self) -> Subject<HashSet<&String>, (), R> {
        let assertion_result = get_assertion_result(self);
        let keys: HashSet<&String> = assertion_result
            .facts()
            .iter()
            .flat_map(|fact| match fact {
                Fact::KeyValue { key, .. } => Some(key),
                Fact::Value { .. } => None,
                Fact::Splitter => None,
            })
            .collect();
        self.new_owned_subject(
            keys,
            Some(format!("{}.keys()", self.description_or_expr())),
            (),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use crate::{assert_that, check_that, Fact};

    use super::*;

    trait TestAssertion<'a, S, R> {
        fn is_same_to<B>(&self, expected: B) -> R
        where
            B: Borrow<S>,
            S: PartialEq + Debug;
    }

    impl<'a, S, R> TestAssertion<'a, S, R> for Subject<'a, S, (), R>
    where
        AssertionResult: ReturnStrategy<R>,
    {
        fn is_same_to<B>(&self, expected: B) -> R
        where
            B: Borrow<S>,
            S: PartialEq + Debug,
        {
            match expected.borrow().eq(self.actual().borrow()) {
                true => self.new_result().add_simple_fact("same").do_ok(),
                false => self.new_result().add_simple_fact("not same").do_fail(),
            }
        }
    }

    #[test]
    fn test_assertion() {
        assert_that!("same").is_same_to("same");
        assert_that!(check_that!("actual").is_same_to("expected"))
            .facts_are(vec![Fact::new_simple_fact("not same")]);
    }

    #[test]
    fn facts_are() {
        let failed: CheckThatResult = check_that!("actual").is_same_to("expected");
        assert_that!(check_that!(failed).facts_are(vec![])).facts_are(vec![
            Fact::new("value of", "failed.facts()"),
            Fact::new("unexpected (1)", r#"[Value { value: "not same" }]"#),
            Fact::new_splitter(),
            Fact::new("expected", "[]"),
            Fact::new("actual", r#"[Value { value: "not same" }]"#),
        ]);
    }
}
