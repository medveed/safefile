use std::ffi::OsStr;
use std::path::PathBuf;

use iced::widget::{Container, button, column, row, text};
use iced::{Alignment, Element, Font, Length};

use rfd::FileDialog;

#[derive(Debug, Clone)]
pub enum FileRowMessage {
    SelectFile,
    FileSelected(Option<PathBuf>),
}

pub struct FileRow {
    label: String,
    value: PathBuf,
    file_filter: Option<Vec<String>>,
    directory_mode: bool,
    save_mode: bool,
}

impl FileRow {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value: PathBuf::new(),
            file_filter: None,
            directory_mode: false,
            save_mode: false
        }
    }

    pub fn directory_mode(mut self, is_directory: bool) -> Self {
        self.directory_mode = is_directory;
        self
    }

    pub fn save_mode(mut self, is_save: bool) -> Self {
        self.save_mode = is_save;
        self
    }

    pub fn value(&self) -> &PathBuf {
        &self.value
    }

    pub fn is_empty(&self) -> bool {
        self.value.as_os_str().is_empty()
    }

    pub fn update(&mut self, message: FileRowMessage) {
        match message {
            FileRowMessage::SelectFile => {
                if self.directory_mode {
                    let dir = FileDialog::new().pick_folder();
                    if let Some(path) = dir {
                        self.value = path;
                    }
                } else {
                    let mut dialog = FileDialog::new();

                    if let Some(filter) = &self.file_filter {
                        dialog = dialog.add_filter("Files", filter);
                    }

                    let file = if self.save_mode {
                        dialog.save_file()
                    } else {
                        dialog.pick_file()
                    };
                    if let Some(path) = file {
                        self.value = path;
                    }
                }
            }
            FileRowMessage::FileSelected(path) => {
                if let Some(p) = path {
                    self.value = p;
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, FileRowMessage> {
        let display_text = if self.is_empty() {
            if self.directory_mode {
                "Папка не выбрана".to_string()
            } else {
                "Файл не выбран".to_string()
            }
        } else {
            let filename = self
                .value
                .file_name()
                .unwrap_or(OsStr::new("Invalid"))
                .to_string_lossy();
            if filename.len() > 40 {
                let start = &filename[..10];
                let end = &filename[filename.len() - 30..];
                format!("{}...{}", start, end)
            } else {
                filename.to_string()
            }
        };

        let content = column![
            text(format!("{}:", self.label)).font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::default()
            }),
            row![
                text(display_text).width(Length::Fill),
                button("Выбрать")
                    .padding(8)
                    .on_press(FileRowMessage::SelectFile)
            ]
            .align_y(Alignment::Center)
            .spacing(10)
        ];

        Container::new(content).width(Length::Fill).into()
    }
}
