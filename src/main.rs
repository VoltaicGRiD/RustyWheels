use std::path::PathBuf;
use eframe::egui;
use egui::Color32;
use std::fs;
use serde::{Deserialize, Serialize};
use powershell_script;


fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Harrier Build & Compile",
        native_options,
        Box::new(|cc| Box::new(HarrierBnC::new(cc))),
    );
}

fn os_str_to_string(os_str: &std::ffi::OsStr) -> Option<String> {
    os_str.to_str().map(|s| s.to_owned())
}


// Script: Full output of the file, all lines
// Script holds: Lines
// Lines hold: success, error


#[derive(Deserialize, Serialize)]
#[serde(default)]
struct HarrierBnC {
    script_path: String,
    scripts: Vec<Script>,
    scripts_to_remove: Vec<usize>,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct Script {
    title: String,
    lines: Vec<Line>,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct Line {
    line: String,
    success: bool,
    error: String,
    output: String
}


impl HarrierBnC {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        };

        Default::default()
    }
}

impl Default for HarrierBnC {
    fn default() -> Self {
        Self {
            script_path: String::new(),
            scripts: Vec::new(),
            scripts_to_remove: Vec::new(),
        }
    }
}

impl Default for Script {
    fn default() -> Self {
        Self {
            title: String::new(),
            lines: Vec::new(),
        }
    }
}

impl Default for Line {
    fn default() -> Self {
        Self {
            line: String::new(),
            success: false,
            error: String::new(),
            output: String::new(),
        }
    }
}

impl eframe::App for HarrierBnC {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_| {});

        egui::SidePanel::left("Control Panel").show(&ctx, |ui| {
            ui.heading("Control Panel");
            ui.set_max_width(400f32);

            ui.text_edit_singleline(&mut self.script_path);

            let path = PathBuf::from(&self.script_path); // Create a PathBuf object from the
                                                         // script_path file that the user input

            if let Ok(metadata) = fs::metadata(&path) {
                if metadata.is_dir() {
                    match fs::read_dir(&path) {
                        Ok(read_dir) => {
                            let files: Vec<PathBuf> = read_dir.filter_map(|x| {
                                let path = x.ok()?.path();
                                if path.extension().map_or(false, |ext| ext == "ps1") {
                                    Some(path)
                                } else {
                                    None
                                }
                            })
                            .collect();

                            for (_index, file) in files.iter().enumerate() {
                                let name = os_str_to_string(file.file_name().expect("File name invalid")).expect("Unable to convert OsStr to String");
                                if ui.button(name).clicked() {
                                    let mut new_script = Script::default();
                                    let file_path = PathBuf::from(file);
                                    let file_contents = fs::read_to_string(&file_path).expect("Unable to read file");
                                    for (_index,line) in file_contents.split('\n').into_iter().enumerate() {
                                        let mut new_line = Line::default();
                                        new_line.line = line.to_string();
                                        new_script.lines.push(new_line);
                                    }
                                    new_script.title = os_str_to_string(file.file_name().expect("File name invalid")).expect("Unable to convert OsStr to String").to_uppercase();
                                    self.scripts.push(new_script);
                                }
                            };
                        }
                        Err(e) => {
                            println!("Unable to read directory: {:?}", e);
                        }
                    }

                } else {
                    println!("Not a directory: {:?}", &self.script_path);
                }
            } else {
                println!("Invalid path: {:?}", &self.script_path);
            }
        });

        // Iterate throught the scripts_to_remove buffer and remove all of the scripts from the
        // parent scripts vector
        for (index) in self.scripts_to_remove.iter().rev() {
            self.scripts.remove(*index);
        }

        self.scripts_to_remove.clear(); // Clear the scripts_to_remove buffer

        for (index, script) in self.scripts.iter_mut().enumerate() {
            egui::Window::new(&script.title).show(ctx, |ui| {
                if ui.button("Run all").clicked() {
                    for (index, line) in script.lines.iter_mut().enumerate() {
                        match powershell_script::run(&line.line) {
                            Ok(output) => {
                                line.success = true;
                                line.output = output.stdout().unwrap_or_else(|| String::from("No output captured"));
                            }
                            Err(e) => {
                                line.success = false;
                                line.error = e.to_string();
                            }
                        }

                        std::thread::sleep(std::time::Duration::from_millis(50_u64));
                    }
                }

                for (index, line) in script.lines.iter_mut().enumerate() {
                    if ui.checkbox(&mut line.success, &line.line).changed() {
                        match powershell_script::run(&line.line) {
                            Ok(output) => {
                                line.success = true;
                                line.output = output.stdout().unwrap_or_else(|| String::from("No output captured"));
                            }
                            Err(e) => {
                                line.success = false;
                                line.error = e.to_string();
                            }
                        }
                    };

                    if line.success == false && !line.error.trim().is_empty() {
                        ui.colored_label(Color32::from_rgb(255, 128, 128), format!("Error: {}", line.error.to_string()));
                    }

                    if line.success == true && !line.output.trim().is_empty() {
                        ui.colored_label(Color32::from_rgb(144, 238, 144), format!("Output: {}", line.error.to_string()));
                    }
                }

                if ui.button("Clear and close").clicked() {
                    self.scripts_to_remove.push(index);
                }
            });
        }
    }
}
