use egui::{Response, RichText, Ui, Widget};
use polars::prelude::AnyValue;

/// Float value widget
#[derive(Clone, Copy, Debug, Default)]
pub struct FloatValue {
    pub value: Option<f64>,
    pub disable: bool,
    pub hover: bool,
    pub percent: bool,
    pub precision: Option<usize>,
}

impl FloatValue {
    pub fn new(value: Option<f64>) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }

    pub fn disable(self, disable: bool) -> Self {
        Self { disable, ..self }
    }

    pub fn hover(self) -> Self {
        Self {
            hover: true,
            ..self
        }
    }

    pub fn percent(self, percent: bool) -> Self {
        Self { percent, ..self }
    }

    pub fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl Widget for FloatValue {
    fn ui(self, ui: &mut Ui) -> Response {
        if self.disable {
            ui.disable();
        }
        let text = match self.value {
            None => RichText::new(AnyValue::Null.to_string()),
            Some(mut value) => {
                if self.percent {
                    value *= 100.0;
                }
                match self.precision {
                    Some(precision) => RichText::new(format!("{value:.precision$}")),
                    None => RichText::new(AnyValue::from(value).to_string()),
                }
            }
        };
        let mut response = ui.label(text);
        if self.hover {
            let mut value = self.value.unwrap_or_default();
            if self.percent {
                value *= 100.0;
            }
            let text = RichText::new(AnyValue::Float64(value).to_string());
            response = response
                .on_hover_text(text.clone())
                .on_disabled_hover_text(text);
        }
        response
    }
}
