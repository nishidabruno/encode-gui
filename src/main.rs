#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    borrow::Cow,
    fmt,
    fs::File,
    io::{Error, Read, Write},
    path::{self},
};

use eframe::egui;
use egui::{vec2, Color32, FontId};
use egui_notify::Toasts;

#[derive(PartialEq, Debug, Default)]
enum Encoder {
    #[default]
    Utf8,
    Iso88591,
    Euckr,
    Shiftjis,
}

impl fmt::Display for Encoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Encoder::Utf8 => write!(f, "UTF-8"),
            Encoder::Iso88591 => write!(f, "ISO8859-1"),
            Encoder::Euckr => write!(f, "EUC-KR"),
            Encoder::Shiftjis => write!(f, "SHIFT-JIS"),
        }
    }
}

const WINDOW_WIDTH: f32 = 440.0;
const WINDOW_HEIGHT: f32 = 180.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_resizable(false)
            .with_maximize_button(false),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Encode Base",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp {
                toasts: Toasts::default(),
                ..Default::default()
            }))
        }),
    )
}

#[derive(Default)]
struct MyApp {
    file_path: Option<Cow<'static, str>>,
    output_directory: Option<Cow<'static, str>>,
    overwrite_file: bool,
    toasts: Toasts,
    encoder: Encoder,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Encode Base");
            ui.label("A Simple file encoder");
            ui.separator();
            let file_path = egui::Label::new(
                egui::RichText::new(
                    self.file_path
                        .clone()
                        .unwrap_or(Cow::Borrowed("path/to/file.txt")),
                )
                .monospace(),
            )
            .truncate();
            let folder_path = egui::Label::new(
                egui::RichText::new(
                    self.output_directory
                        .clone()
                        .unwrap_or(Cow::Borrowed("path/to/folder")),
                )
                .monospace(),
            )
            .truncate();
            ui.horizontal(|ui| {
                ui.label("File to convert:");
                ui.add_sized(vec2(254.0, 16.0), file_path);

                if ui.button("Open file…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.file_path = Some(Cow::Owned(path.display().to_string()));
                    }
                }
            });

            let file_overwritten_label = egui::Label::new(
                egui::RichText::new("File will be overwritten!")
                    .monospace()
                    .color(Color32::YELLOW),
            );
            ui.horizontal(|ui| {
                ui.label("Output directory:");
                match self.overwrite_file {
                    true => ui.add_sized(vec2(226.0, 16.0), file_overwritten_label),
                    false => ui.add_sized(vec2(226.0, 16.0), folder_path),
                };

                if !self.overwrite_file && ui.button("Open folder...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.output_directory = Some(Cow::Owned(path.display().to_string()));
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Encode to:");
                egui::ComboBox::from_label("")
                    .selected_text(self.encoder.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.encoder, Encoder::Iso88591, "ISO8859-1");
                        ui.selectable_value(&mut self.encoder, Encoder::Euckr, "EUC-KR");
                        ui.selectable_value(&mut self.encoder, Encoder::Shiftjis, "SHIFT-JIS");
                        ui.selectable_value(&mut self.encoder, Encoder::Utf8, "UTF-8");
                    });
            });

            ui.checkbox(&mut self.overwrite_file, "Overwrite file");
            ui.separator();

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Clear").clicked() {
                    self.file_path = None;
                    self.output_directory = None;
                }

                if ui.button("Convert").clicked() {
                    match (&self.file_path, &self.output_directory) {
                        (Some(path), _) if self.overwrite_file => {
                            if self
                                .handle_encoding(path, path, true, &self.encoder)
                                .is_ok()
                            {
                                self.toasts
                                    .success("File encoded successfully")
                                    .font(FontId::proportional(12.5));
                            }
                        }
                        (Some(path), Some(output_dir)) if !self.overwrite_file => {
                            if self
                                .handle_encoding(path, output_dir, false, &self.encoder)
                                .is_ok()
                            {
                                self.toasts
                                    .success("File encoded successfully")
                                    .font(FontId::proportional(12.5));
                            }
                        }
                        (None, _) => {
                            self.toasts
                                .info("Open the file to be encoded")
                                .font(FontId::proportional(12.5));
                        }
                        (_, None) if !self.overwrite_file => {
                            self.toasts
                                .info("Open the folder to be saved in")
                                .font(FontId::proportional(12.5));
                        }
                        _ => {}
                    }
                }
            });
        });

        self.toasts.show(ctx);
    }
}

impl MyApp {
    fn handle_encoding(
        &self,
        file_path: &str,
        output_dir: &str,
        overwrite: bool,
        encode_to: &Encoder,
    ) -> Result<(), Error> {
        let input_path = path::Path::new(file_path);

        let output_dir = path::Path::new(output_dir);
        let output_file = match overwrite {
            true => output_dir.to_path_buf(),
            false => output_dir.join("encoded_file.txt"),
        };

        let mut input_file = File::open(input_path)?;
        let mut content = Vec::new();
        input_file.read_to_end(&mut content)?;

        let (content, _, _) = encoding_rs::UTF_8.decode(&content);

        let (encoded_content, _encoder, _has_unmappable_content) = match encode_to {
            Encoder::Iso88591 => encoding_rs::ISO_8859_10.encode(&content),
            Encoder::Euckr => encoding_rs::EUC_KR.encode(&content),
            Encoder::Shiftjis => encoding_rs::SHIFT_JIS.encode(&content),
            _ => encoding_rs::UTF_8.encode(&content),
        };

        let mut output_file = File::create(output_file)?;
        output_file.write_all(&encoded_content)?;

        Ok(())
    }
}
