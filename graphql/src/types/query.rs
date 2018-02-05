/*
Example query:

query {
  human(202) {
    name,
    appearsIn
  }
}
*/
use {QlError, QlResult};
use execution::Context;
use parser::parse_query::parse_query;
use types::{result, schema, Id, Name};

use std::collections::HashMap;

pub type Variables = HashMap<String, Value>;

// TODO variables, directives
#[derive(Clone, Debug)]
pub enum Operation {
    Query(Field),
    // TODO
    Mutation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: Name,
    pub alias: Option<Name>,
    pub args: Vec<(Name, Value)>,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Null,
    String(String),
    // A Name is basically any unquoted string, it might get used as an Id or whatever.
    Name(Name),
    Array(Vec<Value>),
    // TODO input object types
}

pub trait Root: result::Resolve {
    fn schema() -> schema::Schema;
}

impl Operation {
    pub fn parse(input: &str) -> QlResult<Operation> {
        parse_query(input)
    }

    pub fn validate(&self, schema: &schema::Schema) -> QlResult<()> {
        // TODO - we will need to return some type info about the query I think, or save that in self
        // FIXME should generate validation statically, rather than using the dynamic schema
        ::validation::validate_query(self, schema)
    }

    // TODO don't need schema to execute?
    pub fn execute<R: Root>(&self, variables: Variables, _schema: &schema::Schema, root: R) -> QlResult<result::Value> {
        let ctxt = Context::new(variables, self.get_field().clone());
        // TODO use ctxt
        match *self {
            // TODO change resolve sig to just take a single field.
            Operation::Query(ref f) => root.resolve(&[f.clone()]),
            _ => unimplemented!(),
        }
    }

    pub fn get_field(&self) -> &Field {
        match *self {
            Operation::Query(ref f) => f,
            Operation::Mutation => unreachable!(),
        }
    }
}

impl Field {
    pub fn find_arg(&self, name: &Name) -> Option<&Value> {
        for a in &self.args {
            if &a.0 == name {
                return Some(&a.1);
            }
        }

        None
    }
}

pub trait FromValue: Sized {
    fn from(value: &Value) -> QlResult<Self>;
}

impl FromValue for String {
    fn from(value: &Value) -> QlResult<String> {
        if let Value::String(ref s) = *value {
            Ok(s.clone())
        } else {
            Err(QlError::TranslationError(
                format!("{:?}", value),
                "String".to_owned(),
            ))
        }
    }
}
impl FromValue for Id {
    fn from(value: &Value) -> QlResult<Id> {
        if let Value::Name(ref n) = *value {
            Ok(Id(n.0.clone()))
        } else {
            Err(QlError::TranslationError(
                format!("{:?}", value),
                "Id".to_owned(),
            ))
        }
    }
}
impl FromValue for Name {
    fn from(value: &Value) -> QlResult<Name> {
        if let Value::Name(ref n) = *value {
            Ok(n.clone())
        } else {
            Err(QlError::TranslationError(
                format!("{:?}", value),
                "Name".to_owned(),
            ))
        }
    }
}
impl<T: FromValue> FromValue for Vec<T> {
    fn from(value: &Value) -> QlResult<Vec<T>> {
        if let Value::Array(ref a) = *value {
            a.iter().map(|x| T::from(x)).collect()
        } else {
            Err(QlError::TranslationError(
                format!("{:?}", value),
                "Array".to_owned(),
            ))
        }
    }
}
impl<T: FromValue> FromValue for Option<T> {
    fn from(value: &Value) -> QlResult<Option<T>> {
        if let Value::Null = *value {
            Ok(None)
        } else {
            Ok(Some(T::from(value)?))
        }
    }
}
