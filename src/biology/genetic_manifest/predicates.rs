use crate::HashMap;
use std::rc::Rc;

pub type OperatorId = u8;

pub type OperatorParam = i32;

#[derive(Clone)]
pub enum OperatorParamType {
    Constant,
    Property,
}

#[derive(Clone)]
pub struct OperatorParamDefinition {
    pub operator_type: OperatorParamType,
}

//pub type OperatorIndex = u8;

#[derive(Clone)]
pub struct Operator {
    pub name: &'static str,
    pub num_params: usize,
    pub evaluate: OperatorFunctionInstance,
    pub index: OperatorId,
    pub render: OperatorRenderFunction,
    pub is_constant: bool,
}

pub type OperatorFunction = dyn Fn(&[OperatorParam]) -> bool;
pub type OperatorFunctionInstance = Rc<OperatorFunction>;

pub type OperatorRenderFunction = Rc<dyn Fn(&[String]) -> String>;
pub type OperatorRenderFunctionInstance = Rc<OperatorRenderFunction>;

#[derive(Clone)]
pub struct OperatorSet {
    pub operators: Vec<Operator>,
    pub by_string_key: HashMap<String, usize>,
}

impl OperatorSet {
    pub fn by_key(&self, key: &str) -> &Operator {
        let i: usize = *self
            .by_string_key
            .get(key)
            .expect(&format!("cant find operator key: {}", key));
        self.operators.get(i).unwrap()
    }
}

fn to_operator_set<'a>(mut operators: Vec<Operator>) -> OperatorSet {
    let mut by_string_key: HashMap<String, usize> = HashMap::new();

    let operators = operators
        .into_iter()
        .enumerate()
        .map(|(i, mut op)| -> Operator {
            op.index = i as OperatorId;
            by_string_key.insert(op.name.to_string(), i);
            op
        })
        .collect::<Vec<_>>();

    OperatorSet {
        operators: operators,
        by_string_key: by_string_key,
    }
}

pub fn default_operators() -> OperatorSet {
    return to_operator_set(vec![
        Operator {
            index: 0,
            name: "eq",
            num_params: 2,
            is_constant: false,

            evaluate: Rc::new(|params: &[OperatorParam]| -> bool { return params[0] == params[1] }),

            render: Rc::new(|param_strs: &[String]| -> String {
                format!("{} == {}", &param_strs[0], &param_strs[1])
            }),
        },
        Operator {
            index: 0,
            name: "is_truthy",
            num_params: 1,
            is_constant: false,
            evaluate: Rc::new(|params: &[OperatorParam]| -> bool {
                return params[0] > 0;
            }),

            render: Rc::new(|param_strs: &[String]| -> String {
                format!("is_truthy({})", param_strs[0])
            }),
        },
        Operator {
            index: 0,
            name: "is_falsy",
            num_params: 1,
            is_constant: false,
            evaluate: Rc::new(|params: &[OperatorParam]| -> bool {
                return params[0] == 0;
            }),

            render: Rc::new(|param_strs: &[String]| -> String {
                format!("is_truthy({})", param_strs[0])
            }),
        },
        Operator {
            index: 0,
            name: "gt",
            num_params: 2,
            is_constant: false,
            evaluate: Rc::new(|params: &[OperatorParam]| -> bool { return params[0] > params[1] }),
            render: Rc::new(|param_strs: &[String]| -> String {
                format!("{} > {}", param_strs[0], param_strs[1])
            }),
        },
        Operator {
            index: 0,
            name: "gte",
            num_params: 2,
            is_constant: false,
            evaluate: Rc::new(|params: &[OperatorParam]| -> bool { return params[0] >= params[1] }),
            render: Rc::new(|param_strs: &[String]| -> String {
                format!("{} >= {}", param_strs[0], param_strs[1])
            }),
        },
        Operator {
            index: 0,
            name: "lt",
            num_params: 2,
            is_constant: false,
            evaluate: Rc::new(|params: &[OperatorParam]| -> bool { return params[0] < params[1] }),
            render: Rc::new(|param_strs: &[String]| -> String {
                format!("{} < {}", param_strs[0], param_strs[1])
            }),
        },
        Operator {
            index: 0,
            name: "lte",
            num_params: 2,
            is_constant: false,
            evaluate: Rc::new(|params: &[OperatorParam]| -> bool { return params[0] <= params[1] }),
            render: Rc::new(|param_strs: &[String]| -> String {
                format!("{} <= {}", param_strs[0], param_strs[1])
            }),
        },
        Operator {
            index: 0,
            name: "true",
            num_params: 0,
            is_constant: true,
            evaluate: Rc::new(|params: &[OperatorParam]| -> bool { true }),
            render: Rc::new(|param_strs: &[String]| -> String { format!("TRUE") }),
        },
        Operator {
            index: 0,
            name: "false",
            num_params: 0,
            is_constant: true,
            evaluate: Rc::new(|params: &[OperatorParam]| -> bool { false }),
            render: Rc::new(|param_strs: &[String]| -> String { format!("FALSE") }),
        },
        Operator {
            index: 0,
            name: "is_even",
            num_params: 0,
            is_constant: false,
            evaluate: Rc::new(|params: &[OperatorParam]| -> bool { params[0] % 2 == 0 }),
            render: Rc::new(|param_strs: &[String]| -> String {
                format!("is_even({})", param_strs[0])
            }),
        },
    ]);
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_equals() {
        let operators = default_operators();
        let op = operators.by_key("eq");
        assert_eq!((op.evaluate)(&vec![1, 1]), true);
        assert_eq!((op.evaluate)(&vec![1, 2]), false);
    }

    fn test_lt() {
        let operators = default_operators();
        let op = operators.by_key("lt");
        assert_eq!((op.evaluate)(&vec![1, 3]), true);
        assert_eq!((op.evaluate)(&vec![1, 1]), false);
    }

    fn test_gt() {
        let operators = default_operators();
        let op = operators.by_key("gt");
        assert_eq!((op.evaluate)(&vec![2, 1]), true);
        assert_eq!((op.evaluate)(&vec![1, 1]), false);
    }
}
