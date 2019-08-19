// Generic walk through a typed AST. Not mutating in place

use crate::typed_absy::*;
use zokrates_field::field::Field;

pub trait Folder<'ast, T: Field>: Sized {
    fn fold_program(&mut self, p: TypedProgram<'ast, T>) -> TypedProgram<'ast, T> {
        fold_program(self, p)
    }

    fn fold_module(&mut self, p: TypedModule<'ast, T>) -> TypedModule<'ast, T> {
        fold_module(self, p)
    }

    fn fold_function_symbol(
        &mut self,
        s: TypedFunctionSymbol<'ast, T>,
    ) -> TypedFunctionSymbol<'ast, T> {
        fold_function_symbol(self, s)
    }

    fn fold_function(&mut self, f: TypedFunction<'ast, T>) -> TypedFunction<'ast, T> {
        fold_function(self, f)
    }

    fn fold_parameter(&mut self, p: Parameter<'ast>) -> Parameter<'ast> {
        Parameter {
            id: self.fold_variable(p.id),
            ..p
        }
    }

    fn fold_name(&mut self, n: Identifier<'ast>) -> Identifier<'ast> {
        n
    }

    fn fold_variable(&mut self, v: Variable<'ast>) -> Variable<'ast> {
        Variable {
            id: self.fold_name(v.id),
            ..v
        }
    }

    fn fold_assignee(&mut self, a: TypedAssignee<'ast, T>) -> TypedAssignee<'ast, T> {
        match a {
            TypedAssignee::Identifier(v) => TypedAssignee::Identifier(self.fold_variable(v)),
            TypedAssignee::ArrayElement(box a, box index) => TypedAssignee::ArrayElement(
                box self.fold_assignee(a),
                box self.fold_field_expression(index),
            ),
        }
    }

    fn fold_statement(&mut self, s: TypedStatement<'ast, T>) -> Vec<TypedStatement<'ast, T>> {
        fold_statement(self, s)
    }

    fn fold_expression(&mut self, e: TypedExpression<'ast, T>) -> TypedExpression<'ast, T> {
        match e {
            TypedExpression::FieldElement(e) => self.fold_field_expression(e).into(),
            TypedExpression::Boolean(e) => self.fold_boolean_expression(e).into(),
            TypedExpression::FieldElementArray(e) => self.fold_field_array_expression(e).into(),
        }
    }

    fn fold_expression_list(
        &mut self,
        es: TypedExpressionList<'ast, T>,
    ) -> TypedExpressionList<'ast, T> {
        match es {
            TypedExpressionList::FunctionCall(id, arguments, types) => {
                TypedExpressionList::FunctionCall(
                    id,
                    arguments
                        .into_iter()
                        .map(|a| self.fold_expression(a))
                        .collect(),
                    types,
                )
            }
        }
    }

    fn fold_field_expression(
        &mut self,
        e: FieldElementExpression<'ast, T>,
    ) -> FieldElementExpression<'ast, T> {
        fold_field_expression(self, e)
    }
    fn fold_boolean_expression(
        &mut self,
        e: BooleanExpression<'ast, T>,
    ) -> BooleanExpression<'ast, T> {
        fold_boolean_expression(self, e)
    }
    fn fold_field_array_expression(
        &mut self,
        e: FieldElementArrayExpression<'ast, T>,
    ) -> FieldElementArrayExpression<'ast, T> {
        fold_field_array_expression(self, e)
    }

    fn fold_array_value(&mut self, v: TypedArrayValue<'ast, T>) -> TypedArrayValue<'ast, T> {
        fold_array_value(self, v)
    }

    fn fold_spread_or_expression(
        &mut self,
        e: TypedArrayValueElement<'ast, T>,
    ) -> TypedArrayValueElement<'ast, T> {
        fold_spread_or_expression(self, e)
    }
}

pub fn fold_module<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    p: TypedModule<'ast, T>,
) -> TypedModule<'ast, T> {
    TypedModule {
        functions: p
            .functions
            .into_iter()
            .map(|(key, fun)| (key, f.fold_function_symbol(fun)))
            .collect(),
        ..p
    }
}

pub fn fold_statement<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    s: TypedStatement<'ast, T>,
) -> Vec<TypedStatement<'ast, T>> {
    let res = match s {
        TypedStatement::Return(expressions) => TypedStatement::Return(
            expressions
                .into_iter()
                .map(|e| f.fold_expression(e))
                .collect(),
        ),
        TypedStatement::Definition(a, e) => {
            TypedStatement::Definition(f.fold_assignee(a), f.fold_expression(e))
        }
        TypedStatement::Declaration(v) => TypedStatement::Declaration(f.fold_variable(v)),
        TypedStatement::Condition(left, right) => {
            TypedStatement::Condition(f.fold_expression(left), f.fold_expression(right))
        }
        TypedStatement::For(v, from, to, statements) => TypedStatement::For(
            f.fold_variable(v),
            from,
            to,
            statements
                .into_iter()
                .flat_map(|s| f.fold_statement(s))
                .collect(),
        ),
        TypedStatement::MultipleDefinition(variables, elist) => TypedStatement::MultipleDefinition(
            variables.into_iter().map(|v| f.fold_variable(v)).collect(),
            f.fold_expression_list(elist),
        ),
    };
    vec![res]
}

pub fn fold_array_value<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    v: TypedArrayValue<'ast, T>,
) -> TypedArrayValue<'ast, T> {
    TypedArrayValue(
        v.0.into_iter()
            .map(|v| f.fold_spread_or_expression(v))
            .collect(),
    )
}

pub fn fold_field_array_expression<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    e: FieldElementArrayExpression<'ast, T>,
) -> FieldElementArrayExpression<'ast, T> {
    match e {
        FieldElementArrayExpression::Identifier(size, id) => {
            FieldElementArrayExpression::Identifier(size, f.fold_name(id))
        }
        FieldElementArrayExpression::Value(size, val) => {
            FieldElementArrayExpression::Value(size, f.fold_array_value(val))
        }
        FieldElementArrayExpression::FunctionCall(size, id, exps) => {
            let exps = exps.into_iter().map(|e| f.fold_expression(e)).collect();
            FieldElementArrayExpression::FunctionCall(size, id, exps)
        }
        FieldElementArrayExpression::IfElse(box condition, box consequence, box alternative) => {
            FieldElementArrayExpression::IfElse(
                box f.fold_boolean_expression(condition),
                box f.fold_field_array_expression(consequence),
                box f.fold_field_array_expression(alternative),
            )
        }
        FieldElementArrayExpression::Slice(size, box array, start) => {
            FieldElementArrayExpression::Slice(
                size,
                box f.fold_field_array_expression(array),
                start,
            )
        }
    }
}

pub fn fold_field_expression<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    e: FieldElementExpression<'ast, T>,
) -> FieldElementExpression<'ast, T> {
    match e {
        FieldElementExpression::Number(n) => FieldElementExpression::Number(n),
        FieldElementExpression::Identifier(id) => {
            FieldElementExpression::Identifier(f.fold_name(id))
        }
        FieldElementExpression::Add(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            FieldElementExpression::Add(box e1, box e2)
        }
        FieldElementExpression::Sub(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            FieldElementExpression::Sub(box e1, box e2)
        }
        FieldElementExpression::Mult(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            FieldElementExpression::Mult(box e1, box e2)
        }
        FieldElementExpression::Div(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            FieldElementExpression::Div(box e1, box e2)
        }
        FieldElementExpression::Pow(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            FieldElementExpression::Pow(box e1, box e2)
        }
        FieldElementExpression::IfElse(box cond, box cons, box alt) => {
            let cond = f.fold_boolean_expression(cond);
            let cons = f.fold_field_expression(cons);
            let alt = f.fold_field_expression(alt);
            FieldElementExpression::IfElse(box cond, box cons, box alt)
        }
        FieldElementExpression::FunctionCall(key, exps) => {
            let exps = exps.into_iter().map(|e| f.fold_expression(e)).collect();
            FieldElementExpression::FunctionCall(key, exps)
        }
        FieldElementExpression::Select(box array, box index) => {
            let array = f.fold_field_array_expression(array);
            let index = f.fold_field_expression(index);
            FieldElementExpression::Select(box array, box index)
        }
    }
}

pub fn fold_boolean_expression<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    e: BooleanExpression<'ast, T>,
) -> BooleanExpression<'ast, T> {
    match e {
        BooleanExpression::Value(v) => BooleanExpression::Value(v),
        BooleanExpression::Identifier(id) => BooleanExpression::Identifier(f.fold_name(id)),
        BooleanExpression::Eq(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            BooleanExpression::Eq(box e1, box e2)
        }
        BooleanExpression::Lt(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            BooleanExpression::Lt(box e1, box e2)
        }
        BooleanExpression::Le(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            BooleanExpression::Le(box e1, box e2)
        }
        BooleanExpression::Gt(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            BooleanExpression::Gt(box e1, box e2)
        }
        BooleanExpression::Ge(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1);
            let e2 = f.fold_field_expression(e2);
            BooleanExpression::Ge(box e1, box e2)
        }
        BooleanExpression::Or(box e1, box e2) => {
            let e1 = f.fold_boolean_expression(e1);
            let e2 = f.fold_boolean_expression(e2);
            BooleanExpression::Or(box e1, box e2)
        }
        BooleanExpression::And(box e1, box e2) => {
            let e1 = f.fold_boolean_expression(e1);
            let e2 = f.fold_boolean_expression(e2);
            BooleanExpression::And(box e1, box e2)
        }
        BooleanExpression::Not(box e) => {
            let e = f.fold_boolean_expression(e);
            BooleanExpression::Not(box e)
        }
    }
}

pub fn fold_function<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    fun: TypedFunction<'ast, T>,
) -> TypedFunction<'ast, T> {
    TypedFunction {
        arguments: fun
            .arguments
            .into_iter()
            .map(|a| f.fold_parameter(a))
            .collect(),
        statements: fun
            .statements
            .into_iter()
            .flat_map(|s| f.fold_statement(s))
            .collect(),
        ..fun
    }
}

pub fn fold_function_symbol<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    s: TypedFunctionSymbol<'ast, T>,
) -> TypedFunctionSymbol<'ast, T> {
    match s {
        TypedFunctionSymbol::Here(fun) => TypedFunctionSymbol::Here(f.fold_function(fun)),
        there => there, // by default, do not fold modules recursively
    }
}

pub fn fold_spread_or_expression<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    e: TypedArrayValueElement<'ast, T>,
) -> TypedArrayValueElement<'ast, T> {
    match e {
        TypedArrayValueElement::Spread(a) => {
            TypedArrayValueElement::Spread(f.fold_field_array_expression(a))
        }
        TypedArrayValueElement::Expression(e) => {
            TypedArrayValueElement::Expression(f.fold_expression(e))
        }
    }
}

pub fn fold_program<'ast, T: Field, F: Folder<'ast, T>>(
    f: &mut F,
    p: TypedProgram<'ast, T>,
) -> TypedProgram<'ast, T> {
    TypedProgram {
        modules: p
            .modules
            .into_iter()
            .map(|(module_id, module)| (module_id, f.fold_module(module)))
            .collect(),
        main: p.main,
    }
}
