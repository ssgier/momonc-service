use log::debug;

#[derive(Debug)]
pub enum AppState {
    Idle(),
    Processing(),
    Terminal(),
}

#[derive(Debug)]
pub enum AppEvent {
    NewSpec(Spec),
}

#[derive(Debug)]
pub struct TransitionError(String);

struct ProcessingContext {}

#[derive(Debug)]
pub enum Dim {
    Boolean(DimSpec<bool>),
    RealNumber(DimSpecWithBounds<f64>),
    Integer(DimSpecWithBounds<i64>),
}

#[derive(Debug)]
pub struct DimSpec<T> {
    name: String,
    initial_value: T,
}

#[derive(Debug)]
pub struct DimSpecWithBounds<T> {
    dim_spec: DimSpec<T>,
    min_value_incl: T,
    max_value_excl: T,
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
pub struct Spec {
    pub dims: Vec<Dim>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState::Idle()
    }

    fn transition_to(&mut self, new_state: AppState) {
        debug!("Transition to {:?}", new_state);
        *self = new_state;
    }
    pub fn on_event(&mut self, event: AppEvent) -> Result<(), TransitionError> {
        match self {
            AppState::Idle() => match event {
                AppEvent::NewSpec(_spec) => {
                    self.transition_to(AppState::Processing());
                    Ok(())
                }
            },
            AppState::Processing() => match event {
                AppEvent::NewSpec(_) => self.illegal_transition_error(event),
            },
            AppState::Terminal() => match event {
                AppEvent::NewSpec(_) => self.illegal_transition_error(event),
            },
        }
    }

    fn illegal_transition_error(&self, event: AppEvent) -> Result<(), TransitionError> {
        Err(TransitionError(format!(
            "Event {:?} not allowed in state {:?}",
            event, *self
        )))
    }
}
