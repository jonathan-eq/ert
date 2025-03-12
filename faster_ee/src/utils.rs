use serde::{Deserialize, Serialize};

// Macro to update fields if `Some` is available
#[macro_export]
macro_rules! update_field_if_set {
    ($self:ident, $other:ident, $field:ident) => {
        if let Some(value) = &$other.$field {
            $self.$field = Some(value.clone());
        }
    };
}

#[macro_export]
macro_rules! update_field_if_not_empty {
    ($self:ident, $other:ident, $field:ident) => {
        $self.$field.extend($other.$field.clone());
    };
}
