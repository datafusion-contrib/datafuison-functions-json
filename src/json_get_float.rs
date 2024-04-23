use std::any::Any;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array};
use arrow_schema::DataType;
use datafusion_common::{Result as DataFusionResult, ScalarValue};
use datafusion_expr::{ColumnarValue, ScalarUDFImpl, Signature, Volatility};
use jiter::{NumberAny, Peek};

use crate::common_get::{check_args, get_err, get_invoke, jiter_json_find, GetError, JsonPath};
use crate::common_macros::make_udf_function;

make_udf_function!(
    JsonGetFloat,
    json_get_float,
    json_data path, // arg name
    r#"Get a float value from a JSON object by it's "path""#
);

#[derive(Debug)]
pub(super) struct JsonGetFloat {
    signature: Signature,
    aliases: Vec<String>,
}

impl Default for JsonGetFloat {
    fn default() -> Self {
        Self {
            signature: Signature::variadic_any(Volatility::Immutable),
            aliases: vec!["json_get_float".to_string()],
        }
    }
}

impl ScalarUDFImpl for JsonGetFloat {
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
        check_args(arg_types, self.name()).map(|()| DataType::Float64)
    }

    fn invoke(&self, args: &[ColumnarValue]) -> DataFusionResult<ColumnarValue> {
        get_invoke::<Float64Array, f64>(
            args,
            jiter_json_get_float,
            |c| Ok(Arc::new(c) as ArrayRef),
            ScalarValue::Float64,
        )
    }

    fn aliases(&self) -> &[String] {
        &self.aliases
    }
}

fn jiter_json_get_float(json_data: Option<&str>, path: &[JsonPath]) -> Result<f64, GetError> {
    if let Some((mut jiter, peek)) = jiter_json_find(json_data, path) {
        match peek {
            // numbers are represented by everything else in peek, hence doing it this way
            Peek::Null
            | Peek::True
            | Peek::False
            | Peek::Minus
            | Peek::Infinity
            | Peek::NaN
            | Peek::String
            | Peek::Array
            | Peek::Object => get_err!(),
            _ => match jiter.known_number(peek)? {
                NumberAny::Float(f) => Ok(f),
                NumberAny::Int(int) => Ok(int.into()),
            },
        }
    } else {
        get_err!()
    }
}
