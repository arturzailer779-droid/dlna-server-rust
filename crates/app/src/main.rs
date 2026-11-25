mod i18n;
mod tray;

use anyhow::{Context, Result};
use dlna_config::AppConfig;
use dlna_media_library::MediaIndex;
use dlna_ui::*;
use slint::VecModel;
use std::{
    cell::RefCell,
    net::TcpListener,
    path::PathBuf,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

struct ActiveServer {
    running: Arc<AtomicBool>,
    index: Arc<RwLock<MediaIndex>>,
    ssdp_handle: Option<thread::JoinHandle<()>>,
    http_handle: Option<thread::JoinHandle<()>>,
    _sleep_guard: Option<dlna_platform::SleepInhibitor>,
}

impl ActiveServer {
    fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self._sleep_guard = None;
        if let Some(h) = self.ssdp_handle.take() {
            let _ = h.join();
        }
        if let Some(h) = self.http_handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for ActiveServer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn main() -> Result<()> {
    let _log_path = dlna_diagnostics::init_logging()?;
    let start_minimized = std::env::args().any(|a| a == "--minimized" || a == "--autostart");

    let config_path = dlna_config::default_config_path()?;
    let cache_path = dlna_config::default_index_cache_path()?;
    let mut config = AppConfig::load(&config_path)
        .with_context(|| format!("Не удалось загрузить настройки: {}", config_path.display()))?;

    config.autostart = dlna_platform::is_autostart_enabled();

    let system_icon = std::env::var_os("DLNA_SERVER_ASSET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/share/dlna-server/assets"))
        .join("logo_tv.png");
    if !config.icon.exists() && system_icon.exists() {
        config.icon = system_icon;
    }

    let ui = MainWindow::new()?;
    tray::spawn_tray(ui.as_weak());

    // При нажатии на крестик окна сворачиваем в трей (как в старой версии)
    {
        let ui_weak = ui.as_weak();
        ui.window().on_close_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let _ = ui.hide();
            }
            slint::CloseRequestResponse::HideWindow
        });
    }

    let config = Rc::new(RefCell::new(config));
    let server: Rc<RefCell<Option<ActiveServer>>> = Rc::new(RefCell::new(None));
    refresh_ui(&ui, &config);
    i18n::apply_language(&ui, &config.borrow().language);

    if dlna_platform::is_ufw_blocking() {
        ui.set_status("Внимание: активен UFW и порты 8208/TCP или 1900/UDP не разрешены.".into());
    }
    {
        let config = Rc::clone(&config);
        let ui_weak = ui.as_weak();
        let config_path = config_path.clone();
        ui.on_toggle_autostart(move |enabled| {
            let Some(ui) = ui_weak.upgrade() else { return };
            match dlna_platform::set_autostart(enabled) {
                Ok(()) => {
                    ui.set_autostart_enabled(enabled);
                    let mut settings = config.borrow_mut();
                    settings.autostart = enabled;
                    let _ = settings.save(&config_path);
                    ui.set_status(if enabled { "Автозапуск при входе в систему включён." } else { "Автозапуск отключён." }.into());
                }
                Err(err) => {
                    ui.set_autostart_enabled(!enabled);
                    ui.set_status(format!("Ошибка настройки автозапуска: {err}").into());
                }
            }
        });
    }

    {
        let config = Rc::clone(&config);
        let server = Rc::clone(&server);
        let ui_weak = ui.as_weak();
        let config_path = config_path.clone();
        ui.on_toggle_anti_sleep(move |enabled| {
            let Some(ui) = ui_weak.upgrade() else { return };
            let real_enabled = enabled;
            if enabled {
                let mut probe = dlna_platform::SleepInhibitor::acquire();
                if !probe.is_active() {
                    ui.set_anti_sleep_enabled(false);
                    let mut settings = config.borrow_mut();
                    settings.anti_sleep = false;
                    let _ = settings.save(&config_path);
                    ui.set_status("Не удалось запустить systemd-inhibit: защита от сна недоступна.".into());
                    return;
                }
                if let Some(active) = server.borrow_mut().as_mut() {
                    active._sleep_guard = Some(probe);
                }
            } else if let Some(active) = server.borrow_mut().as_mut() {
                active._sleep_guard = None;
            }
            ui.set_anti_sleep_enabled(real_enabled);
            let mut settings = config.borrow_mut();
            settings.anti_sleep = real_enabled;
            let _ = settings.save(&config_path);
            ui.set_status(if real_enabled {
                "Защита от сна (systemd-inhibit) активна."
            } else {
                "Защита от сна отключена."
            }.into());
        });
    }

    {
        let config = Rc::clone(&config);
        let ui_weak = ui.as_weak();
        let config_path = config_path.clone();
        let dialog_open = Arc::new(AtomicBool::new(false));
        ui.on_browse_folder(move || {
            if dialog_open.swap(true, Ordering::SeqCst) {
                return;
            }
            let flag = Arc::clone(&dialog_open);
            let ui_w = ui_weak.clone();
            let cfg = Rc::clone(&config);
            let cfg_p = config_path.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            thread::spawn(move || {
                let picked = rfd::FileDialog::new()
                    .set_title("Выберите папку с видеофайлами")
                    .pick_folder();
                flag.store(false, Ordering::SeqCst);
                let _ = tx.send(picked);
            });
            slint::Timer::single_shot(std::time::Duration::from_millis(50), move || {
                fn poll_dialog(
                    rx: std::sync::mpsc::Receiver<Option<std::path::PathBuf>>,
                    ui_w: slint::Weak<MainWindow>,
                    cfg: Rc<RefCell<AppConfig>>,
                    cfg_p: std::path::PathBuf,
                ) {
                    match rx.try_recv() {
                        Ok(Some(folder)) => {
                            if let Some(ui) = ui_w.upgrade() {
                                add_folder_path(&ui, &cfg, &cfg_p, folder);
                            }
                        }
                        Ok(None) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            slint::Timer::single_shot(std::time::Duration::from_millis(80), move || {
                                poll_dialog(rx, ui_w, cfg, cfg_p);
                            });
                        }
                    }
                }
                poll_dialog(rx, ui_w, cfg, cfg_p);
            });
        });
    }

    {
        let config = Rc::clone(&config);
        let ui_weak = ui.as_weak();
        let config_path = config_path.clone();
        {
        let config = Rc::clone(&config);
        let ui_weak = ui.as_weak();
        let config_path = config_path.clone();
        ui.on_select_lang(move |lang_code| {
            let Some(ui) = ui_weak.upgrade() else { return };
            let code_str = lang_code.to_string();
            {
                let mut settings = config.borrow_mut();
                settings.language = code_str.clone();
                let _ = settings.save(&config_path);
            }
            i18n::apply_language(&ui, &code_str);
        });
    } if false { let _ = move |entered_path: String| {
            let Some(ui) = ui_weak.upgrade() else { return };
            let trimmed = entered_path.trim();
            if trimmed.is_empty() {
                ui.set_status("Укажите путь к папке или нажмите «Обзор...».".into());
                return;
            }
            add_folder_path(&ui, &config, &config_path, PathBuf::from(trimmed));
        };
    } }

    {
        let config = Rc::clone(&config);
        let server = Rc::clone(&server);
        let ui_weak = ui.as_weak();
        let config_path = config_path.clone();
        let cache_path = cache_path.clone();
        ui.on_remove_folder(move |index| {
            let Some(ui) = ui_weak.upgrade() else { return };
            if index < 0 { return; }
            let mut settings = config.borrow_mut();
            let idx = index as usize;
            if idx >= settings.roots.len() { return; }
            let removed = settings.roots.remove(idx);
            let saved = settings.save(&config_path);
            let roots_after = settings.roots.clone();
            drop(settings);
            match saved {
                Ok(()) => ui.set_status(format!("Папка удалена: {}", removed.display()).into()),
                Err(error) => {
                    config.borrow_mut().roots.insert(idx, removed);
                    ui.set_status(format!("Ошибка сохранения: {error}").into());
                    return;
                }
            }
            refresh_ui(&ui, &config);
            i18n::apply_language(&ui, &config.borrow().language);
            if let Some((index_arc, running_arc)) = server.borrow().as_ref().map(|a| (Arc::clone(&a.index), Arc::clone(&a.running))) {
                let (ui_w, c_path) = (ui.as_weak(), cache_path.clone());
                thread::spawn(move || {
                    let prev = index_arc.read().ok().map(|i| i.update_id).unwrap_or(0);
                    let refreshed = MediaIndex::scan(&roots_after, prev);
                    let count = refreshed.video_count();
                    let _ = refreshed.save_cache(&roots_after, &c_path);
                    if running_arc.load(Ordering::Relaxed)
                        && let Ok(mut guard) = index_arc.write()
                    {
                        *guard = refreshed;
                    }
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_w.upgrade() {
                            ui.set_library_stats(format!("Видеофайлов в индексе: {count}").into());
                        }
                    });
                });
            }
        });
    }
    {
        let config = Rc::clone(&config);
        let server = Rc::clone(&server);
        let ui_weak = ui.as_weak();
        let cache_path = cache_path.clone();

        ui.on_start_server(move || {
            let Some(ui) = ui_weak.upgrade() else { return };
            if server.borrow().is_some() { return; }
            let cfg_snapshot = config.borrow().clone();
            if cfg_snapshot.roots.is_empty() {
                ui.set_status("Добавьте хотя бы одну папку с видео перед запуском.".into());
                return;
            }
            let ip = match dlna_network::default_route_ip() {
                Ok(ip) => ip,
                Err(err) => { ui.set_status(format!("Ошибка сети: {err}").into()); return; }
            };
            let mut chosen_port = cfg_snapshot.port;
            let mut bound = None;
            for candidate in cfg_snapshot.port..=(cfg_snapshot.port + 40) {
                if let Ok(l) = TcpListener::bind((ip, candidate)) {
                    chosen_port = candidate;
                    bound = Some(l);
                    break;
                }
            }
            let Some(listener) = bound else {
                ui.set_status("Не удалось найти свободный порт (8208..8248).".into());
                return;
            };
            let mut cfg_snapshot = cfg_snapshot;
            cfg_snapshot.port = chosen_port;
            let _ = listener.set_nonblocking(true);

            let index = Arc::new(RwLock::new(MediaIndex::default()));
            let running = Arc::new(AtomicBool::new(true));
            let ready = Arc::new(AtomicBool::new(false));
            let cfg_arc = Arc::new(cfg_snapshot);

            let ssdp_handle = match dlna_protocol::ssdp::spawn(Arc::clone(&cfg_arc), Arc::clone(&running), Arc::clone(&ready), ip) {
                Ok(h) => h,
                Err(err) => { running.store(false, Ordering::SeqCst); ui.set_status(format!("Ошибка SSDP: {err}").into()); return; }
            };

            let (http_cfg, http_idx, http_run) = (Arc::clone(&cfg_arc), Arc::clone(&index), Arc::clone(&running));
            let http_handle = thread::spawn(move || dlna_streaming::http::run(listener, http_cfg, http_idx, http_run));

            ui.set_is_running(true);
            ui.set_endpoint_info(format!("Адрес: http://{}:{}/  |  UUID: {}", ip, cfg_arc.port, cfg_arc.uuid).into());
            ui.set_library_stats("Индексация папок...".into());
            ui.set_status("DLNA-сервер запущен. Идёт быстрая загрузка медиатеки...".into());

            let (scan_cfg, scan_idx, scan_run, scan_ready, scan_cache, ui_scan_weak) =
                (Arc::clone(&cfg_arc), Arc::clone(&index), Arc::clone(&running), Arc::clone(&ready), cache_path.clone(), ui.as_weak());
            thread::spawn(move || {
                if let Some(cached) = MediaIndex::load_cache(&scan_cfg.roots, &scan_cache) {
                    let c_count = cached.video_count();
                    if scan_run.load(Ordering::Relaxed) {
                        if let Ok(mut guard) = scan_idx.write() { *guard = cached; }
                        scan_ready.store(true, Ordering::SeqCst);
                        let ui_w = ui_scan_weak.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_w.upgrade() {
                                ui.set_library_stats(format!("Видеофайлов в индексе: {c_count}").into());
                            }
                        });
                    }
                }
                let prev = scan_idx.read().map(|i| i.update_id).unwrap_or(0);
                let refreshed = MediaIndex::scan(&scan_cfg.roots, prev);
                let count = refreshed.video_count();
                if scan_run.load(Ordering::Relaxed) {
                    let _ = refreshed.save_cache(&scan_cfg.roots, &scan_cache);
                    if let Ok(mut guard) = scan_idx.write() { *guard = refreshed; }
                }
                scan_ready.store(true, Ordering::SeqCst);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_scan_weak.upgrade() {
                        ui.set_library_stats(format!("Видеофайлов в индексе: {count}").into());
                        ui.set_status("Сервер в эфире. Индексация завершена.".into());
                    }
                });
            });

            let sleep_guard = if cfg_arc.anti_sleep { Some(dlna_platform::SleepInhibitor::acquire()) } else { None };
            *server.borrow_mut() = Some(ActiveServer { running, index, ssdp_handle: Some(ssdp_handle), http_handle: Some(http_handle), _sleep_guard: sleep_guard });
        });
    }

    {
        let server = Rc::clone(&server);
        let ui_weak = ui.as_weak();
        ui.on_stop_server(move || {
            let Some(ui) = ui_weak.upgrade() else { return };
            if let Some(mut active) = server.borrow_mut().take() {
                thread::spawn(move || active.stop());
            }
            ui.set_is_running(false);
            ui.set_endpoint_info("Сеть: сервер остановлен".into());
            ui.set_status("Сервер остановлен.".into());
        });
    }

    {
        let config = Rc::clone(&config);
        let server = Rc::clone(&server);
        let ui_weak = ui.as_weak();
        let cache_path = cache_path.clone();
        ui.on_rescan_library(move || {
            let Some(ui) = ui_weak.upgrade() else { return };
            let active_handles = server.borrow().as_ref().map(|a| (Arc::clone(&a.index), Arc::clone(&a.running)));
            let roots = config.borrow().roots.clone();
            let (ui_rescan_weak, cache_path) = (ui.as_weak(), cache_path.clone());
            ui.set_status("Пересканирование видеотеки...".into());
            thread::spawn(move || {
                let prev = active_handles.as_ref().and_then(|(idx, _)| idx.read().ok().map(|i| i.update_id)).unwrap_or(0);
                let refreshed = MediaIndex::scan(&roots, prev);
                let count = refreshed.video_count();
                let _ = refreshed.save_cache(&roots, &cache_path);
                if let Some((index_arc, running_arc)) = active_handles
                    && running_arc.load(Ordering::Relaxed)
                        && let Ok(mut guard) = index_arc.write() { *guard = refreshed; }
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_rescan_weak.upgrade() {
                        ui.set_library_stats(format!("Видеофайлов в индексе: {count}").into());
                        ui.set_status(format!("Библиотека обновлена. Найдено видео: {count}.").into());
                    }
                });
            });
        });
    }

    {
        let config = Rc::clone(&config);
        let ui_weak = ui.as_weak();
        let config_path = config_path.clone();
        ui.on_select_transcode_mode(move |mode| {
            let Some(ui) = ui_weak.upgrade() else { return };
            let mode_str = mode.to_string();
            let enabled = mode_str != "off";
            if enabled {
                use std::sync::OnceLock;
                static HAS_FFMPEG: OnceLock<bool> = OnceLock::new();
                let ok = *HAS_FFMPEG.get_or_init(|| dlna_transcoding::probe_engine().available);
                if !ok {
                    ui.set_transcode_enabled(false);
                    ui.set_transcode_label("Перекод: ВЫКЛ".into());
                    ui.set_status("FFmpeg не найден в системе. Установите пакет ffmpeg.".into());
                    return;
                }
            }
            ui.set_transcode_enabled(enabled);
            let lbl = match mode_str.as_str() {
                "auto" => "Перекод: АВТО",
                "on" | "1080p" | "720p" => "Перекод: ВКЛ",
                _ => "Перекод: ВЫКЛ",
            };
            ui.set_transcode_label(lbl.into());
            let mut settings = config.borrow_mut();
            settings.transcoding_enabled = enabled;
            settings.transcode_mode = mode_str;
            let _ = settings.save(&config_path);
            ui.set_status(format!("Режим: {}. Перезапустите эфир для обновления каталога ТВ.", lbl).into());
        });
    }

    {
        use slint::winit_030::WinitWindowAccessor;
        let ui_weak = ui.as_weak();
        ui.on_minimize_window(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.window().with_winit_window(|w| { w.set_minimized(true); });
            }
        });
        let ui_weak = ui.as_weak();
        ui.on_close_to_tray(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let _ = ui.hide();
            }
        });
        let ui_weak = ui.as_weak();
        ui.on_start_drag(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.window().with_winit_window(|w| { let _ = w.drag_window(); });
            }
        });

        let ui_weak = ui.as_weak();
        ui.on_show_help(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_help_open(true);
            }
        });
    }

    if !start_minimized {
        ui.show()?;
    }

    if (start_minimized || dlna_platform::is_autostart_enabled()) && !config.borrow().roots.is_empty() {
        let ui_auto = ui.as_weak();
        let cfg_auto = Rc::clone(&config);
        fn schedule_autostart(ui_w: slint::Weak<MainWindow>, cfg: Rc<RefCell<AppConfig>>, attempt: u8) {
            if attempt >= 10 {
                return;
            }
            slint::Timer::single_shot(Duration::from_millis(if attempt == 0 { 400 } else { 1500 }), move || {
                let Some(ui) = ui_w.upgrade() else { return };
                if ui.get_is_running() || cfg.borrow().roots.is_empty() {
                    return;
                }
                if dlna_network::default_route_ip().is_ok() {
                    ui.invoke_start_server();
                } else {
                    ui.set_status("Ожидание подключения к локальной сети для автозапуска...".into());
                    schedule_autostart(ui_w, cfg, attempt + 1);
                }
            });
        }
        schedule_autostart(ui_auto, cfg_auto, 0);
    }

    slint::run_event_loop_until_quit()?;

    if let Some(mut active) = server.borrow_mut().take() {
        active.stop();
        thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

fn add_folder_path(ui: &MainWindow, config: &Rc<RefCell<AppConfig>>, config_path: &PathBuf, raw_path: PathBuf) {
    let path = match std::fs::canonicalize(&raw_path) {
        Ok(p) if p.is_dir() => p,
        Ok(_) => { ui.set_status("Указанный путь не является папкой.".into()); return; }
        Err(error) => { ui.set_status(format!("Не удалось открыть папку: {error}").into()); return; }
    };
    let mut settings = config.borrow_mut();
    if settings.roots.iter().any(|item| item == &path) {
        ui.set_status("Эта папка уже добавлена.".into());
        return;
    }
    settings.roots.push(path);
    match settings.save(config_path) {
        Ok(()) => {  ui.set_status("Папка добавлена и сохранена.".into()); }
        Err(error) => { settings.roots.pop(); ui.set_status(format!("Ошибка сохранения: {error}").into()); }
    }
    drop(settings);
    refresh_ui(ui, config);
}

fn refresh_ui(ui: &MainWindow, config: &Rc<RefCell<AppConfig>>) {
    let settings = config.borrow();
    ui.set_server_name(settings.name.clone().into());
    ui.set_autostart_enabled(settings.autostart);
    ui.set_anti_sleep_enabled(settings.anti_sleep);
    ui.set_transcode_enabled(settings.transcoding_enabled);
    ui.set_transcode_label(match settings.transcode_mode.as_str() {
        "auto" => "Перекод: АВТО",
        "on" | "1080p" | "720p" => "Перекод: ВКЛ",
        _ => "Перекод: ВЫКЛ",
    }.into());
    let items: Vec<slint::SharedString> = settings.roots.iter().map(|p| p.to_string_lossy().to_string().into()).collect();
    ui.set_folders(Rc::new(VecModel::from(items)).into());
}
