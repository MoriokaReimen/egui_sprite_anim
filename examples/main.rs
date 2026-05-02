use eframe::egui;
use egui::Pos2;
use egui::Rect;
use sprite_anim::GifPlayer;
use sprite_anim::Sprite;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "sprite_anim Example",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    gif_player: GifPlayer,
    counter: usize,
    sprite: Sprite,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut gif_player = GifPlayer::new();
        let gif_data1 = include_bytes!("../asset/example1.gif");
        let gif_data2 = include_bytes!("../asset/example2.gif");
        let gif_data3 = include_bytes!("../asset/example3.gif");
        let gif_data4 = include_bytes!("../asset/example4.gif");
        gif_player
            .add(&cc.egui_ctx, "example1", gif_data1)
            .expect("Failed to add gif");
        gif_player
            .add(&cc.egui_ctx, "example2", gif_data2)
            .expect("Failed to add gif");
        gif_player
            .add(&cc.egui_ctx, "example3", gif_data3)
            .expect("Failed to add gif");
        gif_player
            .add(&cc.egui_ctx, "example4", gif_data4)
            .expect("Failed to add gif");
        gif_player.size(egui::vec2(300.0, 300.0));

        let sprite_data = include_bytes!("../asset/sprite.png");
        let mut sprite = Sprite::new(&cc.egui_ctx, sprite_data).expect("Failed to add Sprite");
        sprite
            .add_map(
                "Button1",
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(50.0, 50.0)),
            )
            .expect("Failed to add map");
        sprite
            .add_map(
                "Button2",
                Rect::from_min_max(Pos2::new(50.0, 0.0), Pos2::new(100.0, 50.0)),
            )
            .expect("Failed to add map");
        Self {
            gif_player,
            counter: 0usize,
            sprite,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sprite Anim Example");

            ui.add(&mut self.gif_player);

            let image1 = self.sprite.get_as_button("Button2").unwrap();
            if ui.add(image1).clicked() {
                self.counter = (self.counter + 1) % 4;
                let _ = match self.counter {
                    0 => self.gif_player.select_image("example1"),
                    1 => self.gif_player.select_image("example2"),
                    2 => self.gif_player.select_image("example3"),
                    3 => self.gif_player.select_image("example4"),
                    _ => self.gif_player.select_image("example4"),
                };
                println!("Button clicked!");
            }
            let image2 = self.sprite.get_as_button("Button1").unwrap();

            if ui.add(image2).clicked() {
                println!("Sprite Button Clicked!");
                if self.gif_player.is_playing() {
                    self.gif_player.stop();
                } else {
                    self.gif_player.play();
                }
            }
        });
    }
}
