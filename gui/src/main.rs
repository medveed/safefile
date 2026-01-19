use iced::widget::{column, row, text};
use iced::{Alignment, Element, Length};

mod components;
mod decryption_panel;
mod encryption_panel;

use decryption_panel::DecryptionPanel;
use encryption_panel::EncryptionPanel;
use iced_dialog::dialog;

use crate::decryption_panel::DecryptionPanelMessage;
use crate::encryption_panel::EncryptionPanelMessage;

pub fn main() -> iced::Result {
    iced::application(SafeFileApp::new, SafeFileApp::update, SafeFileApp::view)
        .title("SafeFile")
        .run()
}

#[derive(Debug)]
pub enum Update {
    Downloading(u64, u64),
    Finished(Result<(), safefile::error::Error>),
}

#[derive(Debug, Clone)]
pub enum PanelCommand {
    ShowError(String),
    None,
}

#[derive(Default)]
struct SafeFileApp {
    encryption_panel: EncryptionPanel,
    decryption_panel: DecryptionPanel,

    error_dialog_open: bool,
    error_dialog_text: String,

    statusbar_message: String,
}

#[derive(Debug, Clone)]
enum Message {
    EncryptionPanel(EncryptionPanelMessage),
    DecryptionPanel(DecryptionPanelMessage),

    ShowErrorDialog(String),
    HideErrorDialog,

    EncryptionFinished(Result<(), String>),
    DecryptionFinished(Result<(), String>),
}

impl SafeFileApp {
    fn new() -> Self {
        Self {
            encryption_panel: EncryptionPanel::new(),
            decryption_panel: DecryptionPanel::new(),
            statusbar_message: "Готов".to_string(),
            ..Default::default()
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::EncryptionPanel(EncryptionPanelMessage::Encrypt) => {
                self.statusbar_message = "Шифрование...".to_string();
                let (input, output, outdir, shares, threshold, label) = self.encryption_panel.get_params();
                iced::Task::perform(
                    async move {
                        match tokio::task::spawn_blocking(move || {
                            safefile::ops::encrypt_and_split(
                                &input,
                                &output,
                                &outdir,
                                shares,
                                threshold,
                                label.as_deref(),
                                |_, _| {}, // TODO: Progress bar
                            )
                        }).await {
                            Ok(Ok(_)) => Ok(()),
                            Ok(Err(e)) => Err(format!("{:?}", e)),
                            Err(join_e) => Err(format!("Join error: {:?}", join_e)),
                        }
                    },
                    Message::EncryptionFinished,
                )
            }
            Message::DecryptionPanel(DecryptionPanelMessage::Decrypt) => {
                self.statusbar_message = "Расшифровка...".to_string();
                let (input, output, shares) = self.decryption_panel.get_params();
                iced::Task::perform(
                    async move {
                        match tokio::task::spawn_blocking(move || {
                            let share_refs: Vec<&std::path::Path> = shares.iter().map(|p| p.as_path()).collect();
                            safefile::ops::decrypt_and_reconstruct(
                                &input,
                                &output,
                                &share_refs,
                                |_, _| {}, // TODO: Progress bar
                            )
                        }).await {
                            Ok(Ok(_)) => Ok(()),
                            Ok(Err(e)) => Err(format!("{:?}", e)),
                            Err(join_e) => Err(format!("Join error: {:?}", join_e)),
                        }
                    },
                    Message::DecryptionFinished,
                )
            }
            Message::EncryptionPanel(msg) => {
                let command = self.encryption_panel.update(msg);
                match command {
                    PanelCommand::ShowError(text) => iced::Task::done(Message::ShowErrorDialog(text)),
                    PanelCommand::None => iced::Task::none(),
                }
            }
            Message::DecryptionPanel(msg) => {
                let command = self.decryption_panel.update(msg);
                match command {
                    PanelCommand::ShowError(text) => iced::Task::done(Message::ShowErrorDialog(text)),
                    PanelCommand::None => iced::Task::none(),
                }
            }
            Message::ShowErrorDialog(text) => {
                self.error_dialog_open = true;
                self.error_dialog_text = text;
                iced::Task::none()
            }
            Message::HideErrorDialog => {
                self.error_dialog_open = false;
                iced::Task::none()
            }
            Message::EncryptionFinished(result) => {
                if result.is_ok() {
                    self.statusbar_message = "Шифрование завершено".to_string();
                } else {
                    self.statusbar_message = "Ошибка".to_string();
                    if let Err(e) = result {
                        self.error_dialog_text = format!("Ошибка шифрования: {}", e);
                        self.error_dialog_open = true;
                    }
                }
                iced::Task::none()
            }            Message::DecryptionFinished(result) => {
                if result.is_ok() {
                    self.statusbar_message = "Расшифровка завершена".to_string();
                } else {
                    self.statusbar_message = "Ошибка".to_string();
                    if let Err(e) = result {
                        self.error_dialog_text = format!("Ошибка расшифровки: {}", e);
                        self.error_dialog_open = true;
                    }
                }
                iced::Task::none()
            }        }
    }

    fn view(&self) -> Element<'_, Message> {
        let main_panel = row![
            self.encryption_panel
                .view()
                .map(|m| Message::EncryptionPanel(m)),
            self.decryption_panel
                .view()
                .map(|m| Message::DecryptionPanel(m)),
        ]
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Fill);

        let content = column![
            main_panel.height(Length::Fill),
            text(&self.statusbar_message)
                .width(Length::Fill)
                .align_x(Alignment::Center),
        ]
        .spacing(10)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

        dialog(
            self.error_dialog_open,
            content,
            text(&self.error_dialog_text),
        )
        .title("Ошибка")
        .push_button(iced_dialog::button("OK", Message::HideErrorDialog))
        .width(384)
        .height(256)
        .on_press(Message::HideErrorDialog)
        .into()
    }
}
