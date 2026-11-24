use dlna_ui::MainWindow;

#[allow(dead_code)]
pub const LANGS: &[&str] = &["ru", "en", "de", "fr", "es", "it", "pt", "pl", "uk", "zh", "ja", "ko"];

#[allow(dead_code)]
pub fn next_lang(current: &str) -> &'static str {
    let idx = LANGS.iter().position(|&l| l.eq_ignore_ascii_case(current)).unwrap_or(0);
    LANGS[(idx + 1) % LANGS.len()]
}

pub struct Texts {
    pub code: &'static str,
    pub on_air: &'static str,
    pub stopped: &'static str,
    pub folders_title: &'static str,
    pub empty_hint: &'static str,
    pub add_folder: &'static str,
    pub remove_folder: &'static str,
    pub rescan: &'static str,
    pub sleep_on: &'static str,
    pub sleep_off: &'static str,
    pub auto_on: &'static str,
    pub auto_off: &'static str,
    pub start_btn: &'static str,
    pub stop_btn: &'static str,
    pub ready_status: &'static str,
}
pub fn texts_for(lang: &str) -> Texts {
    match lang.to_ascii_lowercase().as_str() {
        "en" => Texts {
            code: "EN", on_air: "●  ON AIR", stopped: "○  STOPPED",
            folders_title: "MEDIA FOLDERS", empty_hint: "Click «Add Folder» to choose a video directory",
            add_folder: "Add Folder", remove_folder: "Remove", rescan: "Refresh",
            sleep_on: "No Sleep: ON", sleep_off: "No Sleep: OFF",
            auto_on: "Autostart: ON", auto_off: "Autostart: OFF",
            start_btn: "START SERVER", stop_btn: "STOP SERVER",
            ready_status: "Server stopped. Add folders and click «Start Server».",
        },
        "de" => Texts {
            code: "DE", on_air: "●  SENDET", stopped: "○  GESTOPPT",
            folders_title: "MEDIENORDNER", empty_hint: "Klicken Sie auf «Ordner hinzufügen»",
            add_folder: "Ordner hinzufügen", remove_folder: "Entfernen", rescan: "Aktualisieren",
            sleep_on: "Wachhalten: EIN", sleep_off: "Wachhalten: AUS",
            auto_on: "Autostart: EIN", auto_off: "Autostart: AUS",
            start_btn: "SERVER STARTEN", stop_btn: "SERVER STOPPEN",
            ready_status: "Server gestoppt. Ordner hinzufügen und starten.",
        },
        "fr" => Texts {
            code: "FR", on_air: "●  EN DIRECT", stopped: "○  ARRÊTÉ",
            folders_title: "DOSSIERS VIDÉO", empty_hint: "Cliquez sur «Ajouter un dossier»",
            add_folder: "Ajouter un dossier", remove_folder: "Supprimer", rescan: "Actualiser",
            sleep_on: "Anti-veille : OUI", sleep_off: "Anti-veille : NON",
            auto_on: "Démarrage auto : OUI", auto_off: "Démarrage auto : NON",
            start_btn: "DÉMARRER LE SERVEUR", stop_btn: "ARRÊTER LE SERVEUR",
            ready_status: "Serveur arrêté. Ajoutez des dossiers et démarrez.",
        },
        "es" => Texts {
            code: "ES", on_air: "●  EN VIVO", stopped: "○  DETENIDO",
            folders_title: "CARPETAS DE VÍDEO", empty_hint: "Pulse «Añadir carpeta» para elegir vídeos",
            add_folder: "Añadir carpeta", remove_folder: "Eliminar", rescan: "Actualizar",
            sleep_on: "Sin reposo: SÍ", sleep_off: "Sin reposo: NO",
            auto_on: "Inicio auto: SÍ", auto_off: "Inicio auto: NO",
            start_btn: "INICIAR SERVIDOR", stop_btn: "DETENER SERVIDOR",
            ready_status: "Servidor detenido. Añada carpetas e inicie.",
        },
        "it" => Texts {
            code: "IT", on_air: "●  IN ONDA", stopped: "○  FERMO",
            folders_title: "CARTELLE VIDEO", empty_hint: "Premi «Aggiungi cartella» per scegliere i video",
            add_folder: "Aggiungi cartella", remove_folder: "Rimuovi", rescan: "Aggiorna",
            sleep_on: "No Sospensione: SÌ", sleep_off: "No Sospensione: NO",
            auto_on: "Avvio auto: SÌ", auto_off: "Avvio auto: NO",
            start_btn: "AVVIA SERVER", stop_btn: "ARRESTA SERVER",
            ready_status: "Server fermo. Aggiungi cartelle e avvia.",
        },
        "pt" => Texts {
            code: "PT", on_air: "●  NO AR", stopped: "○  PARADO",
            folders_title: "PASTAS DE VÍDEO", empty_hint: "Clique em «Adicionar pasta» para escolher vídeos",
            add_folder: "Adicionar pasta", remove_folder: "Remover", rescan: "Atualizar",
            sleep_on: "Sem suspensão: SIM", sleep_off: "Sem suspensão: NÃO",
            auto_on: "Início auto: SIM", auto_off: "Início auto: NÃO",
            start_btn: "INICIAR SERVIDOR", stop_btn: "PARAR SERVIDOR",
            ready_status: "Servidor parado. Adicione pastas e inicie.",
        },
        "pl" => Texts {
            code: "PL", on_air: "●  NA ŻYWO", stopped: "○  ZATRZYMANY",
            folders_title: "FOLDERY WIDEO", empty_hint: "Kliknij «Dodaj folder», aby wybrać katalog",
            add_folder: "Dodaj folder", remove_folder: "Usuń", rescan: "Odśwież",
            sleep_on: "Nie usypiaj: WŁ", sleep_off: "Nie usypiaj: WYŁ",
            auto_on: "Autostart: WŁ", auto_off: "Autostart: WYŁ",
            start_btn: "URUCHOM SERWER", stop_btn: "ZATRZYMAJ SERWER",
            ready_status: "Serwer zatrzymany. Dodaj foldery i uruchom.",
        },
        "uk" => Texts {
            code: "UK", on_air: "●  В ЕФІРІ", stopped: "○  ЗУПИНЕНО",
            folders_title: "ПАПКИ МЕДІАТЕКИ", empty_hint: "Натисніть «Додати папку», щоб вибрати відео",
            add_folder: "Додати папку", remove_folder: "Видалити", rescan: "Оновити",
            sleep_on: "Не спати: УВІМК", sleep_off: "Не спати: ВИМК",
            auto_on: "Автозапуск: УВІМК", auto_off: "Автозапуск: ВИМК",
            start_btn: "ЗАПУСТИТИ СЕРВЕР", stop_btn: "ЗУПИНИТИ СЕРВЕР",
            ready_status: "Сервер зупинено. Додайте папки та запустіть.",
        },
        "zh" => Texts {
            code: "ZH", on_air: "●  运行中", stopped: "○  已停止",
            folders_title: "视频文件夹", empty_hint: "点击«添加文件夹»选择视频目录",
            add_folder: "添加文件夹", remove_folder: "删除", rescan: "刷新",
            sleep_on: "防休眠: 开", sleep_off: "防休眠: 关",
            auto_on: "开机自启: 开", auto_off: "开机自启: 关",
            start_btn: "启动服务器", stop_btn: "停止服务器",
            ready_status: "服务器已停止。请添加文件夹并启动。",
        },
        "ja" => Texts {
            code: "JA", on_air: "●  配信中", stopped: "○  停止中",
            folders_title: "ビデオフォルダ", empty_hint: "«フォルダを追加» をクリックして選択",
            add_folder: "フォルダを追加", remove_folder: "削除", rescan: "更新",
            sleep_on: "スリープ防止: ON", sleep_off: "スリープ防止: OFF",
            auto_on: "自動起動: ON", auto_off: "自動起動: OFF",
            start_btn: "サーバー開始", stop_btn: "サーバー停止",
            ready_status: "停止中です。フォルダを追加して開始してください。",
        },
        "ko" => Texts {
            code: "KO", on_air: "●  방송 중", stopped: "○  중지됨",
            folders_title: "비디오 폴더", empty_hint: "«폴더 추가»를 눌러 동영상 폴더를 선택하세요",
            add_folder: "폴더 추가", remove_folder: "삭제", rescan: "새로고침",
            sleep_on: "절전 방지: 켜짐", sleep_off: "절전 방지: 꺼짐",
            auto_on: "자동 시작: 켜짐", auto_off: "자동 시작: 꺼짐",
            start_btn: "서버 시작", stop_btn: "서버 중지",
            ready_status: "서버가 중지되었습니다. 폴더를 추가하고 시작하세요.",
        },
        _ => Texts {
            code: "RU", on_air: "●  В ЭФИРЕ", stopped: "○  ОСТАНОВЛЕН",
            folders_title: "ПАПКИ МЕДИАТЕКИ", empty_hint: "Нажмите «Добавить папку», чтобы выбрать каталог с видеофайлами",
            add_folder: "Добавить папку", remove_folder: "Удалить", rescan: "Обновить",
            sleep_on: "Не спать: ВКЛ", sleep_off: "Не спать: ВЫКЛ",
            auto_on: "Автозапуск: ВКЛ", auto_off: "Автозапуск: ВЫКЛ",
            start_btn: "ЗАПУСТИТЬ СЕРВЕР", stop_btn: "ОСТАНОВИТЬ СЕРВЕР",
            ready_status: "Сервер остановлен. Добавьте папки и нажмите «Запустить».",
        },
    }
}


fn help_texts_for(lang: &str) -> [&'static str; 10] {
    match lang {
        "en" => [
            "1. Start & Folders",
            "Click 'Add Folder', then 'START SERVER'. Click 'Refresh' after adding new files.",
            "2. Smart TV Connection",
            "PC and TV must be on the same LAN/Wi-Fi. Open Sources / Media Player -> DLNA Server on your TV.",
            "3. Codec Transcoding (Native Resolution)",
            "Limit is 1080p (4K is skipped to protect PC). AUTO converts H.265/AV1 to H.264 without resizing.",
            "4. Background & Anti-Sleep",
            "'Keep Awake' prevents system sleep. Clicking '✕' minimizes the server to the system tray.",
            "About: DLNA Video Server v2.6.0",
            "License: GNU GPL-3.0-or-later • Open Source (Rust + Slint).",
        ],
        "de" => [
            "1. Start & Ordner",
            "Klicken Sie auf 'Ordner hinzufügen' und dann auf 'SERVER STARTEN'. 'Aktualisieren' scannt neue Dateien.",
            "2. Smart-TV-Verbindung",
            "PC und TV im selben Netzwerk. Öffnen Sie am TV 'Quellen / Mediaplayer' -> DLNA Server.",
            "3. Codec-Transkodierung (Originalgröße)",
            "Limit 1080p (4K wird zum Schutz des PCs übersprungen). AUTO wandelt H.265/AV1 in H.264 um.",
            "4. Hintergrund & Energiesparen",
            "'Wach halten' verhindert den Ruhemodus. '✕' minimiert den Server in den Infobereich.",
            "Über: DLNA Video Server v2.6.0",
            "Lizenz: GNU GPL-3.0-or-later • Open Source (Rust + Slint).",
        ],
        "fr" => [
            "1. Démarrage et dossiers",
            "Cliquez sur 'Ajouter un dossier', puis 'DÉMARRER LE SERVEUR'. 'Actualiser' détecte les nouveaux fichiers.",
            "2. Connexion Smart TV",
            "PC et TV sur le même réseau Wi-Fi/LAN. Ouvrez Sources / Lecteur multimédia -> DLNA Server sur la TV.",
            "3. Transcodage codec (Taille native)",
            "Limite 1080p (4K ignorée pour protéger le PC). AUTO convertit H.265/AV1 en H.264 sans redimensionner.",
            "4. Arrière-plan et veille",
            "'Empêcher la veille' bloque la mise en veille. '✕' réduit le serveur dans la zone de notification.",
            "À propos : DLNA Video Server v2.6.0",
            "Licence : GNU GPL-3.0-or-later • Open Source (Rust + Slint).",
        ],
        "es" => [
            "1. Inicio y carpetas",
            "Pulse 'Añadir carpeta' y luego 'INICIAR SERVIDOR'. Use 'Actualizar' para nuevos archivos.",
            "2. Conexión Smart TV",
            "PC y TV en la misma red Wi-Fi/LAN. Abra Fuentes / Reproductor multimedia -> DLNA Server en su TV.",
            "3. Transcodificación de códec (Tamaño nativo)",
            "Límite 1080p (4K omitido para proteger el PC). AUTO convierte H.265/AV1 a H.264 sin escalar.",
            "4. Segundo plano y suspensión",
            "'No suspender' evita el reposo del PC. '✕' minimiza el servidor a la bandeja del sistema.",
            "Acerca de: DLNA Video Server v2.6.0",
            "Licencia: GNU GPL-3.0-or-later • Código abierto (Rust + Slint).",
        ],
        "it" => [
            "1. Avvio e cartelle",
            "Fai clic su 'Aggiungi cartella', poi 'AVVIA SERVER'. Usa 'Aggiorna' per i nuovi file.",
            "2. Connessione Smart TV",
            "PC e TV sulla stessa rete Wi-Fi/LAN. Apri Sorgenti / Lettore multimediale -> DLNA Server sulla TV.",
            "3. Transcodifica codec (Risoluzione nativa)",
            "Limite 1080p (4K saltato per proteggere il PC). AUTO converte H.265/AV1 in H.264 senza ridimensionare.",
            "4. Background e sospensione",
            "'Resta attivo' impedisce la sospensione. '✕' riduce a icona il server nell'area di notifica.",
            "Informazioni: DLNA Video Server v2.6.0",
            "Licenza: GNU GPL-3.0-or-later • Open Source (Rust + Slint).",
        ],
        "pt" => [
            "1. Início e pastas",
            "Clique em 'Adicionar pasta' e depois em 'INICIAR SERVIDOR'. Use 'Atualizar' para novos arquivos.",
            "2. Conexão Smart TV",
            "PC e TV na mesma rede Wi-Fi/LAN. Abra Fontes / Reprodutor de mídia -> DLNA Server na TV.",
            "3. Transcodificação de codec (Tamanho nativo)",
            "Limite de 1080p (4K ignorado para proteger o PC). AUTO converte H.265/AV1 para H.264 sem redimensionar.",
            "4. Segundo plano e suspensão",
            "'Não suspender' evita a suspensão do PC. '✕' minimiza o servidor para a bandeja do sistema.",
            "Sobre: DLNA Video Server v2.6.0",
            "Licença: GNU GPL-3.0-or-later • Código aberto (Rust + Slint).",
        ],
        "pl" => [
            "1. Uruchamianie i foldery",
            "Kliknij 'Dodaj folder', a następnie 'URUCHOM SERWER'. 'Odśwież' skanuje nowe pliki.",
            "2. Połączenie ze Smart TV",
            "PC i TV w tej samej sieci Wi-Fi/LAN. Otwórz Źródła / Odtwarzacz multimediów -> DLNA Server w TV.",
            "3. Transkodowanie kodeka (Oryginalny rozmiar)",
            "Limit 1080p (4K pomijane dla ochrony PC). AUTO konwertuje H.265/AV1 do H.264 bez skalowania.",
            "4. Praca w tle i uśpienie",
            "'Nie usypiaj' blokuje uśpienie PC. '✕' minimalizuje serwer do zasobnika systemowego.",
            "O programie: DLNA Video Server v2.6.0",
            "Licencja: GNU GPL-3.0-or-later • Open Source (Rust + Slint).",
        ],
        "uk" => [
            "1. Запуск і папки",
            "Натисніть «Додати папку», потім «ЗАПУСТИТИ СЕРВЕР». Нові файли — «Оновити».",
            "2. Підключення Smart TV",
            "ПК і ТБ в одній мережі Wi-Fi/LAN. На ТБ відкрийте «Джерела / Медіаплеєр» → DLNA Server.",
            "3. Перекодування кодека (без стиснення розміру)",
            "Межа транскодера — 1080p (файли 4K не перекодуються для захисту ПК). АВТО — лише H.265 та рідкісні формати.",
            "4. Робота у фоні та сон",
            "«Не спати» блокує сплячий режим ПК. Кнопка «✕» згортає сервер у трей без обриву відео.",
            "Про програму: DLNA Video Server v2.6.0",
            "Ліцензія: GNU GPL-3.0-or-later • Відкритий вихідний код (Rust + Slint).",
        ],
        "zh" => [
            "1. 启动与文件夹",
            "点击“添加文件夹”，然后点击“启动服务器”。添加新文件后请点击“刷新”。",
            "2. 智能电视连接",
            "电脑和电视需在同一 Wi-Fi/局域网下。在电视上打开“信号源 / 媒体播放器” → DLNA Server。",
            "3. 编码转换（保持原始分辨率）",
            "最高支持 1080p（跳过 4K 以保护电脑）。自动模式仅将 H.265/AV1 转为 H.264，不缩放画面。",
            "4. 后台运行与防休眠",
            "“防止休眠”可阻止电脑进入睡眠。点击“✕”可将服务器最小化到系统托盘。",
            "关于：DLNA Video Server v2.6.0",
            "许可证：GNU GPL-3.0-or-later • 开源软件 (Rust + Slint)。",
        ],
        "ja" => [
            "1. 起動とフォルダ",
            "「フォルダを追加」をクリックし、「サーバー起動」を押します。新しいファイルは「更新」で反映されます。",
            "2. スマートTV接続",
            "PCとTVを同じWi-Fi/LANに接続し、TVの「入力切換 / メディアプレーヤー」→ DLNA Server を開きます。",
            "3. コーデック変換（元の解像度を維持）",
            "上限は1080p（PC保護のため4Kは変換しません）。自動モードはH.265/AV1のみH.264に変換します。",
            "4. バックグラウンドとスリープ防止",
            "「スリープ防止」でPCの休止を防ぎます。「✕」でシステムトレイに最小化します。",
            "バージョン情報: DLNA Video Server v2.6.0",
            "ライセンス: GNU GPL-3.0-or-later • オープンソース (Rust + Slint)。",
        ],
        "ko" => [
            "1. 시작 및 폴더",
            "'폴더 추가'를 누른 후 '서버 시작'을 누르세요. 새 파일은 '새로고침'으로 스캔합니다.",
            "2. 스마트 TV 연결",
            "PC와 TV가 같은 Wi-Fi/LAN에 있어야 합니다. TV에서 '외부입력 / 미디어 플레이어' → DLNA Server를 여세요.",
            "3. 코덱 트랜스코딩 (원본 해상도 유지)",
            "최대 1080p 제한 (PC 보호를 위해 4K 제외). 자동 모드는 H.265/AV1만 H.264로 변환합니다.",
            "4. 백그라운드 및 절전 방지",
            "'절전 방지'는 PC의 대기 모드를 막습니다. '✕' 버튼은 서버를 시스템 트레이로 최소화합니다.",
            "정보: DLNA Video Server v2.6.0",
            "라이선스: GNU GPL-3.0-or-later • 오픈 소스 (Rust + Slint).",
        ],
        _ => [
            "1. Запуск и папки",
            "Нажмите «Добавить папку», затем «ЗАПУСТИТЬ СЕРВЕР». Новые файлы — «Обновить».",
            "2. Подключение Smart TV",
            "ПК и ТВ в одной сети Wi-Fi/LAN. На ТВ откройте «Источники / Медиаплеер» → DLNA Server.",
            "3. Перекодирование кодека (без сжатия размера)",
            "Предел транскодера — 1080p (файлы 4K не перекодируются для защиты ПК). АВТО — только H.265 и редкие форматы.",
            "4. Работа в фоне и сон",
            "«Не спать» блокирует спящий режим ПК. Кнопка «✕» сворачивает сервер в трей без обрыва видео.",
            "О программе: DLNA Video Server v2.6.0",
            "Лицензия: GNU GPL-3.0-or-later • Открытый исходный код (Rust + Slint).",
        ],
    }
}

pub fn apply_language(ui: &MainWindow, lang: &str) {
    let t = texts_for(lang);
    ui.set_lang_label(t.code.into());
    ui.set_txt_on_air(t.on_air.into());
    ui.set_txt_stopped(t.stopped.into());
    ui.set_txt_folders_title(t.folders_title.into());
    ui.set_txt_empty_hint(t.empty_hint.into());
    ui.set_txt_add_folder(t.add_folder.into());
    ui.set_txt_remove_folder(t.remove_folder.into());
    ui.set_txt_rescan(t.rescan.into());
    ui.set_txt_sleep_on(t.sleep_on.into());
    ui.set_txt_sleep_off(t.sleep_off.into());
    ui.set_txt_auto_on(t.auto_on.into());
    ui.set_txt_auto_off(t.auto_off.into());
    ui.set_txt_start_btn(t.start_btn.into());
    ui.set_txt_stop_btn(t.stop_btn.into());
    let h = help_texts_for(lang);
    ui.set_help_t1(h[0].into());
    ui.set_help_d1(h[1].into());
    ui.set_help_t2(h[2].into());
    ui.set_help_d2(h[3].into());
    ui.set_help_t3(h[4].into());
    ui.set_help_d3(h[5].into());
    ui.set_help_t4(h[6].into());
    ui.set_help_d4(h[7].into());
    ui.set_help_t5(h[8].into());
    ui.set_help_d5(h[9].into());
    if !ui.get_is_running() {
        ui.set_status(t.ready_status.into());
    }
}
