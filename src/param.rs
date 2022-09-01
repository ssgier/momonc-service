use Dim::*;
use serde_json::{Map as JsonMap, Value::{self, Bool, Number}};
use serde_json::Number as NumberValue;

#[derive(Debug)]
pub enum Dim {
    Boolean(DimSpec<bool>),
    RealNumber(DimSpecWithBounds<f64>),
    Integer(DimSpecWithBounds<i64>),
}

#[derive(Debug)]
pub struct DimSpec<T> {
    pub name: String,
    pub initial_value: T,
}

#[derive(Debug)]
pub struct DimSpecWithBounds<T> {
    pub dim_spec: DimSpec<T>,
    pub min_value_incl: T,
    pub max_value_excl: T,
}

impl<T> DimSpecWithBounds<T> {
    pub fn new(
        name: String,
        initial_value: T,
        min_value_incl: T,
        max_value_excl: T,
    ) -> DimSpecWithBounds<T> {
        DimSpecWithBounds {
            dim_spec: DimSpec {
                name,
                initial_value,
            },
            min_value_incl,
            max_value_excl,
        }
    }
}

#[derive(Debug)]
pub struct ParamsSpec {
    pub dims: Vec<Dim>,
}

pub type ParamsValue = JsonMap<String, Value>;
impl ParamsSpec {
    pub fn extract_initial_guess(&self) -> ParamsValue {

        let mut result  = ParamsValue::new();

        for dim in &self.dims {
            match dim {
                Boolean(dim_spec) => {
                    result.insert(dim_spec.name.clone(), Bool(dim_spec.initial_value));
                }
                RealNumber(dim_spec_with_bounds) => {
                    result.insert(
                        dim_spec_with_bounds.dim_spec.name.clone(),
                        Number(NumberValue::from_f64(dim_spec_with_bounds.dim_spec.initial_value).unwrap()),
                    );
                }
                Integer(dim_spec_with_bounds) => {
                    result.insert(
                        dim_spec_with_bounds.dim_spec.name.clone(),
                        Number(NumberValue::from(dim_spec_with_bounds.dim_spec.initial_value)),
                    );
                }
            }
        }
        result
    }
}

