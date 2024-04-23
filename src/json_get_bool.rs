use std::any::Any;
use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray};
use arrow_schema::DataType;
use datafusion_common::{Result as DataFusionResult, ScalarValue};
use datafusion_expr::{ColumnarValue, ScalarUDFImpl, Signature, Volatility};
use jiter::Peek;

use crate::common_get::{check_args, get_invoke, jiter_json_find, GetError, JsonPath};
use crate::common_macros::make_udf_function;

make_udf_function!(
    JsonGetBool,
    json_get_bool,
    json_data path, // arg name
    r#"Get an boolean value from a JSON object by it's "path""#
);

#[derive(Debug)]
pub(super) struct JsonGetBool {
    signature: Signature,
    aliases: Vec<String>,
}

impl Default for JsonGetBool {
    fn default() -> Self {
        Self {
            signature: Signature::variadic_any(Volatility::Immutable),
            aliases: vec!["json_get_bool".to_string()],
        }
    }
}

impl ScalarUDFImpl for JsonGetBool {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        self.aliases[0].as_str()
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, arg_types: &[DataType]) -> DataFusionResult<DataType> {
        check_args(arg_types, self.name()).map(|()| DataType::Boolean)
    }

    fn invoke(&self, args: &[ColumnarValue]) -> DataFusionResult<ColumnarValue> {
        get_invoke::<BooleanArray, bool>(
            args,
            jiter_json_get_bool,
            |c| Ok(Arc::new(c) as ArrayRef),
            ScalarValue::Boolean,
        )
    }

    fn aliases(&self) -> &[String] {
        &self.aliases
    }
}

fn jiter_json_get_bool(json_data: Option<&str>, path: &[JsonPath]) -> Result<bool, GetError> {
    if let Some((mut jiter, peek)) = jiter_json_find(json_data, path) {
        match peek {
            Peek::True | Peek::False => Ok(jiter.known_bool(peek)?),
            _ => Err(GetError),
        }
    } else {
        Err(GetError)
    }
}
