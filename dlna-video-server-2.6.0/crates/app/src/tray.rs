use dlna_ui::MainWindow;
use ksni::{menu::StandardItem, Category, Icon, MenuItem, Status, Tray, TrayMethods};
use slint::ComponentHandle;

const TRAY_ICON_PNG: &[u8] = include_bytes!("../../ui/assets/brand_48.png");

pub struct AppTray {
    ui: slint::Weak<MainWindow>,
}

impl Tray for AppTray {
    fn id(&self) -> String {
        "dlna-video-server-2".into()
    }

    fn category(&self) -> Category {
        Category::ApplicationStatus
    }

    fn status(&self) -> Status {
        Status::Active
    }

    fn title(&self) -> String {
        "DLNA VIDEO SERVER 2.0".into()
    }

    fn icon_name(&self) -> String {
        "dlna-video-server-2".into()
    }

    fn icon_theme_path(&self) -> String {
        "/usr/share/dlna-server/assets".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let img = image::load_from_memory(TRAY_ICON_PNG).expect("valid embedded PNG");
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mut argb = Vec::with_capacity((width * height * 4) as usize);
        for px in rgba.pixels() {
            let [r, g, b, a] = px.0;
            argb.extend_from_slice(&[a, r, g, b]);
        }
        vec![Icon {
            width: width as i32,
            height: height as i32,
            data: argb,
        }]
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let ui_open = self.ui.clone();
        vec![
            MenuItem::Standard(StandardItem {
                label: "Открыть окно".into(),
                activate: Box::new(move |_| {
                    let ui = ui_open.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(w) = ui.upgrade() {
                            let _ = w.show();
                        }
                    });
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Выход".into(),
                activate: Box::new(move |_| {
                    let _ = slint::invoke_from_event_loop(|| {
                        let _ = slint::quit_event_loop();
                    });
                }),
                ..Default::default()
            }),
        ]
    }
}

pub fn spawn_tray(ui: slint::Weak<MainWindow>) {
    std::thread::spawn(move || {
        let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() else {
            return;
        };
        rt.block_on(async move {
            let tray = AppTray { ui };
            match tray.spawn().await {
                Ok(_handle) => {
                    std::future::pending::<()>().await;
                }
                Err(err) => eprintln!("[TRAY-ERR] {err}"),
            }
        });
    });
}
