use serde_json::Number as NumberValue;
use serde_json::{
    Map as JsonMap,
    Value::{self, Bool, Number, Object},
};
use Dim::*;

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

impl ParamsSpec {
    pub fn from_json(json: Value) -> Result<ParamsSpec, String> {
        // TODO: place holder supporting only real number dims. Generalize and clean up.
        if let Some(values) = json.as_object() {
            if let Some(Object(inital_guess)) = values.get("initial_guess") {
                if let Some(Object(definition)) = values.get("definition") {
                    let mut dims = Vec::new();
                    for (param_name, value) in definition {
                        if let Some(bounds) = value.as_array() {
                            if bounds.len() != 2 {
                                return Err(
                                    "Bounds array must have exactly two elements".to_string()
                                );
                            }

                            if let (Some(lower_bound), Some(upper_bound)) =
                                (bounds[0].as_f64(), bounds[1].as_f64())
                            {
                                if let Some(initial_guess_value) = inital_guess.get(param_name) {
                                    if let Number(initial_guess_value) = initial_guess_value {
                                        let initial_guess_value =
                                            initial_guess_value.as_f64().unwrap();

                                        dims.push(RealNumber(DimSpecWithBounds::new(
                                            param_name.clone(),
                                            initial_guess_value,
                                            lower_bound,
                                            upper_bound,
                                        )))
                                    } else {
                                        return Err(format!(
                                            "Initial guess property {} not a number",
                                            param_name
                                        )
                                        .to_string());
                                    }
                                } else {
                                    return Err(format!("Initial guess not aligned with definition. Property {} not found in initial guess.", param_name));
                                }
                            } else {
                                return Err(format!(
                                    "Bounds for param {} are not numbers",
                                    param_name
                                ));
                            }
                        } else {
                            return Err("Values of definition object must be arrays".to_string());
                        }
                    }
                    Ok(ParamsSpec {dims})
                } else {
                    Err("Missing definition property".to_string())
                }
            } else {
                Err("Missing initial_guess property".to_string())
            }
        } else {
            Err("Spec json is not an object".to_string())
        }
    }
}

pub type ParamsValue = JsonMap<String, Value>;
impl ParamsSpec {
    pub fn extract_initial_guess(&self) -> ParamsValue {
        let mut result = ParamsValue::new();

        for dim in &self.dims {
            match dim {
                Boolean(dim_spec) => {
                    result.insert(dim_spec.name.clone(), Bool(dim_spec.initial_value));
                }
                RealNumber(dim_spec_with_bounds) => {
                    result.insert(
                        dim_spec_with_bounds.dim_spec.name.clone(),
                        Number(
                            NumberValue::from_f64(dim_spec_with_bounds.dim_spec.initial_value)
                                .unwrap(),
                        ),
                    );
                }
                Integer(dim_spec_with_bounds) => {
                    result.insert(
                        dim_spec_with_bounds.dim_spec.name.clone(),
                        Number(NumberValue::from(
                            dim_spec_with_bounds.dim_spec.initial_value,
                        )),
                    );
                }
            }
        }
        result
    }
}
