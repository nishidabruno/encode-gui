#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs::File,
    io::{Error, Read, Write},
};

use eframe::egui;
use egui::{vec2, Color32, FontId};
use egui_notify::Toasts;
use std::path;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 180.0])
            .with_resizable(false)
            .with_maximize_button(false),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "File encoder",
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
    file_path: Option<String>,
    output_directory: Option<String>,
    overwrite_file: bool,
    toasts: Toasts,
    encoder: Encoder,
}

#[derive(PartialEq, Debug, Default)]
enum Encoder {
    #[default]
    UTF8,
    ISO88591,
    EUCKR,
    SHIFTJIS,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File encoder");
            ui.label("A Simple file encode.");
            ui.separator();
            let file_path = egui::Label::new(
                egui::RichText::new(
                    self.file_path
                        .clone()
                        .unwrap_or("path/to/file.txt".to_string()),
                )
                .monospace(),
            )
            .truncate();
            let folder_path = egui::Label::new(
                egui::RichText::new(
                    self.output_directory
                        .clone()
                        .unwrap_or("path/to/folder".to_string()),
                )
                .monospace(),
            )
            .truncate();
            ui.horizontal(|ui| {
                ui.label("File to convert:");
                ui.add_sized(vec2(254.0, 16.0), file_path);

                if ui.button("Open file…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.file_path = Some(path.display().to_string());
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
                if self.overwrite_file {
                    ui.add_sized(vec2(226.0, 16.0), file_overwritten_label);
                } else {
                    ui.add_sized(vec2(226.0, 16.0), folder_path);
                }

                if !self.overwrite_file && ui.button("Open folder...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.output_directory = Some(path.display().to_string());
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Encode to:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.encoder))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.encoder, Encoder::ISO88591, "ISO88591");
                        ui.selectable_value(&mut self.encoder, Encoder::EUCKR, "EUCKR");
                        ui.selectable_value(&mut self.encoder, Encoder::SHIFTJIS, "SHIFTJIS");
                        ui.selectable_value(&mut self.encoder, Encoder::UTF8, "UTF8");
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
                    if let Some(path) = &self.file_path {
                        if self.overwrite_file
                            && encode_file(path, path, true, &self.encoder).is_ok()
                        {
                            self.toasts
                                .success("File encoded successfully")
                                .font(FontId::proportional(12.5));
                        }
                        if let Some(output_dir) = &self.output_directory {
                            if !self.overwrite_file
                                && encode_file(path, output_dir, false, &self.encoder).is_ok()
                            {
                                self.toasts
                                    .success("File encoded successfully")
                                    .font(FontId::proportional(12.5));
                            }
                        } else {
                            // self.toasts.info("Open the folder to be saved in").font(FontId::proportional(12.5));
                        }
                    } else {
                        self.toasts
                            .info("Open the file to be encoded")
                            .font(FontId::proportional(12.5));
                    }
                }
            });
        });

        self.toasts.show(ctx);
    }
}

fn encode_file(
    file_path: &String,
    output_dir: &String,
    overwrite: bool,
    encode_to: &Encoder,
) -> Result<(), Error> {
    println!("Encode function called");
    let input_path = path::Path::new(file_path);

    let output_dir = path::Path::new(output_dir);
    let output_file = if overwrite {
        output_dir.to_path_buf()
    } else {
        output_dir.join("encoded_file.txt")
    };

    let mut input_file = File::open(input_path)?;
    let mut content = Vec::new();
    input_file.read_to_end(&mut content)?;

    let (content, _, _) = encoding_rs::UTF_8.decode(&content);

    let (encoded_content, encoder, has_unmappable_content) = match encode_to {
        Encoder::ISO88591 => encoding_rs::ISO_8859_10.encode(&content),
        Encoder::EUCKR => encoding_rs::EUC_KR.encode(&content),
        Encoder::SHIFTJIS => encoding_rs::SHIFT_JIS.encode(&content),
        _ => encoding_rs::UTF_8.encode(&content),
    };

    let mut output_file = File::create(&output_file)?;
    output_file.write_all(&encoded_content)?;

    Ok(())
}
