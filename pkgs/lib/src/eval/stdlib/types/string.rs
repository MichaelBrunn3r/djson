#![allow(clippy::missing_errors_doc)]

use crate::{
    eval::{EvalError, Map, Value},
    map,
};

#[must_use]
pub fn create_map() -> Map {
    map! {
        len_chars: Value::Function(len_chars),
        len_bytes: Value::Function(len_bytes),
        is_empty: Value::Function(is_empty),
        is_blank: Value::Function(is_blank),
        is_ascii: Value::Function(is_ascii),
        contains: Value::Function(contains),
        starts_with: Value::Function(starts_with),
        ends_with: Value::Function(ends_with),
        remove_prefix: Value::Function(remove_prefix),
        remove_suffix: Value::Function(remove_suffix),
        count: Value::Function(count),
        find: Value::Function(find),
        split: Value::Function(split),
        lines: Value::Function(lines),
        trim: Value::Function(trim),
        trim_start: Value::Function(trim_start),
        trim_end: Value::Function(trim_end),
        uppercase: Value::Function(uppercase),
        lowercase: Value::Function(lowercase),
        replace: Value::Function(replace),
        repeat: Value::Function(repeat),
        pad_start: Value::Function(pad_start),
        pad_end: Value::Function(pad_end),
        reverse: Value::Function(reverse),
    }
}

pub fn len_chars(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => {
            let Ok(length) = i64::try_from(value.chars().count()) else {
                return Err(EvalError::Overflow { span: None });
            };
            Ok(Value::Int(length))
        }
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn len_bytes(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => {
            let Ok(length) = i64::try_from(value.len()) else {
                return Err(EvalError::Overflow { span: None });
            };
            Ok(Value::Int(length))
        }
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub const fn is_empty(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::Bool(value.is_empty())),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn is_blank(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::Bool(value.trim().is_empty())),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn is_ascii(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::Bool(value.is_ascii())),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn contains(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Str(needle)] => Ok(Value::Bool(value.contains(needle.as_str()))),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn starts_with(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Str(prefix)] => {
            Ok(Value::Bool(value.starts_with(prefix.as_str())))
        }
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn ends_with(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Str(suffix)] => {
            Ok(Value::Bool(value.ends_with(suffix.as_str())))
        }
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn remove_prefix(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Str(prefix)] => Ok(Value::Str(
            value
                .strip_prefix(prefix.as_str())
                .unwrap_or(value)
                .to_owned(),
        )),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn remove_suffix(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Str(suffix)] => Ok(Value::Str(
            value
                .strip_suffix(suffix.as_str())
                .unwrap_or(value)
                .to_owned(),
        )),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn count(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Str(needle)] => {
            let count = if needle.is_empty() {
                0
            } else {
                value.matches(needle.as_str()).count()
            };
            let Ok(count) = i64::try_from(count) else {
                return Err(EvalError::Overflow { span: None });
            };
            Ok(Value::Int(count))
        }
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn find(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Str(needle)] => {
            let index = value
                .find(needle.as_str())
                .map(|index| value[..index].chars().count())
                .map(i64::try_from)
                .transpose()
                .map_err(|_| EvalError::Overflow { span: None })?
                .unwrap_or(-1);
            Ok(Value::Int(index))
        }
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn split(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Str(delimiter)] => Ok(Value::List(
            value
                .split(delimiter.as_str())
                .map(|part| Value::Str(part.to_owned()))
                .collect(),
        )),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn lines(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::List(
            value
                .lines()
                .map(|line| Value::Str(line.to_owned()))
                .collect(),
        )),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn uppercase(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::Str(value.to_uppercase())),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn trim(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::Str(value.trim().to_owned())),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn trim_start(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::Str(value.trim_start().to_owned())),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn trim_end(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::Str(value.trim_end().to_owned())),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn lowercase(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::Str(value.to_lowercase())),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn replace(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Str(from), Value::Str(to)] => {
            Ok(Value::Str(value.replace(from.as_str(), to.as_str())))
        }
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn repeat(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value), Value::Int(count)] if *count >= 0 => {
            let Ok(count) = usize::try_from(*count) else {
                return Err(EvalError::Overflow { span: None });
            };
            value
                .len()
                .checked_mul(count)
                .ok_or(EvalError::Overflow { span: None })?;
            Ok(Value::Str(value.repeat(count)))
        }
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

pub fn pad_start(arguments: &[Value]) -> Result<Value, EvalError> {
    let [Value::Str(value), Value::Int(width), Value::Str(padding)] = arguments else {
        return Err(EvalError::TypeMismatch { span: None });
    };
    if *width < 0 || padding.is_empty() {
        return Err(EvalError::TypeMismatch { span: None });
    }

    let Ok(width) = usize::try_from(*width) else {
        return Err(EvalError::Overflow { span: None });
    };
    let length = value.chars().count();
    let missing = width.saturating_sub(length);
    let mut result = padding.chars().cycle().take(missing).collect::<String>();
    result.push_str(value);
    Ok(Value::Str(result))
}

pub fn pad_end(arguments: &[Value]) -> Result<Value, EvalError> {
    let [Value::Str(value), Value::Int(width), Value::Str(padding)] = arguments else {
        return Err(EvalError::TypeMismatch { span: None });
    };
    if *width < 0 || padding.is_empty() {
        return Err(EvalError::TypeMismatch { span: None });
    }

    let Ok(width) = usize::try_from(*width) else {
        return Err(EvalError::Overflow { span: None });
    };
    let length = value.chars().count();
    let missing = width.saturating_sub(length);
    let mut result = value.clone();
    result.extend(padding.chars().cycle().take(missing));
    Ok(Value::Str(result))
}

pub fn reverse(arguments: &[Value]) -> Result<Value, EvalError> {
    match arguments {
        [Value::Str(value)] => Ok(Value::Str(value.chars().rev().collect::<String>())),
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        eval::{Scope, evaluate_ast, stdlib},
        parser::Parser,
        value,
    };

    fn evaluate(input: &str) -> Result<Value, EvalError> {
        let ast = Parser::new(input).parse_stmnts().expect("valid input");
        let mut scope = Scope::child(stdlib::new());
        evaluate_ast(&ast, &mut scope)
    }

    fn evaluate_error(input: &str) -> String {
        evaluate(input)
            .expect_err("expected an evaluation error")
            .to_string()
    }

    #[test]
    fn methods_can_be_invoked() {
        let cases = [
            ("'héllo'.len_chars()", value!(5)),
            ("'héllo'.len_bytes()", value!(6)),
            ("''.is_empty()", value!(true)),
            ("' \\t\\n'.is_blank()", value!(true)),
            ("'hello'.is_ascii()", value!(true)),
            ("'hello'.contains('ell')", value!(true)),
            ("'hello'.starts_with('he')", value!(true)),
            ("'hello'.ends_with('lo')", value!(true)),
            ("'hello'.remove_prefix('he')", value!("llo")),
            ("'hello'.remove_suffix('lo')", value!("hel")),
            ("'hello hello'.count('hello')", value!(2)),
            ("'héllo'.find('ll')", value!(2)),
            ("'a,b,c'.split(',')", value!(["a", "b", "c"])),
            ("'one\\ntwo'.lines()", value!(["one", "two"])),
            ("'  hello  '.trim()", value!("hello")),
            ("'  hello  '.trim_start()", value!("hello  ")),
            ("'  hello  '.trim_end()", value!("  hello")),
            ("'héllo'.uppercase()", value!("HÉLLO")),
            ("'HÉLLO'.lowercase()", value!("héllo")),
            ("'hello hello'.replace('hello', 'hi')", value!("hi hi")),
            ("'ha'.repeat(3)", value!("hahaha")),
            ("'7'.pad_start(3, '0')", value!("007")),
            ("'7'.pad_end(3, '0')", value!("700")),
            ("'héllo'.reverse()", value!("olléh")),
        ];
        for (expression, expected) in cases {
            assert_eq!(evaluate(expression), Ok(expected), "{expression}");
        }
    }

    #[test]
    fn rejects_invalid_string_method_arguments() {
        assert_eq!(
            evaluate_error("1.len_chars()"),
            "unknown function `len_chars`"
        );
        assert_eq!(evaluate_error("\"hello\".contains(1)"), "type mismatch");
        assert_eq!(evaluate_error("\"hello\".repeat(-1)"), "type mismatch");
    }
}
