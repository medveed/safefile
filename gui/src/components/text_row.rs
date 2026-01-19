use iced::widget::{Container, column, text, text_input};
use iced::{Element, Font, Length};

#[derive(Debug, Clone)]
pub struct TextRowMessage {
    pub text: String,
}

pub struct TextRow {
    label: String,
    value: String,
    placeholder: String,
}

impl TextRow {
    pub fn new(label: &str, placeholder: &str) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            placeholder: placeholder.to_string(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn update(&mut self, message: TextRowMessage) {
        self.value = message.text;
    }

    pub fn view(&self) -> Element<'_, TextRowMessage> {
        let content = column![
            text(&self.label)
                .font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::default()
                }),
            text_input(&self.placeholder, &self.value)
                .on_input(|text| TextRowMessage { text })
                .width(Length::Fill)
                .padding(10)
        ]
        .spacing(10);

        Container::new(content).width(Length::Fill).into()
    }
}
