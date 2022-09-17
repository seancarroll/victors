use std::collections::HashMap;
use std::marker::PhantomData;
use serde::Serialize;

use crate::experiment_result::ExperimentResult;

// https://github.com/ex0dus-0x/structmap/blob/master/src/value.rs


// serde
// ScientistValue / PublisherValue /
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

/// Represents the numeric primitive types that are supported for conversion.
#[derive(Debug, Clone)]
pub enum Number {
    I64(i64),
    U64(u64),
    F64(f64),
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


pub trait Publisher<R: Clone + PartialEq + Serialize> {
    fn publish(&self, result: &ExperimentResult<R>);
}

pub struct NoopPublisher;
impl<R: Clone + PartialEq + Serialize> Publisher<R> for NoopPublisher {
    fn publish(&self, _result: &ExperimentResult<R>) {}
}

pub(crate) struct InMemoryPublisher<R: Clone + PartialEq + Serialize, CB>
where
    CB: FnOnce(&ExperimentResult<R>) + Copy,
{
    phantom: PhantomData<R>,
    pub cb: CB,
}

impl<R: Clone + PartialEq + Serialize, CB> InMemoryPublisher<R, CB>
where
    CB: FnOnce(&ExperimentResult<R>) + Copy,
{
    pub fn new(block: CB) -> Self {
        Self {
            phantom: PhantomData,
            cb: block,
        }
    }
}

impl<R: Clone + PartialEq + Serialize, CB> Publisher<R> for InMemoryPublisher<R, CB>
where
    CB: FnOnce(&ExperimentResult<R>) + Copy,
{
    fn publish(&self, result: &ExperimentResult<R>) {
        (self.cb)(result);
    }
}
