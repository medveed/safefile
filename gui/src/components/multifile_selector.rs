use std::ffi::OsStr;
use std::path::PathBuf;

use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use rfd::FileDialog;

#[derive(Debug, Clone)]
pub enum MultiFileSelectorMessage {
    AddFiles,
    RemoveFile(usize),
    ClearAll,
}

pub struct MultiFileSelector {
    label: String,
    files: Vec<PathBuf>,
}

impl MultiFileSelector {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            files: Vec::new(),
        }
    }

    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }

    pub fn update(&mut self, message: MultiFileSelectorMessage) {
        match message {
            MultiFileSelectorMessage::AddFiles => {
                let selected_files = FileDialog::new().set_title("Выберите файлы").pick_files();

                if let Some(files) = selected_files {
                    self.files.extend(files);
                }
            }
            MultiFileSelectorMessage::RemoveFile(index) => {
                if index < self.files.len() {
                    self.files.remove(index);
                }
            }
            MultiFileSelectorMessage::ClearAll => {
                self.files.clear();
            }
        }
    }

    pub fn view(&self) -> Element<'_, MultiFileSelectorMessage> {
        let files_list = if self.files.is_empty() {
            column![text("Файлы не выбраны")].height(Length::Fill)
        } else {
            let files_column = self.files.iter().enumerate().fold(
                Column::new().spacing(5),
                |column, (index, path)| {
                    let filename = path
                        .file_name()
                        .unwrap_or(OsStr::new("Empty Name"))
                        .to_str()
                        .unwrap_or("Invalid Name");

                    column.push(
                        row![
                            button("✕")
                                .on_press(MultiFileSelectorMessage::RemoveFile(index))
                                .style(button::danger),
                            text(format!("{}. {}", index + 1, filename)).width(Length::Fill)
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    )
                },
            );

            column![
                scrollable(
                    container(files_column)
                        .width(Length::Fill)
                        .style(container::rounded_box)
                )
                .height(Length::Fill)
            ]
        };

        let buttons_row = row![
            button("Добавить")
                .padding(8)
                .on_press(MultiFileSelectorMessage::AddFiles)
                .width(Length::FillPortion(1)),
            button("Очистить")
                .padding(8)
                .on_press(MultiFileSelectorMessage::ClearAll)
                .width(Length::FillPortion(1))
                .style(button::secondary),
        ]
        .spacing(10)
        .width(Length::Fill);

        let content = column![text(&self.label), buttons_row, files_list].spacing(10);

        container(content)
            .style(container::rounded_box)
            .width(Length::Fill)
            .into()
    }
}
