use crate::ssa::node::NodeId;
use iter_extended::vecmap;
use noirc_frontend::monomorphization::ast::Type;

/// `Value` is used only to construct the SSA IR.
#[derive(Debug, Clone)]
pub enum Value {
    Single(NodeId),
    Tuple(Vec<Value>),
}

impl Value {
    /// Returns a single NodeId.
    /// Panics: If `Value` holds multiple Values
    pub fn unwrap_id(&self) -> NodeId {
        match self {
            Value::Single(id) => *id,
            Value::Tuple(_) => panic!("Tried to unwrap a struct into a single value"),
        }
    }

    /// Returns a placeholder NodeId that can
    /// be used to represent the absence of a value.
    pub fn dummy() -> Value {
        Value::Single(NodeId::dummy())
    }

    /// Checks if the `Value` corresponds to
    /// `Option::None` or no value.
    pub fn is_dummy(&self) -> bool {
        match self {
            Value::Single(id) => *id == NodeId::dummy(),
            _ => false,
        }
    }

    /// Converts `Value` into a vector of NodeId's
    pub fn to_node_ids(&self) -> Vec<NodeId> {
        match self {
            Value::Single(id) => vec![*id],
            Value::Tuple(v) => v.iter().flat_map(|i| i.to_node_ids()).collect(),
        }
    }

    /// Calls the function `f` on `self` and the given `Value`
    /// Panics: If `self` and the given value are not the same
    /// enum variant
    pub fn zip<F>(&self, rhs_value: &Value, f: &mut F) -> Value
    where
        F: FnMut(NodeId, NodeId) -> NodeId,
    {
        if self.is_dummy() || rhs_value.is_dummy() {
            return Value::dummy();
        }

        match (self, rhs_value) {
            (Value::Single(lhs), Value::Single(rhs)) => Value::Single(f(*lhs, *rhs)),
            (Value::Tuple(lhs), Value::Tuple(rhs)) => {
                Value::Tuple(vecmap(lhs.iter().zip(rhs), |(lhs_value, rhs_value)| {
                    lhs_value.zip(rhs_value, f)
                }))
            }
            _ => {
                unreachable!("ICE: expected both `Value` instances to be of the same enum variant")
            }
        }
    }

    /// Returns the `Value` at position `field_index` in the
    /// Tuple Variant.
    /// Panics: If the `self` is not the `Tuple` Variant.
    pub fn into_field_member(self, field_index: usize) -> Value {
        match self {
            Value::Single(_) => {
                unreachable!("Runtime type error, expected struct but found a single value")
            }
            Value::Tuple(mut fields) => fields.remove(field_index),
        }
    }
    pub fn get_field_member(&self, field_index: usize) -> &Value {
        match self {
            Value::Single(_) => {
                unreachable!("Runtime type error, expected struct but found a single value")
            }
            Value::Tuple(fields) => &fields[field_index],
        }
    }

    /// Reconstruct a `Value` instance whose type is `value_type`
    fn reshape(value_type: &Type, iter: &mut core::slice::Iter<NodeId>) -> Value {
        match value_type {
            Type::Tuple(tup) => {
                let values = vecmap(tup, |v| Self::reshape(v, iter));
                Value::Tuple(values)
            }
            Type::Unit
            | Type::Function(..)
            | Type::Array(..)
            | Type::String(..)
            | Type::Integer(..)
            | Type::Bool
            | Type::Field => Value::Single(*iter.next().unwrap()),
        }
    }

    /// Reconstruct a `Value` instance from a slice of NodeId's
    pub(crate) fn from_slice(value_type: &Type, slice: &[NodeId]) -> Value {
        let mut iter = slice.iter();
        let result = Value::reshape(value_type, &mut iter);
        assert!(iter.next().is_none());
        result
    }
}
