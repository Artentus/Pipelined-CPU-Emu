use langbox::TextSpan;

use super::ast::*;
use super::{AssemblerError, SharedStr};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum EvalError {
    InvalidLiteralValue(IntegerLiteral),
    DivideByZero(BinaryExpression),
    ErrorInReferenceEval,
    MissingReferenceValue,
    UndefinedSymbol(Identifier),
}

impl Expression {
    pub fn try_eval(
        &self,
        label_set: &HashMap<SharedStr, TextSpan>,
        value_map: &HashMap<SharedStr, Option<i64>>,
    ) -> Result<i64, EvalError> {
        match self {
            Expression::Literal(expr) => expr
                .value()
                .ok_or_else(|| EvalError::InvalidLiteralValue(expr.as_ref().clone())),
            Expression::Identifier(expr) => {
                if let Some(value) = value_map.get(&expr.name()).copied() {
                    value.ok_or(EvalError::ErrorInReferenceEval)
                } else if label_set.contains_key(&expr.name()) {
                    Err(EvalError::MissingReferenceValue)
                } else {
                    Err(EvalError::UndefinedSymbol(expr.as_ref().clone()))
                }
            }
            Expression::Group(expr) => expr.inner().try_eval(label_set, value_map),
            Expression::Identity(expr) => expr.inner().try_eval(label_set, value_map),
            Expression::Negation(expr) => expr
                .inner()
                .try_eval(label_set, value_map)
                .map(|value| value.wrapping_neg()),
            Expression::BitwiseNot(expr) => expr
                .inner()
                .try_eval(label_set, value_map)
                .map(|value| !value),
            Expression::Addition(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                Ok(lhs.wrapping_add(rhs))
            }
            Expression::Subtraction(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                Ok(lhs.wrapping_sub(rhs))
            }
            Expression::Multiplication(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                Ok(lhs.wrapping_mul(rhs))
            }
            Expression::Division(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                if rhs == 0 {
                    Err(EvalError::DivideByZero(expr.as_ref().clone()))
                } else {
                    Ok(lhs.wrapping_div(rhs))
                }
            }
            Expression::Remainder(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                if rhs == 0 {
                    Err(EvalError::DivideByZero(expr.as_ref().clone()))
                } else {
                    Ok(lhs.wrapping_rem(rhs))
                }
            }
            Expression::LeftShift(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                Ok(lhs << rhs)
            }
            Expression::ArithmeticRightShift(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                Ok(lhs >> rhs)
            }
            Expression::LogicalRightShift(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)? as u64;
                let rhs = expr.rhs().try_eval(label_set, value_map)? as u64;
                Ok((lhs >> rhs) as i64)
            }
            Expression::BitwiseAnd(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                Ok(lhs & rhs)
            }
            Expression::BitwiseOr(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                Ok(lhs | rhs)
            }
            Expression::BitwiseXor(expr) => {
                let lhs = expr.lhs().try_eval(label_set, value_map)?;
                let rhs = expr.rhs().try_eval(label_set, value_map)?;
                Ok(lhs ^ rhs)
            }
        }
    }

    pub fn eval_or_zero(
        &self,
        label_set: &HashMap<SharedStr, TextSpan>,
        value_map: &HashMap<SharedStr, Option<i64>>,
        errors: &mut Vec<AssemblerError>,
    ) -> i64 {
        match self.try_eval(label_set, value_map) {
            Ok(value) => value,
            Err(EvalError::InvalidLiteralValue(_))
            | Err(EvalError::ErrorInReferenceEval)
            | Err(EvalError::MissingReferenceValue) => {
                unreachable!("uncought eval error");
            }
            Err(EvalError::DivideByZero(expr)) => {
                errors.push(AssemblerError::DivideByZero { expr: expr.span() });
                0
            }
            Err(EvalError::UndefinedSymbol(ident)) => {
                errors.push(AssemblerError::UndefinedSymbol {
                    ident: ident.span(),
                });
                0
            }
        }
    }
}
