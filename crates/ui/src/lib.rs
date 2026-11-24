slint::slint! {
    #[edition = "1.19"]
    import { ListView } from "std-widgets.slint";

    component MetalButton inherits Rectangle {
        min-width: 0px;
        preferred-width: 216px;
        in property <string> label;
        in property <image> icon;
        in property <bool> has_icon: true;
        in property <bool> enabled: true;
        in property <length> font_size: 16px;
        in property <length> radius: 20px;
        in property <bool> active_tint: false;
        callback clicked();

        border-radius: root.radius;
        border-width: 2px;
        border-color: !root.enabled ? #94A3B8 : (touch.has-hover ? #24B7FF : (root.active_tint ? #299BDF : #7188A5));
        background: !root.enabled ? #CBD5E1 :
            (touch.pressed ? @linear-gradient(180deg, rgb(158, 178, 202) 0%, #C8D5E4 100%) :
            (root.active_tint ? @linear-gradient(180deg, #F0FAFF 0%, #CBEAFF 50%, #A6D9FF 100%) :
            @linear-gradient(180deg, #FFFFFF 0%, #F8FBFF 8%, #DCE6F2 24%, #AEBED2 48%, #F5F8FC 68%, #C7D3E2 88%, #91A4BD 100%)));


        HorizontalLayout {
            padding-left: 14px;
            padding-right: 14px;
            spacing: 8px;
            alignment: center;

            if root.has_icon : VerticalLayout {
                alignment: center;
                Image {
                    source: root.icon;
                    width: 22px;
                    height: 22px;
                }
            }
            VerticalLayout {
                alignment: center;
                Text {
                    text: root.label;
                    color: !root.enabled ? #64748B : (touch.has-hover ? #073B67 : #14243A);
                    font-size: root.font_size;
                    font-weight: 800;
                }
            }
        }

        touch := TouchArea {
            width: 100%;
            height: 100%;
            enabled: root.enabled;
            mouse-cursor: root.enabled ? pointer : default;
            clicked => { root.clicked(); }
        }
    }
    component DarkCtrlButton inherits Rectangle {
        in property <string> label;
        in property <length> font_size: 16px;
        callback clicked();

        border-radius: 14px;
        border-width: 2px;
        border-color: touch.has-hover ? #54D4FF : #D4E4F5;
        background: touch.pressed ? #071426 :
            @linear-gradient(180deg, #DDEBFA 0%, #7188A5 18%, #203650 50%, #071426 100%);

        Text {
            text: root.label;
            color: #FFFFFF;
            font-size: root.font_size;
            font-weight: 900;
            horizontal-alignment: center;
            vertical-alignment: center;
        }
        touch := TouchArea {
            width: 100%;
            height: 100%;
            mouse-cursor: pointer;
            clicked => { root.clicked(); }
        }
    }

    export component MainWindow inherits Window {
    default-font-family: "DejaVu Sans";
        no-frame: true;
        title: "DLNA SERVER";
        preferred-width: 900px;
        preferred-height: 820px;
        min-width: 780px;
        min-height: 680px;

        in-out property <string> server_name: "DLNA SERVER";
        in-out property <string> status: "Сервер остановлен. Добавьте папки и нажмите «Запустить».";
        in-out property <string> endpoint_info: "Сеть: ожидание запуска";
        in-out property <string> library_stats: "Видео: 0";
        in-out property <string> lang_label: "RU";
        in-out property <bool> lang_menu_open: false;
        in-out property <string> txt_on_air: "●  В ЭФИРЕ";
        in-out property <string> txt_stopped: "○  ОСТАНОВЛЕН";
        in-out property <string> txt_folders_title: "ПАПКИ МЕДИАТЕКИ";
        in-out property <string> txt_empty_hint: "Нажмите «Добавить папку», чтобы выбрать каталог с видеофайлами";
        in-out property <string> txt_add_folder: "Добавить папку";
        in-out property <string> txt_remove_folder: "Удалить";
        in-out property <string> txt_rescan: "Обновить";
        in-out property <string> txt_sleep_on: "Не спать: ВКЛ";
        in-out property <string> txt_sleep_off: "Не спать: ВЫКЛ";
        in-out property <string> txt_auto_on: "Автозапуск: ВКЛ";
        in-out property <string> txt_auto_off: "Автозапуск: ВЫКЛ";
        in-out property <string> txt_start_btn: "ЗАПУСТИТЬ СЕРВЕР";
        in-out property <string> txt_stop_btn: "ОСТАНОВИТЬ СЕРВЕР";
    in-out property <string> help_t1: "1. Запуск и папки";
    in-out property <string> help_d1: "Нажмите «Добавить папку», затем «ЗАПУСТИТЬ СЕРВЕР». Новые файлы — «Обновить».";
    in-out property <string> help_t2: "2. Подключение Smart TV";
    in-out property <string> help_d2: "ПК и ТВ в одной сети Wi-Fi/LAN. На ТВ откройте «Источники / Медиаплеер» → DLNA Server.";
    in-out property <string> help_t3: "3. Перекодирование кодека (без сжатия размера)";
    in-out property <string> help_d3: "Предел транскодера — 1080p (файлы 4K не перекодируются для защиты ПК). АВТО — только H.265 и редкие форматы.";
    in-out property <string> help_t4: "4. Работа в фоне и сон";
    in-out property <string> help_d4: "«Не спать» блокирует спящий режим ПК. Кнопка «✕» сворачивает сервер в трей без обрыва видео.";
    in-out property <string> help_t5: "О программе: DLNA Video Server v2.6.0";
    in-out property <string> help_d5: "Лицензия: GNU GPL-3.0-or-later • Открытый исходный код (Rust + Slint).";
        in-out property <bool> is_running: false;
        in-out property <bool> autostart_enabled: false;
        in-out property <bool> anti_sleep_enabled: true;
        in-out property <bool> transcode_enabled: false;
        in-out property <string> transcode_label: "Перекод: ВЫКЛ";
        in-out property <bool> transcode_menu_open: false;
        in-out property <bool> help_open: false;
        in-out property <int> selected_index: -1;
        in-out property <[string]> folders;

        callback browse_folder();
        callback remove_folder(int);
        callback start_server();
        callback stop_server();
        callback rescan_library();
        callback toggle_autostart(bool);
        callback toggle_anti_sleep(bool);
        callback select_transcode_mode(string);
        callback select_lang(string);
        callback minimize_window();
        callback close_to_tray();
        callback start_drag();
        callback show_help();

        background: transparent;

        Rectangle {
            border-radius: 24px;
            clip: true;
            border-width: 3px;
            border-color: #5B7394;
            background: @linear-gradient(180deg, #F7FAFF 0%, #E8EDF5 18%, #D7DFEA 55%, #EEF2F8 82%, #B7C3D3 100%);

            VerticalLayout {
                spacing: 0px;

                // Глубокая тёмно-синяя шапка с неоновой подсветкой как в 1.5
                Rectangle {
                    height: 158px;
                    background: @linear-gradient(180deg, #07101F 0%, #24364F 48%, #101C30 82%, #06101D 100%);

                    // Мягкое неоновое свечение и яркая линия снизу шапки
                    Rectangle {
                        y: parent.height - 10px;
                        width: 100%;
                        height: 6px;
                        background: @linear-gradient(180deg, transparent 0%, #19AFFF55 100%);
                    }
                    Rectangle {
                        y: parent.height - 4px;
                        width: 100%;
                        height: 4px;
                        background: #19AFFF;
                    }

                    hdr_drag := TouchArea {
                        property <bool> drag_started: false;
                        pointer-event(ev) => {
                            if (ev.kind == PointerEventKind.down && ev.button == PointerEventButton.left) {
                                self.drag_started = false;
                            } else if (ev.kind == PointerEventKind.move && self.pressed && !self.drag_started) {
                                self.drag_started = true;
                                root.start_drag();
                            } else if (ev.kind == PointerEventKind.up || ev.kind == PointerEventKind.cancel) {
                                self.drag_started = false;
                            }
                        }
                    }

                    HorizontalLayout {
                        padding: 14px;
                        spacing: 14px;

                        VerticalLayout {
                            spacing: 8px;
                            DarkCtrlButton {
                                label: root.lang_label;
                                font_size: 15px;
                                width: 58px;
                                height: 42px;
                                clicked => { root.lang_menu_open = !root.lang_menu_open; }
                            }
                            DarkCtrlButton {
                                label: "?";
                                font_size: 18px;
                                width: 58px;
                                height: 42px;
                                clicked => { root.show_help(); }
                            }
                            Rectangle { vertical-stretch: 1; }
                        }

                        HorizontalLayout {
                            horizontal-stretch: 1;
                            alignment: center;
                            spacing: 16px;

                            VerticalLayout {
                                alignment: center;
                                Image {
                                    source: @image-url("../assets/header_logo.png");
                                    width: 118px;
                                    height: 118px;
                                    image-fit: contain;
                                }
                            }

                            VerticalLayout {
                                alignment: center;
                                spacing: 4px;
                                Text {
                                    text: "DLNA SERVER";
                                    color: #F8FBFF;
                                    font-size: 34px;
                                    font-weight: 900;
                                    horizontal-alignment: left;
                                }
                                Text {
                                    text: "MEDIA SHARING";
                                    color: #DCEBFF;
                                    font-size: 14px;
                                    font-weight: 800;
                                    letter-spacing: 3px;
                                    horizontal-alignment: left;
                                }
                            }
                        }

                        VerticalLayout {
                            spacing: 8px;
                            HorizontalLayout {
                                alignment: end;
                                spacing: 8px;
                                DarkCtrlButton {
                                    label: "—";
                                    font_size: 16px;
                                    width: 42px;
                                    height: 36px;
                                    clicked => { root.minimize_window(); }
                                }
                                DarkCtrlButton {
                                    label: "✕";
                                    font_size: 16px;
                                    width: 42px;
                                    height: 36px;
                                    clicked => { root.close_to_tray(); }
                                }
                            }
                            Rectangle { vertical-stretch: 1; }
                            Rectangle {
                                width: 156px;
                                height: 38px;
                                border-radius: 19px;
                                border-width: 2px;
                                border-color: root.is_running ? #E1FFE0 : #FFD0D0;
                                background: root.is_running ?
                                    @linear-gradient(180deg, #C8FFB0 0%, #42D95B 16%, #0D9A3D 55%, #075525 100%) :
                                    @linear-gradient(180deg, #FF9C9C 0%, #D93636 16%, #A80E19 55%, #650711 100%);

                                Text {
                                    text: root.is_running ? root.txt_on_air : root.txt_stopped;
                                    color: root.is_running ? #E8FFE9 : #FFE8E8;
                                    font-size: 13px;
                                    font-weight: 900;
                                    horizontal-alignment: center;
                                    vertical-alignment: center;
                                }
                            }
                        }
                    }
                }
                VerticalLayout {
                    padding-left: 26px;
                    padding-right: 26px;
                    padding-top: 18px;
                    padding-bottom: 18px;
                    spacing: 12px;
                    vertical-stretch: 1;

                    HorizontalLayout {
                        Text {
                            text: root.txt_folders_title;
                            color: #536985;
                            font-size: 12px;
                            font-weight: 900;
                            letter-spacing: 2px;
                        }
                        Rectangle { horizontal-stretch: 1; }
                        Text {
                            text: root.library_stats;
                            color: #1B2C45;
                            font-size: 13px;
                            font-weight: 800;
                        }
                    }

                    Rectangle {
                        border-radius: 20px;
                        border-width: 2px;
                        border-color: #8295AF;
                        background: @linear-gradient(180deg, #A4B3C7 0%, #F8FAFE 8%, #E4EAF3 55%, #AAB8CB 100%);
                        vertical-stretch: 1;

                        if root.folders.length == 0 : Text {
                            text: root.txt_empty_hint;
                            color: #536985;
                            font-size: 15px;
                            font-weight: 700;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }

                        if root.folders.length > 0 : ListView {
                            for folder[index] in root.folders: Rectangle {
                                height: 64px;
                                Rectangle {
                                    height: 56px;
                                    border-radius: 15px;
                                    border-width: root.selected_index == index ? 2px : 1px;
                                    border-color: root.selected_index == index ? #299BDF : #A7B7CC;
                                    background: root.selected_index == index ?
                                        @linear-gradient(180deg, #F0FAFF 0%, #CBEAFF 50%, #A6D9FF 100%) :
                                        @linear-gradient(180deg, #FFFFFF 0%, #F8FAFF 12%, #E8EEF7 50%, #D9E2EF 86%, #C3CFDF 100%);

                                    HorizontalLayout {
                                        padding-left: 14px;
                                        padding-right: 14px;
                                        spacing: 12px;

                                        VerticalLayout {
                                            alignment: center;
                                            Image {
                                                source: @image-url("../assets/icons/folder_video.png");
                                                width: 34px;
                                                height: 34px;
                                            }
                                        }
                                        Text {
                                            text: folder;
                                            color: #17263B;
                                            font-size: 16px;
                                            font-weight: 700;
                                            overflow: elide;
                                            vertical-alignment: center;
                                            horizontal-stretch: 1;
                                        }
                                    }
                                    TouchArea {
                                        width: 100%;
                                        height: 100%;
                                        mouse-cursor: pointer;
                                        clicked => { root.selected_index = index; }
                                    }
                                }
                            }
                        }
                    }

                    HorizontalLayout {
                        spacing: 14px;
                        height: 58px;

                        MetalButton {
                            label: root.txt_add_folder;
                            icon: @image-url("../assets/icons/plus.png");
                            horizontal-stretch: 1;
                            clicked => { root.browse_folder(); }
                        }
                        MetalButton {
                            label: root.txt_remove_folder;
                            icon: @image-url("../assets/icons/cross.png");
                            enabled: root.selected_index >= 0 && root.selected_index < root.folders.length;
                            horizontal-stretch: 1;
                            clicked => {
                                root.remove_folder(root.selected_index);
                                root.selected_index = -1;
                            }
                        }
                        MetalButton {
                            label: root.txt_rescan;
                            icon: @image-url("../assets/icons/refresh.png");
                            enabled: root.folders.length > 0;
                            horizontal-stretch: 1;
                            clicked => { root.rescan_library(); }
                        }
                    }
                    HorizontalLayout {
                        spacing: 12px;
                        height: 50px;

                        MetalButton {
                            label: root.transcode_label;
                            has_icon: false;
                            font_size: 13px;
                            radius: 17px;
                            active_tint: root.transcode_enabled;
                            horizontal-stretch: 1;
                            clicked => { root.transcode_menu_open = !root.transcode_menu_open; }
                        }

                        MetalButton {
                            label: root.anti_sleep_enabled ? root.txt_sleep_on : root.txt_sleep_off;
                            icon: @image-url("../assets/icons/shield.png");
                            font_size: 13px;
                            radius: 17px;
                            active_tint: root.anti_sleep_enabled;
                            horizontal-stretch: 1;
                            clicked => { root.toggle_anti_sleep(!root.anti_sleep_enabled); }
                        }

                        MetalButton {
                            label: root.autostart_enabled ? root.txt_auto_on : root.txt_auto_off;
                            has_icon: false;
                            font_size: 13px;
                            radius: 17px;
                            active_tint: root.autostart_enabled;
                            horizontal-stretch: 1;
                            clicked => { root.toggle_autostart(!root.autostart_enabled); }
                        }
                    }

                    Rectangle {
                        height: 76px;
                        border-radius: 34px;
                        border-width: 3px;
                        border-color: #F7FCFF;
                        background: root.is_running ?
                            @linear-gradient(180deg, #E9FAFF 0%, rgb(127, 223, 255) 8%, rgb(39, 158, 240) 18%, #0879D6 48%, #0757A8 76%, #0A3D78 94%, #DCEBFA 100%) :
                            @linear-gradient(180deg, #F1FFF7 0%, #8BFFC0 8%, #28E58A 18%, #0DBB68 48%, #07834B 78%, #045B37 94%, #E1FFF0 100%);

                        main_touch := TouchArea {
                            mouse-cursor: pointer;
                            clicked => {
                                if (root.is_running) {
                                    root.stop_server();
                                } else {
                                    root.start_server();
                                }
                            }
                        }

                        HorizontalLayout {
                            alignment: center;
                            spacing: 14px;

                            VerticalLayout {
                                alignment: center;
                                Image {
                                    source: root.is_running ? @image-url("../assets/icons/stop.png") : @image-url("../assets/icons/play.png");
                                    width: 32px;
                                    height: 32px;
                                }
                            }
                            VerticalLayout {
                                alignment: center;
                                Text {
                                    text: root.is_running ? root.txt_stop_btn : root.txt_start_btn;
                                    color: #FFFFFF;
                                    font-size: 23px;
                                    font-weight: 900;
                                }
                            }
                        }
                    }
                }

                Rectangle {
                    height: 52px;
                    border-width: 2px;
                    border-color: #7188A5;
                    background: @linear-gradient(180deg, #FFFFFF 0%, #F2F7FD 12%, #D9E4F1 42%, #C0CEDF 72%, #91A5BF 100%);

                    HorizontalLayout {
                        padding-left: 20px;
                        padding-right: 20px;
                        spacing: 12px;

                        Text {
                            text: root.is_running ? "((●))" : "((○))";
                            color: root.is_running ? #087FC5 : #64748B;
                            font-size: 18px;
                            font-weight: 900;
                            vertical-alignment: center;
                        }
                        Text {
                            text: root.status;
                            color: #1C304B;
                            font-size: 13px;
                            font-weight: 800;
                            min-width: 0px;
                            overflow: elide;
                            vertical-alignment: center;
                            horizontal-stretch: 1.618;
                        }
                        Text {
                            text: root.endpoint_info;
                            color: #087FC5;
                            font-size: 13px;
                            font-weight: 900;
                            vertical-alignment: center;
                        }
                    }
                }
            }
            // Выпадающий список выбора языка
            if root.lang_menu_open : TouchArea {
                width: 100%;
                height: 100%;
                clicked => { root.lang_menu_open = false; }
            }
            if root.lang_menu_open : Rectangle {
                x: 16px;
                y: 58px;
                width: 190px;
                height: 368px;
                border-radius: 14px;
                border-width: 2px;
                border-color: #299BDF;
                background: @linear-gradient(180deg, #FFFFFF 0%, #E8EEF7 50%, #C7D3E2 100%);

                VerticalLayout {
                    padding: 6px;
                    spacing: 2px;
                    for item in [
                        { code: "ru", label: "RU — Русский" },
                        { code: "en", label: "EN — English" },
                        { code: "de", label: "DE — Deutsch" },
                        { code: "fr", label: "FR — Français" },
                        { code: "es", label: "ES — Español" },
                        { code: "it", label: "IT — Italiano" },
                        { code: "pt", label: "PT — Português" },
                        { code: "pl", label: "PL — Polski" },
                        { code: "uk", label: "UK — Українська" },
                        { code: "zh", label: "ZH — 中文" },
                        { code: "ja", label: "JA — 日本語" },
                        { code: "ko", label: "KO — 한국어" },
                    ]: Rectangle {
                        height: 28px;
                        border-radius: 8px;
                        background: item_touch.has-hover ? #CBEAFF : transparent;
                        item_touch := TouchArea {
                            mouse-cursor: pointer;
                            clicked => {
                                root.lang_menu_open = false;
                                root.select_lang(item.code);
                            }
                        }
                        Text {
                            x: 10px;
                            text: item.label;
                            color: #14243A;
                            font-size: 13px;
                            font-weight: 700;
                            vertical-alignment: center;
                        }
                    }
                }
            }
            if root.transcode_menu_open : TouchArea {
                width: 100%;
                height: 100%;
                clicked => { root.transcode_menu_open = false; }
            }
            if root.transcode_menu_open : Rectangle {
                x: 26px;
                y: root.height - 285px;
                width: 260px;
                height: 110px;
                border-radius: 14px;
                border-width: 2px;
                border-color: #299BDF;
                background: @linear-gradient(180deg, #FFFFFF 0%, #E8EEF7 50%, #C7D3E2 100%);

                VerticalLayout {
                    padding: 6px;
                    spacing: 2px;
                    for item in [
                        { mode: "off", label: "ВЫКЛ (Оригинал 1:1)" },
                        { mode: "auto", label: "АВТО (Только H.265 / редкие)" },
                        { mode: "on", label: "ВКЛ (Все видео → H.264)" },
                    ]: Rectangle {
                        height: 30px;
                        border-radius: 8px;
                        background: tc_touch.has-hover ? #CBEAFF : transparent;
                        tc_touch := TouchArea {
                            mouse-cursor: pointer;
                            clicked => {
                                root.transcode_menu_open = false;
                                root.select_transcode_mode(item.mode);
                            }
                        }
                        Text {
                            x: 10px;
                            text: item.label;
                            color: #14243A;
                            font-size: 13px;
                            font-weight: 700;
                            vertical-alignment: center;
                        }
                    }
                }
            }
            // Выпадающий список справки (в стиле меню RU)
            if root.help_open : TouchArea {
                width: 100%;
                height: 100%;
                clicked => { root.help_open = false; }
            }
            if root.help_open : Rectangle {
                x: 16px;
                y: 108px;
                width: 470px;
                height: 335px;
                border-radius: 14px;
                border-width: 2px;
                border-color: #299BDF;
                background: @linear-gradient(180deg, #FFFFFF 0%, #E8EEF7 50%, #C7D3E2 100%);

                VerticalLayout {
                    padding: 8px;
                    spacing: 4px;
                    for item in [
                        { title: root.help_t1, desc: root.help_d1 },
                        { title: root.help_t2, desc: root.help_d2 },
                        { title: root.help_t3, desc: root.help_d3 },
                        { title: root.help_t4, desc: root.help_d4 },
                        { title: root.help_t5, desc: root.help_d5 },
                    ]: Rectangle {
                        height: 58px;
                        border-radius: 8px;
                        background: h_touch.has-hover ? #CBEAFF : transparent;
                        h_touch := TouchArea {
                            mouse-cursor: pointer;
                            clicked => { root.help_open = false; }
                        }
                        VerticalLayout {
                            padding-left: 10px;
                            padding-right: 10px;
                            padding-top: 6px;
                            padding-bottom: 6px;
                            spacing: 2px;
                            Text {
                                text: item.title;
                                color: #087FC5;
                                font-size: 13px;
                                font-weight: 800;
                            }
                            Text {
                                text: item.desc;
                                color: #14243A;
                                font-size: 12px;
                                font-weight: 600;
                                wrap: word-wrap;
                            }
                        }
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod ui_click_tests {
    use super::*;
    use slint::platform::PointerEventButton;
    use slint::{ComponentHandle, LogicalPosition, ModelRc, SharedString, VecModel};
    use std::cell::Cell;
    use std::rc::Rc;

    fn click_at(ui: &MainWindow, x: f32, y: f32) {
        let pos = LogicalPosition::new(x, y);
        ui.window().dispatch_event(slint::platform::WindowEvent::PointerMoved { position: pos });
        ui.window().dispatch_event(slint::platform::WindowEvent::PointerPressed {
            position: pos,
            button: PointerEventButton::Left,
        });
        ui.window().dispatch_event(slint::platform::WindowEvent::PointerReleased {
            position: pos,
            button: PointerEventButton::Left,
        });
    }

    #[test]
    fn test_all_ui_buttons_hit_testing() {
        i_slint_backend_testing::init_no_event_loop();
        let ui = MainWindow::new().unwrap();
        ui.set_folders(ModelRc::new(VecModel::from(vec![SharedString::from("/mnt/video")])));
        ui.set_selected_index(0);
        ui.window().set_size(slint::PhysicalSize::new(900, 820));

        let hits = Rc::new(Cell::new(0u32));

        macro_rules! bind_hit {
            ($setter:ident) => {{
                let h = hits.clone();
                ui.$setter(move || h.set(h.get() + 1));
            }};
        }

        let _ = ui.get_lang_menu_open();
        bind_hit!(on_show_help);
        bind_hit!(on_minimize_window);
        bind_hit!(on_close_to_tray);
        bind_hit!(on_browse_folder);
        bind_hit!(on_rescan_library);
        bind_hit!(on_start_server);

        let h_rem = hits.clone();
        ui.on_remove_folder(move |_| h_rem.set(h_rem.get() + 1));
        let h_sleep = hits.clone();
        ui.on_toggle_anti_sleep(move |_| h_sleep.set(h_sleep.get() + 1));
        let h_auto = hits.clone();
        ui.on_toggle_autostart(move |_| h_auto.set(h_auto.get() + 1));

        // Кликаем по реальным координатам каждой кнопки в окне 900x820:
        click_at(&ui, 43.0, 35.0);   // Кнопка RU
        assert!(ui.get_lang_menu_open(), "Не сработала кнопка RU"); ui.set_lang_menu_open(false);

        click_at(&ui, 43.0, 83.0);   // Кнопка ?
        assert_eq!(hits.get(), 1, "Не сработала кнопка ?");

        click_at(&ui, 823.0, 32.0);  // Кнопка Свернуть (—)
        assert_eq!(hits.get(), 2, "Не сработала кнопка Свернуть");

        click_at(&ui, 867.0, 32.0);  // Кнопка Закрыть (✕)
        assert_eq!(hits.get(), 3, "Не сработала кнопка Закрыть");

        click_at(&ui, 160.0, 585.0); // Кнопка Добавить папку
        assert_eq!(hits.get(), 4, "Не сработала кнопка Добавить папку");

        click_at(&ui, 450.0, 585.0); // Кнопка Удалить
        assert_eq!(hits.get(), 5, "Не сработала кнопка Удалить");

        click_at(&ui, 740.0, 585.0); // Кнопка Обновить
        assert_eq!(hits.get(), 6, "Не сработала кнопка Обновить");

        click_at(&ui, 160.0, 645.0); // Кнопка Перекод
        assert!(ui.get_transcode_menu_open(), "Не сработала кнопка Перекод"); ui.set_transcode_menu_open(false);

        click_at(&ui, 450.0, 645.0); // Кнопка Не спать
        assert_eq!(hits.get(), 7, "Не сработала кнопка Не спать");

        click_at(&ui, 740.0, 645.0); // Кнопка Автозапуск
        assert_eq!(hits.get(), 8, "Не сработала кнопка Автозапуск");

        click_at(&ui, 450.0, 725.0); // Главная кнопка ЗАПУСТИТЬ СЕРВЕР
        assert_eq!(hits.get(), 9, "Не сработала главная кнопка ЗАПУСТИТЬ СЕРВЕР");
    }
}
