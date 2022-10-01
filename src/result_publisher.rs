use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use serde::Serialize;

use crate::experiment_result::ExperimentResult;

// https://github.com/ex0dus-0x/structmap/blob/master/src/value.rs


// serde
// ScientistValue / PublisherValue /
#[derive(Clone, PartialEq, Serialize)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

/// Represents the numeric primitive types that are supported for conversion.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Number {
    I64(i64),
    U64(u64),
    F64(f64),
}


impl Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => formatter.write_str("Null"),
            Value::Bool(boolean) => write!(formatter, "Bool({})", boolean),
            Value::Number(number) => Debug::fmt(number, formatter),
            Value::String(string) => write!(formatter, "String({:?})", string),
            Value::Array(vec) => {
                formatter.write_str("Array ")?;
                Debug::fmt(vec, formatter)
            }
            Value::Object(map) => {
                formatter.write_str("Object ")?;
                Debug::fmt(map, formatter)
            }
        }
    }
}

impl Value {
    /// Given a genericized input type, encapsulate it as a Value that can be used in a map
    /// container type when converting to and from a struct.
    pub fn new<T: Any>(value: T) -> Value {
        let any_val = &value as &dyn Any;
        if let Some(val) = any_val.downcast_ref::<bool>() {
            Value::Bool(*val)
        } else if let Some(val) = any_val.downcast_ref::<i64>() {
            Value::Number(Number::I64(*val))
        } else if let Some(val) = any_val.downcast_ref::<u64>() {
            Value::Number(Number::U64(*val))
        } else if let Some(val) = any_val.downcast_ref::<f64>() {
            Value::Number(Number::F64(*val))
        } else if let Some(val) = any_val.downcast_ref::<&'static str>() {
            Value::String(val.to_string())
        } else if let Some(val) = any_val.downcast_ref::<String>() {
            Value::String(val.to_string())
        } else if let Some(val) = any_val.downcast_ref::<Vec<Value>>() {
            Value::Array(val.to_vec())
        } else if let Some(val) = any_val.downcast_ref::<HashMap<String, Value>>() {
            Value::Object(val.clone()) // TODO: fix clone
        } else {
            Value::Null
        }
    }

    pub fn bool(&self) -> Option<bool> {
        if let Value::Bool(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn i64(&self) -> Option<i64> {
        if let Value::Number(Number::I64(val)) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn u64(&self) -> Option<u64> {
        if let Value::Number(Number::U64(val)) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn f64(&self) -> Option<f64> {
        if let Value::Number(Number::F64(val)) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn string(&self) -> Option<String> {
        if let Value::String(string) = self {
            Some(string.to_string())
        } else {
            None
        }
    }
}

// #[inline]
// pub fn to_string<T>(value: &T) -> Result<String>
//     where
//         T: ?Sized + Serialize,
// {
//     let vec = tri!(to_vec(value));
//     let string = unsafe {
//         // We do not emit invalid UTF-8.
//         String::from_utf8_unchecked(vec)
//     };
//     Ok(string)
// }

// #[inline]
// pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
//     where
//         T: ?Sized + Serialize,
// {
//     let mut writer = Vec::with_capacity(128);
//     tri!(to_writer(&mut writer, value));
//     Ok(writer)
// }


// https://github.com/open-telemetry/opentelemetry-rust/blob/f20c9b40547ee20b6ec99414bb21abdd3a54d99b/opentelemetry-api/src/common.rs

// #[derive(Clone, Debug, PartialEq)]
// pub enum Value {
//     /// bool values
//     Bool(bool),
//     I8(i8),
//     U8(u8),
//     I16(i16),
//     U16(u16),
//     I32(i32),
//     U32(u32),
//     I64(i64),
//     U64(u64),
//     I128(i128),
//     U128(u128),
//     ISize(isize),
//     USize(usize),
//     /// i64 values
//     I64(i64),
//     /// f32 values
//     F32(f32),
//     /// f64 values
//     F64(f64),
//     /// String values
//     String(StringValue),
//     /// Array of homogeneous values
//     Array(Array),
// }


// #[derive(Clone, Debug, PartialEq)]
// pub enum Array {
//     /// Array of bools
//     Bool(Vec<bool>),
//     /// Array of integers
//     I64(Vec<i64>),
//     /// Array of floats
//     F64(Vec<f64>),
//     /// Array of strings
//     String(Vec<StringValue>),
// }


pub trait Publisher {
    fn publish(&self, result: &ExperimentResult);
}

pub struct NoopPublisher;
impl Publisher for NoopPublisher {
    fn publish(&self, _: &ExperimentResult) {}
}

pub(crate) struct InMemoryPublisher<CB>
where
    CB: FnOnce(&ExperimentResult) + Copy,
{
    // phantom: PhantomData<R>,
    pub cb: CB,
}

impl<CB> InMemoryPublisher<CB>
where
    CB: FnOnce(&ExperimentResult) + Copy,
{
    pub fn new(block: CB) -> Self {
        Self {
            // phantom: PhantomData,
            cb: block,
        }
    }
}

impl<CB> Publisher for InMemoryPublisher<CB>
where
    CB: FnOnce(&ExperimentResult) + Copy,
{
    fn publish(&self, result: &ExperimentResult) {
        (self.cb)(result);
    }
}
