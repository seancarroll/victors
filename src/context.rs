// This was based on Tera's Context. See https://github.com/Keats/tera
use std::collections::HashMap;
use serde::ser::Serialize;
use serde_json::{Map, to_value, Value};
use crate::errors::{VictorsErrors, VictorsResult};

/// The struct that holds the context of an experiment.
///
/// Light wrapper around a `HashMap` for easier insertions of Serializable values
#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    data: HashMap<String, Value>,
}

impl Context {
    /// Initializes an empty context
    pub fn new() -> Self {
        Context { data: HashMap::new() }
    }

    /// Converts the `val` parameter to `Value` and insert it into the context.
    ///
    /// Panics if the serialization fails.
    ///
    /// ```rust
    /// # use victors::Context;
    /// let mut context = victors::Context::new();
    /// context.insert("number_users", &42);
    /// ```
    pub fn insert<S: Into<String>, T: Serialize + ?Sized>(&mut self, key: S, val: &T) {
        self.data.insert(key.into(), to_value(val).unwrap());
    }

    /// Converts the `val` parameter to `Value` and insert it into the context.
    ///
    /// Returns an error if the serialization fails.
    pub fn try_insert<S: Into<String>, T: Serialize + ?Sized>(
        &mut self,
        key: S,
        val: &T,
    ) -> VictorsResult<()> {
        self.data.insert(key.into(), to_value(val)?);
        Ok(())
    }

    /// Appends the data of the `source` parameter to current (target) context, overwriting existing keys.
    /// The source context will be dropped.
    ///
    /// ```rust
    /// # use victors::Context;
    /// let mut target = Context::new();
    /// target.insert("a", &1);
    /// target.insert("b", &2);
    /// let mut source = Context::new();
    /// source.insert("b", &3);
    /// source.insert("d", &4);
    /// target.extend(source);
    /// ```
    pub fn extend(&mut self, mut source: Context) {
        self.data.extend( source.data);
    }

    /// Converts the context to a `serde_json::Value` consuming the context.
    pub fn into_json(self) -> Value {
        let mut m = Map::new();
        for (key, value) in self.data {
            m.insert(key, value);
        }
        Value::Object(m)
    }

    /// Takes a serde-json `Value` and convert it into a `Context` with no overhead/cloning.
    pub fn from_value(obj: Value) -> VictorsResult<Self> {
        match obj {
            Value::Object(m) => {
                let mut data = HashMap::new();
                for (key, value) in m {
                    data.insert(key, value);
                }
                Ok(Context { data })
            }
            _ => Err(VictorsErrors::Msg(
                "Creating a Context from a Value/Serialize requires it being a JSON object".to_string(),
            )),
        }
    }

    /// Takes something that impl Serialize and create a context with it.
    /// Meant to be used if you have a hashmap or a struct and don't want to insert values
    /// one by one in the context.
    pub fn from_serialize(value: impl Serialize) -> VictorsResult<Self> {
        let obj = to_value(value).map_err(VictorsErrors::Json)?;
        Context::from_value(obj)
    }

    /// Returns the value at a given key index.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Returns the key-value pair corresponding to the supplied key.
    pub fn get_key_value(&self, key: &str) -> Option<(&String, &Value)> {
        self.data.get_key_value(key)
    }

    /// Checks if a value exists at a specific index.
    pub fn contains_key(&self, index: &str) -> bool {
        self.data.contains_key(index)
    }
}

impl Default for Context {
    fn default() -> Context {
        Context::new()
    }
}
