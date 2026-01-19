use iced::widget::{button, column, container,text};
use iced::{Element, Length};
use rfd::FileDialog;

use crate::components::file_row::{FileRow, FileRowMessage};
use crate::components::multifile_selector::{MultiFileSelector, MultiFileSelectorMessage};
use crate::PanelCommand;

#[derive(Debug, Clone)]
pub enum DecryptionPanelMessage {
    FileRowInput(FileRowMessage),
    FileRowOutput(FileRowMessage),
    SharesSelector(MultiFileSelectorMessage),
    Decrypt,
}

pub struct DecryptionPanel {
    input_file_row: FileRow,
    output_file_row: FileRow,
    shares_selector: MultiFileSelector,
}

impl Default for DecryptionPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DecryptionPanel {
    pub fn new() -> Self {
        Self {
            input_file_row: FileRow::new("Зашифрованный файл"),
            output_file_row: FileRow::new("Выходной файл").save_mode(true),
            shares_selector: MultiFileSelector::new("Выбранные доли"),
        }
    }

    pub fn get_params(&self) -> (std::path::PathBuf, std::path::PathBuf, Vec<std::path::PathBuf>) {
        let input = self.input_file_row.value().clone();
        let output = self.output_file_row.value().clone();
        let shares = self.shares_selector.files().iter().cloned().collect();
        (input, output, shares)
    }

    pub fn update(&mut self, message: DecryptionPanelMessage) -> PanelCommand {
        match message {
            DecryptionPanelMessage::SharesSelector(msg) => {
                self.shares_selector.update(msg);
                PanelCommand::None
            }
            DecryptionPanelMessage::FileRowInput(FileRowMessage::SelectFile) => {
                let dialog = FileDialog::new();

                let file = dialog.pick_file();
                if file == None {
                    return PanelCommand::None;
                }
                let file = file.unwrap();

                let info = safefile::format::inspect_safe_from_path(&file);
                if let Err(e) = info {
                    return PanelCommand::ShowError(format!("The selected file is not a valid SafeFile: {}", e));
                }
                self.input_file_row.update(FileRowMessage::FileSelected(Some(file)));
                PanelCommand::None
            }
            DecryptionPanelMessage::FileRowInput(msg) => {
                self.input_file_row.update(msg);
                PanelCommand::None
            }
            DecryptionPanelMessage::FileRowOutput(msg) => {
                self.output_file_row.update(msg);
                PanelCommand::None
            }
            _ => PanelCommand::None
        }
    }

    pub fn view(&self) -> Element<'_, DecryptionPanelMessage> {
        let ready_to_decrypt = !self.input_file_row.is_empty()
            && !self.output_file_row.is_empty()
            && self.shares_selector.count() >= 2;

        let decrypt_button = button(text("Расшифровать"))
            .padding(12)
            .width(Length::Fill)
            .on_press_maybe(if ready_to_decrypt {
                Some(DecryptionPanelMessage::Decrypt)
            } else {
                None
            });

        let content = column![
            text("Расшифровать").size(24),
            self.input_file_row
                .view()
                .map(DecryptionPanelMessage::FileRowInput),
            self.output_file_row
                .view()
                .map(DecryptionPanelMessage::FileRowOutput),
            self.shares_selector
                .view()
                .map(DecryptionPanelMessage::SharesSelector),
            decrypt_button,
        ]
        .spacing(20)
        .padding(20);

        container(content)
            .style(container::rounded_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
