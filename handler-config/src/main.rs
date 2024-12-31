use eframe::egui;
use rfd::FileDialog;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 350.0]),
        ..Default::default()
    };

    eframe::run_native(
        "mpv-handler config generator",
        options,
        Box::new(|cc| Ok(Box::new(ConfigApp::new(cc)))),
    )
}

#[derive(Default)]
struct ConfigApp {
    handler_path: String,
    mpv_path: String,
    status_message: String,
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MPV Handler Config Generator");

            ui.add_space(20.0);
            if ui.button("Choose handler path").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("exe file", &["exe"])
                    .pick_file()
                {
                    self.handler_path = path.display().to_string();
                    self.status_message.clear();
                }
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Current handler path:");
                ui.text_edit_singleline(&mut self.handler_path);
            });

            ui.add_space(10.0);

            if ui.button("Choose mpv path").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("exe file", &["exe"])
                    .pick_file()
                {
                    self.mpv_path = path.display().to_string();
                    self.status_message.clear();
                }
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Current mpv path:");
                ui.text_edit_singleline(&mut self.mpv_path);
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                let has_handler = !self.handler_path.is_empty();
                let has_mpv = !self.mpv_path.is_empty();

                if ui
                    .add_enabled(has_handler, egui::Button::new("Generate registry"))
                    .clicked()
                {
                    if let Err(err) = self.generate_reg_file(&self.handler_path) {
                        self.status_message = format!("Generate reg Failed: {}", err);
                    } else {
                        self.status_message = "Generate reg Success!".to_string();
                    }
                }

                if ui
                    .add_enabled(has_handler && has_mpv, egui::Button::new("Generate toml"))
                    .clicked()
                {
                    if let Err(err) = self.generate_toml(&self.handler_path, &self.mpv_path) {
                        self.status_message = format!("Generate toml Failed: {}", err);
                    } else {
                        self.status_message = "Generate toml Success!".to_string();
                    }
                }
            });

            if !self.status_message.is_empty() {
                ui.label(&self.status_message);
            }
        });
    }
}

impl ConfigApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set style
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(20.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
        cc.egui_ctx.set_style(style);

        Default::default()
    }

    fn generate_reg_file(&self, exe_path: &str) -> io::Result<()> {
        let exe_path_escaped = exe_path.replace("\\", "\\\\");

        let reg_content = format!(
            r#"Windows Registry Editor Version 5.00
[HKEY_CLASSES_ROOT\mpv]
"URL Protocol"=""
@="mpv"
[HKEY_CLASSES_ROOT\mpv\shell]
[HKEY_CLASSES_ROOT\mpv\shell\open]
[HKEY_CLASSES_ROOT\mpv\shell\open\command]
@="\"{}\" \"%1\""
"#,
            exe_path_escaped
        );

        let reg_file_path = Path::new("mpv-handler.reg");
        let mut file = File::create(reg_file_path)?;
        let utf16_content: Vec<u16> = reg_content.encode_utf16().collect();
        file.write_all(&[0xFF, 0xFE])?; // UTF-16 LE BOM
        for word in utf16_content {
            file.write_all(&word.to_le_bytes())?;
        }

        Ok(())
    }

    fn generate_toml(&self, handler_path: &str, mpv_path: &str) -> io::Result<()> {
        let toml_path = Path::new(handler_path)
            .parent()
            .unwrap()
            .join("mpv-handler.toml");
        let mut file = File::create(toml_path)?;
        let mpv = format!(r#"mpv = "{}""#, mpv_path.replace("\\", "\\\\"));
        file.write_all(mpv.as_bytes())?;

        Ok(())
    }
}
