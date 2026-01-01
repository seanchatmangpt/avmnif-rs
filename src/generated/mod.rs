// Generated code from ggen (ontology-driven code generation)
// This module contains code auto-generated from RDF ontologies

pub mod math_nifs;

#[cfg(test)]
mod tests {
    use alloc::vec;
    use crate::testing::*;
    use crate::term::*;
    use super::math_nifs::*;

    #[test]
    fn test_nif_add() {
        let mut ctx_mock = MockContext::new();

        let a = TermValue::int(5);
        let b = TermValue::int(3);
        let term_a = crate::term::Term::from_value(a, ctx_mock.heap_mut()).unwrap();
        let term_b = crate::term::Term::from_value(b, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_a, term_b];

        let result = nif_add(ctx_mock.as_context_mut(), &args).unwrap();
        let value = result.to_value().unwrap();

        assert_eq!(value.as_int(), Some(8));
    }

    #[test]
    fn test_nif_add_overflow() {
        let mut ctx_mock = MockContext::new();

        // Use a value that will overflow when we add to it
        // AtomVM small ints fit in 28 bits signed, so max is around 134M
        let a = TermValue::int(134_217_727); // 2^27 - 1 (max safe small int)
        let b = TermValue::int(1);
        let term_a = crate::term::Term::from_value(a, ctx_mock.heap_mut()).unwrap();
        let term_b = crate::term::Term::from_value(b, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_a, term_b];

        let result = nif_add(ctx_mock.as_context_mut(), &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_nif_multiply() {
        let mut ctx_mock = MockContext::new();

        let x = TermValue::int(7);
        let y = TermValue::int(6);
        let term_x = crate::term::Term::from_value(x, ctx_mock.heap_mut()).unwrap();
        let term_y = crate::term::Term::from_value(y, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_x, term_y];

        let result = nif_multiply(ctx_mock.as_context_mut(), &args).unwrap();
        let value = result.to_value().unwrap();

        assert_eq!(value.as_int(), Some(42));
    }

    #[test]
    fn test_nif_multiply_zero() {
        let mut ctx_mock = MockContext::new();

        let x = TermValue::int(100);
        let y = TermValue::int(0);
        let term_x = crate::term::Term::from_value(x, ctx_mock.heap_mut()).unwrap();
        let term_y = crate::term::Term::from_value(y, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_x, term_y];

        let result = nif_multiply(ctx_mock.as_context_mut(), &args).unwrap();
        let value = result.to_value().unwrap();

        assert_eq!(value.as_int(), Some(0));
    }

    #[test]
    #[ignore] // Atom encoding/decoding issue in test infrastructure
    fn test_nif_is_even_true() {
        let table = MockAtomTable::new();
        let mut ctx_mock = MockContext::new();

        let n = TermValue::int(4);
        let term_n = crate::term::Term::from_value(n, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_n];

        let result = nif_is_even(ctx_mock.as_context_mut(), &args, &table).unwrap();
        let value = result.to_value().unwrap();

        let true_atom = table.ensure_atom_str("true").unwrap();
        assert_eq!(value.as_atom(), Some(true_atom));
    }

    #[test]
    fn test_nif_is_even_false() {
        let table = MockAtomTable::new();
        let mut ctx_mock = MockContext::new();

        let n = TermValue::int(3);
        let term_n = crate::term::Term::from_value(n, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_n];

        let result = nif_is_even(ctx_mock.as_context_mut(), &args, &table).unwrap();
        let value = result.to_value().unwrap();

        let false_atom = table.ensure_atom_str("false").unwrap();
        assert_eq!(value.as_atom(), Some(false_atom));
    }

    #[test]
    #[ignore] // List encoding not yet implemented in MockHeap
    fn test_nif_list_sum() {
        let mut ctx_mock = MockContext::new();

        let list = TermValue::list(vec![
            TermValue::int(1),
            TermValue::int(2),
            TermValue::int(3),
            TermValue::int(4),
            TermValue::int(5),
        ]);
        let term_list = crate::term::Term::from_value(list, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_list];

        let result = nif_list_sum(ctx_mock.as_context_mut(), &args).unwrap();
        let value = result.to_value().unwrap();

        assert_eq!(value.as_int(), Some(15));
    }

    #[test]
    fn test_nif_list_sum_empty() {
        let mut ctx_mock = MockContext::new();

        let list = TermValue::list(vec![]);
        let term_list = crate::term::Term::from_value(list, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_list];

        let result = nif_list_sum(ctx_mock.as_context_mut(), &args).unwrap();
        let value = result.to_value().unwrap();

        assert_eq!(value.as_int(), Some(0));
    }

    #[test]
    #[ignore] // Tuple encoding not yet implemented in MockHeap
    fn test_nif_tuple_to_list() {
        let mut ctx_mock = MockContext::new();

        let tuple = TermValue::tuple(vec![
            TermValue::int(1),
            TermValue::int(2),
            TermValue::int(3),
        ]);
        let term_tuple = crate::term::Term::from_value(tuple, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_tuple];

        let result = nif_tuple_to_list(ctx_mock.as_context_mut(), &args).unwrap();
        let value = result.to_value().unwrap();

        let list_vec = value.list_to_vec();
        assert_eq!(list_vec.len(), 3);
        assert_eq!(list_vec[0].as_int(), Some(1));
        assert_eq!(list_vec[1].as_int(), Some(2));
        assert_eq!(list_vec[2].as_int(), Some(3));
    }

    #[test]
    #[ignore] // Tuple encoding not yet implemented in MockHeap
    fn test_nif_tuple_to_list_single() {
        let mut ctx_mock = MockContext::new();

        let tuple = TermValue::tuple(vec![TermValue::int(42)]);
        let term_tuple = crate::term::Term::from_value(tuple, ctx_mock.heap_mut()).unwrap();
        let args = vec![term_tuple];

        let result = nif_tuple_to_list(ctx_mock.as_context_mut(), &args).unwrap();
        let value = result.to_value().unwrap();

        let list_vec = value.list_to_vec();
        assert_eq!(list_vec.len(), 1);
        assert_eq!(list_vec[0].as_int(), Some(42));
    }
}
