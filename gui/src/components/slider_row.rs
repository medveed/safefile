use iced::widget::{Container, column, row, slider, text};
use iced::{Alignment, Element, Font, Length};

#[derive(Debug, Clone)]
pub struct SliderRowMessage(pub u32);

pub struct SliderRow {
    label: String,
    value: u32,
    min: u32,
    max: u32,
}

impl SliderRow {
    pub fn new(label: &str, min: u32, max: u32) -> Self {
        Self {
            label: label.to_string(),
            value: min,
            min,
            max,
        }
    }

    pub fn with_value(mut self, value: u32) -> Self {
        self.value = value;
        self
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn update(&mut self, message: SliderRowMessage) {
        self.value = message.0;
    }

    pub fn view(&self) -> Element<'_, SliderRowMessage> {
        let content = column![
            text(format!("{}:", self.label))
                .font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::default()
                }),
            row![
                text(self.value.to_string()).width(40),
                slider(self.min..=self.max, self.value, |v| {
                    SliderRowMessage(v)
                })
            ]
            .align_y(Alignment::Center)
            .spacing(10)
        ]
        .spacing(10);

        Container::new(content).width(Length::Fill).into()
    }
}
