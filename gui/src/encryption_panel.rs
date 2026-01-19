use iced::widget::{Space, button, column, container, text};
use iced::{Element, Length};

use crate::PanelCommand;
use crate::components::file_row::{FileRow, FileRowMessage};
use crate::components::slider_row::{SliderRow, SliderRowMessage};
use crate::components::text_row::{TextRow, TextRowMessage};

#[derive(Debug, Clone)]
pub enum EncryptionPanelMessage {
    TotalSharesChanged(u32),
    ThresholdChanged(u32),
    LabelChanged(String),
    Encrypt,
    FileRowInput(FileRowMessage),
    FileRowOutput(FileRowMessage),
    FileRowOutputDir(FileRowMessage),
}

pub struct EncryptionPanel {
    input_file_row: FileRow,
    output_file_row: FileRow,
    output_dir_row: FileRow,
    total_shares_row: SliderRow,
    threshold_row: SliderRow,
    label_row: TextRow,
}

impl Default for EncryptionPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptionPanel {
    pub fn new() -> Self {
        Self {
            input_file_row: FileRow::new("Исходный файл"),
            output_file_row: FileRow::new("Выходной файл").save_mode(true),
            output_dir_row: FileRow::new("Папка для долей").directory_mode(true),
            total_shares_row: SliderRow::new("Всего долей", 2, 10).with_value(5),
            threshold_row: SliderRow::new("Порог для восстановления", 2, 10).with_value(3),
            label_row: TextRow::new("Метка секрета", "Опционально"),
        }
    }

    pub fn get_params(&self) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf, u8, u8, Option<String>) {
        let input = self.input_file_row.value().clone();
        let output = self.output_file_row.value().clone();
        let outdir = self.output_dir_row.value().clone();
        let shares = self.total_shares_row.value() as u8;
        let threshold = self.threshold_row.value() as u8;
        let label = if self.label_row.value().is_empty() { None } else { Some(self.label_row.value().to_string()) };
        (input, output, outdir, shares, threshold, label)
    }

    pub fn update(&mut self, message: EncryptionPanelMessage) -> PanelCommand {
        match message {
            EncryptionPanelMessage::FileRowInput(msg) => {
                self.input_file_row.update(msg);
                PanelCommand::None
            }
            EncryptionPanelMessage::FileRowOutput(msg) => {
                self.output_file_row.update(msg);
                PanelCommand::None
            }
            EncryptionPanelMessage::FileRowOutputDir(msg) => {
                self.output_dir_row.update(msg);
                PanelCommand::None
            }
            EncryptionPanelMessage::TotalSharesChanged(value) => {
                self.total_shares_row.update(SliderRowMessage(value));
                PanelCommand::None
            }
            EncryptionPanelMessage::ThresholdChanged(value) => {
                if value <= self.total_shares_row.value() {
                    self.threshold_row.update(SliderRowMessage(value));
                }
                PanelCommand::None
            }
            EncryptionPanelMessage::LabelChanged(label) => {
                self.label_row.update(TextRowMessage { text: label });
                PanelCommand::None
            }
            _ => PanelCommand::None
        }
    }

    pub fn view(&self) -> Element<'_, EncryptionPanelMessage> {
        let ready_to_encrypt = !self.input_file_row.is_empty()
            && !self.output_file_row.is_empty()
            && !self.output_dir_row.is_empty()
            && self.threshold_row.value() <= self.total_shares_row.value();

        let encrypt_button = button(text("Зашифровать"))
            .padding(12)
            .width(Length::Fill)
            .on_press_maybe(if ready_to_encrypt {
                Some(EncryptionPanelMessage::Encrypt)
            } else {
                None
            });

        let content = column![
            text("Зашифровать и разделить").size(24),
            self.input_file_row
                .view()
                .map(EncryptionPanelMessage::FileRowInput),
            self.output_file_row
                .view()
                .map(EncryptionPanelMessage::FileRowOutput),
            self.output_dir_row
                .view()
                .map(EncryptionPanelMessage::FileRowOutputDir),
            self.total_shares_row
                .view()
                .map(|m| EncryptionPanelMessage::TotalSharesChanged(m.0)),
            self.threshold_row
                .view()
                .map(|m| EncryptionPanelMessage::ThresholdChanged(m.0)),
            self.label_row
                .view()
                .map(|m| EncryptionPanelMessage::LabelChanged(m.text)),
            Space::new().height(Length::Fill),
            encrypt_button
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill);

        container(content)
            .style(container::rounded_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
