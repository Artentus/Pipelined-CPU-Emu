use egui::text::LayoutJob;

pub fn highlight(text: &str) -> LayoutJob {
    use egui::text::{LayoutSection, TextFormat};

    struct Performer {
        job: LayoutJob,
    }

    impl vte::Perform for Performer {
        fn print(&mut self, c: char) {
            self.job.text.push(c);
            self.job.sections.last_mut().unwrap().byte_range.end = self.job.text.len();
        }

        fn execute(&mut self, byte: u8) {
            match byte {
                b'\n' => {
                    self.job.text.push('\n');
                    self.job.sections.last_mut().unwrap().byte_range.end = self.job.text.len();
                }
                _ => {}
            }
        }

        fn csi_dispatch(
            &mut self,
            params: &vte::Params,
            _intermediates: &[u8],
            ignore: bool,
            action: char,
        ) {
            if !ignore {
                match action {
                    'm' => {
                        let last_section = self.job.sections.last_mut().unwrap();
                        let format = if last_section.byte_range.end <= last_section.byte_range.start
                        {
                            &mut last_section.format
                        } else {
                            let format = last_section.format.clone();
                            self.job.sections.push(LayoutSection {
                                leading_space: 0.0,
                                byte_range: self.job.text.len()..self.job.text.len(),
                                format,
                            });

                            &mut self.job.sections.last_mut().unwrap().format
                        };

                        let mut params = params.iter();
                        while let Some(param) = params.next() {
                            match param[0] {
                                0 => {
                                    *format = TextFormat {
                                        color: egui::Color32::from_rgb(229, 229, 229),
                                        ..Default::default()
                                    }
                                }

                                1 => { /* TODO: */ }
                                3 => format.italics = true,
                                4 => format.underline = egui::Stroke::new(1.0, format.color),
                                9 => format.strikethrough = egui::Stroke::new(1.0, format.color),

                                21 => { /* TODO: */ }
                                22 => { /* TODO: */ }
                                23 => format.italics = false,
                                24 => format.underline = egui::Stroke::NONE,
                                29 => format.strikethrough = egui::Stroke::NONE,

                                30 => format.color = egui::Color32::from_rgb(0, 0, 0),
                                31 => format.color = egui::Color32::from_rgb(205, 49, 49),
                                32 => format.color = egui::Color32::from_rgb(13, 188, 121),
                                33 => format.color = egui::Color32::from_rgb(229, 229, 16),
                                34 => format.color = egui::Color32::from_rgb(36, 114, 200),
                                35 => format.color = egui::Color32::from_rgb(188, 63, 188),
                                36 => format.color = egui::Color32::from_rgb(17, 168, 205),
                                37 => format.color = egui::Color32::from_rgb(229, 229, 229),
                                39 => format.color = egui::Color32::from_rgb(229, 229, 229),

                                40 => format.background = egui::Color32::from_rgb(0, 0, 0),
                                41 => format.background = egui::Color32::from_rgb(205, 49, 49),
                                42 => format.background = egui::Color32::from_rgb(13, 188, 121),
                                43 => format.background = egui::Color32::from_rgb(229, 229, 16),
                                44 => format.background = egui::Color32::from_rgb(36, 114, 200),
                                45 => format.background = egui::Color32::from_rgb(188, 63, 188),
                                46 => format.background = egui::Color32::from_rgb(17, 168, 205),
                                47 => format.background = egui::Color32::from_rgb(229, 229, 229),
                                49 => format.background = egui::Color32::from_rgb(229, 229, 229),

                                90 => format.color = egui::Color32::from_rgb(102, 102, 102),
                                91 => format.color = egui::Color32::from_rgb(241, 76, 76),
                                92 => format.color = egui::Color32::from_rgb(35, 209, 139),
                                93 => format.color = egui::Color32::from_rgb(245, 245, 67),
                                94 => format.color = egui::Color32::from_rgb(59, 142, 234),
                                95 => format.color = egui::Color32::from_rgb(214, 112, 214),
                                96 => format.color = egui::Color32::from_rgb(41, 184, 219),
                                97 => format.color = egui::Color32::from_rgb(229, 229, 229),

                                100 => format.background = egui::Color32::from_rgb(102, 102, 102),
                                101 => format.background = egui::Color32::from_rgb(241, 76, 76),
                                102 => format.background = egui::Color32::from_rgb(35, 209, 139),
                                103 => format.background = egui::Color32::from_rgb(245, 245, 67),
                                104 => format.background = egui::Color32::from_rgb(59, 142, 234),
                                105 => format.background = egui::Color32::from_rgb(214, 112, 214),
                                106 => format.background = egui::Color32::from_rgb(41, 184, 219),
                                107 => format.background = egui::Color32::from_rgb(229, 229, 229),

                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut performer = Performer {
        job: LayoutJob {
            text: String::new(),
            sections: vec![LayoutSection {
                leading_space: 0.0,
                byte_range: 0..0,
                format: TextFormat {
                    color: egui::Color32::from_rgb(229, 229, 229),
                    ..Default::default()
                },
            }],
            ..Default::default()
        },
    };

    let mut parser = vte::Parser::new();
    for byte in text.bytes() {
        parser.advance(&mut performer, byte)
    }

    performer.job
}
