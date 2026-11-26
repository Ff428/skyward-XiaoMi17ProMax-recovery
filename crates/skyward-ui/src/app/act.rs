//! What a control does once it is tapped, typed on or swiped.

use super::*;

impl App {
    /// Carry out a tapped control.
    pub(super) fn act(&mut self, action: Action, now: u64) -> Vec<Effect> {
        let l = self.state.lang;
        match action {
            Action::Open(s) => {
                // The served folder is chosen once: opening the page it is
                // served on ends the choosing, however it was got to.
                if s == Screen::Serve {
                    self.state.picking = false;
                }
                if s == Screen::Search && self.screen() == Screen::Home {
                    // The bar moves up from where it stands unfolded: a
                    // page easing there for the keys gets there at once.
                    if let Some((rest, _)) = self.seek.take() {
                        self.top_mut().1 = rest;
                    }
                    self.start_morph(true, now);
                    self.push(s);
                } else {
                    self.open(s);
                }
                if s == Screen::StoppedApps {
                    // 这一页的内容就是 pm-state 里那几个，进页先读一次。
                    let mut fx = self.arrive_at(s);
                    fx.push(Effect::ReadAppStates);
                    return fx;
                }
                return self.arrive_at(s);
            }
            // 录屏：没在录就开始，在录就停；还没解密时先说清楚。
            Action::Record => return self.record_toggle(now),
            Action::RecordSetting(which) => return self.record_cycle(which, now),
            Action::OpenData => return self.open_data(),
            // The order the file manager shows entries in: the key cycles, the
            // side turns, and the folders stay on top or mix in.
            Action::CycleSort => {
                self.state.sort = match self.state.sort {
                    SortKey::Name => SortKey::Size,
                    SortKey::Size => SortKey::Modified,
                    SortKey::Modified => SortKey::Type,
                    SortKey::Type => SortKey::Name,
                };
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            Action::ToggleSortOrder => {
                self.state.sort_desc = !self.state.sort_desc;
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            // 文件管理那四枚圆钮里的一枚：排序收进面板，面板里点哪一项就按它排
            // （2026-11-24）。
            Action::SortPanel(on) => {
                self.state.sort_panel = on;
                self.refresh();
            }
            Action::SortTo(key) => {
                self.state.sort = key;
                self.state.sort_panel = false;
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            Action::SortDesc(desc) => {
                self.state.sort_desc = desc;
                self.state.sort_panel = false;
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            Action::ToggleFoldersFirst => {
                self.state.folders_first = !self.state.folders_first;
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            // A row turns the check the page shows; the swipe below writes both.
            Action::Avb(which) => {
                let (mut verity, mut verification) = match (self.state.avb_pending, &self.state.avb) {
                    (Some(pending), _) => pending,
                    (None, Some(Ok(now))) => *now,
                    _ => (true, true),
                };
                if which {
                    verification = !verification;
                } else {
                    verity = !verity;
                }
                self.state.avb_pending = Some((verity, verification));
                self.refresh();
            }
            Action::VbmetaDump => return vec![Effect::VbmetaDump],
            Action::VbmetaPick(k) => {
                self.state.vbmeta_choice = Some(k);
                self.state.vbmeta_note = None;
                self.refresh();
            }
            Action::FetchStart => {
                let url = self.state.fetch_url.trim().to_string();
                if url.is_empty() {
                    self.say(self.state.lang.pick(["请先输入网址", "請先輸入網址", "Type an address first"]), now);
                } else if self.fetching() {
                    self.say(self.state.lang.pick(["已有下载在进行", "已有下載在進行", "A download is under way"]), now);
                } else {
                    self.state.fetch = Fetch::Running { done: 0, total: None };
                    return vec![Effect::Fetch(url)];
                }
            }
            Action::FetchStop => {
                if self.fetching() {
                    return vec![Effect::StopFetch];
                }
            }
            // An address too long to type can be put in a file from a computer
            // and taken from there with one tap.
            Action::FetchFromFile => {
                let at = "/sdcard/Download/link.txt";
                match std::fs::read_to_string(at) {
                    Ok(text) => {
                        let url = text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or("").to_string();
                        let said = match url.is_empty() {
                            true => self.state.lang.pick(["文件里没有链接", "檔案裡沒有連結", "The file holds no link"]).to_string(),
                            false => {
                                self.state.fetch_url = url;
                                tr!(self.state.lang, "已从文件读入链接", "已從檔案讀入連結", "The link was taken from the file").to_string()
                            }
                        };
                        self.say(&said, now);
                    }
                    Err(e) => {
                        let said = format!("{at}: {e}");
                        self.say(&said, now);
                    }
                }
                self.dirty = true;
            }
            // The folder served is the file manager's own: the switch serves
            // it or stops serving it, and changing it opens the file manager,
            // which the page returns to.
            Action::ServeToggle => {
                let on = !matches!(self.state.serving, Serving::Running { .. });
                return vec![Effect::Serve(on)];
            }
            Action::ServePick => {
                self.state.picking = true;
                return self.act(Action::Open(Screen::Files), now);
            }
            // The folder shown is the one that will be served: the bar at the
            // foot of the file manager takes it.
            Action::ServeUseHere => {
                self.state.picking = false;
                let mut fx = self.act(Action::Open(Screen::Serve), now);
                fx.insert(0, Effect::Serve(true));
                return fx;
            }
            Action::Skip => {
                self.leave(true);
                if self.stack.len() > 1 {
                    self.stack.pop();
                } else {
                    *self.top_mut() = (Screen::Home, 0.0, Focus::Unset);
                }
                self.settle_page();
                self.refresh();
            }
            // The card asking to delete is what the page shows: back puts it
            // away before it leaves the page.
            // 排序那一屏不是另一页：返回先把它收起来（2026-11-24）。
            Action::Back if self.state.sort_panel => {
                self.state.sort_panel = false;
                self.refresh();
                return Vec::new();
            }
            Action::Back if self.state.asking_delete || self.state.asking_many.is_some() => {
                self.state.asking_delete = false;
                self.state.asking_many = None;
                self.state.confirm = 0.0;
                self.refresh();
            }
            Action::Back | Action::CloseSearch => return self.back(now),
            Action::Theme { dark } => {
                if self.state.dark != dark {
                    self.state.dark = dark;
                    self.refresh();
                }
            }
            // The switch moves at once; the runtime then tells how the WLAN
            // came up, or went off.
            Action::Toggle(Switch::Wlan) => {
                let w = self.state.wlan.as_mut().expect("the WLAN switch shows on a phone whose WLAN is brought up");
                let on = matches!(w.power, Power::Off | Power::Failed(_));
                *w = Wlan { power: if on { Power::Starting } else { Power::Off }, ..Wlan::default() };
                self.refresh();
                return vec![Effect::Wlan(on)];
            }
            // The zone the system's gave the clock stays as the one chosen,
            // where it is one the pages list.
            Action::Toggle(Switch::SystemTimeZone) => {
                if self.state.follow_system_zone
                    && let Some(at) = self.state.zone_listed()
                {
                    self.state.time_zone = at;
                }
                self.state.follow_system_zone = !self.state.follow_system_zone;
                self.refresh();
            }
            Action::Toggle(switch) => {
                // Keeping the screen lock is a risk taken on, and it is said
                // every time it is turned on, not once: the switch stays off
                // until the slider is taken across. A second tap while the
                // risk is shown puts it away again.
                if switch == Switch::Remember {
                    if self.state.asking_remember {
                        self.state.asking_remember = false;
                        self.refresh();
                        return Vec::new();
                    }
                    if !self.state.remember {
                        self.state.asking_remember = true;
                        self.refresh();
                        return Vec::new();
                    }
                    self.state.remember = false;
                    self.refresh();
                    return vec![Effect::Remember(false)];
                }
                // A charge step is chosen, not switched: tapping the one that
                // is already chosen does nothing. The programs that settle how
                // fast a charge may run are started as the recovery comes up,
                // so the page says the step reaches the phone at the next
                // start.
                if let Switch::ChargeStep(mode) = switch {
                    if self.state.charge_mode == mode {
                        return Vec::new();
                    }
                    self.state.charge_mode = mode;
                    self.refresh();
                    return vec![Effect::ChargeStep(mode)];
                }
                // The guard is not offered on the plain step, and does nothing
                // there: nothing a charge draws at that step can warm the
                // battery enough to be worth watching.
                if switch == Switch::ThermalGuard && !self.state.charge_mode.guarded() {
                    return Vec::new();
                }
                let on = match switch {
                    Switch::Vibrate => &mut self.state.vibrate,
                    Switch::FileTransfer => &mut self.state.file_transfer,
                    Switch::Compress => &mut self.state.compress,
                    Switch::Verify => &mut self.state.verify,
                    Switch::Signature => &mut self.state.check_signature,
                    Switch::KeepRecovery => &mut self.state.keep_recovery,
                    Switch::ShowPassword => &mut self.state.show_password,
                    Switch::Remember => &mut self.state.remember,
                    Switch::WlanAuto => &mut self.state.wlan_auto,
                    Switch::WlanKeep => &mut self.state.wlan_keep,
                    Switch::ThermalGuard => &mut self.state.thermal_guard,
                    Switch::Wlan => unreachable!("the WLAN switch is turned above"),
                    Switch::ChargeStep(_) => unreachable!("a charge step is chosen above"),
                    Switch::SystemTimeZone => unreachable!("the switch is turned above"),
                };
                *on = !*on;
                let now_on = *on;
                self.refresh();
                if switch == Switch::FileTransfer {
                    return vec![Effect::FileTransfer(now_on)];
                }
                if switch == Switch::Remember {
                    return vec![Effect::Remember(now_on)];
                }
                if switch == Switch::WlanAuto {
                    return vec![Effect::WlanAuto(now_on)];
                }
                if switch == Switch::WlanKeep {
                    return vec![Effect::WlanKeep(now_on)];
                }
            }
            Action::MountToggle(i) => {
                if self.state.mount_failure.take().is_some() {
                    self.refresh();
                }
                let row = &self.state.mounts.as_ref().expect("a partition toggled is one the page lists")[i];
                let base = row.base.clone();
                return vec![if row.mounted { Effect::Unmount(base) } else { Effect::Mount(base) }];
            }
            Action::RebootTarget(i) => {
                self.state.reboot = i;
                self.refresh();
            }
            // Everything takes in every part, so choosing it leaves the
            // parts out, and choosing a part on its own leaves it out.
            Action::WipeItem(item) => {
                let k = WIPES.iter().position(|&w| w == item).expect("the page offers only what it lists");
                let on = !self.state.wipe[k];
                match (on, item) {
                    (true, Wipe::All) => {
                        self.state.wipe = [false; WIPES.len()];
                        self.state.wipe[k] = true;
                    }
                    (true, _) => {
                        self.state.wipe[0] = false;
                        self.state.wipe[k] = true;
                    }
                    (false, _) => self.state.wipe[k] = false,
                }
                self.refresh();
            }
            Action::Confirm(_) => self.say(l.pick(["请按住圆形按钮向右滑动", "請按住圓形按鈕向右滑動", "Hold the round button and slide it right"]), now),
            Action::Slider(_) | Action::Pattern => {}
            Action::Sideload => match self.state.install {
                Install::Idle | Install::Finished { .. } => {
                    self.begin_install(Install::Starting, Source::Computer, None);
                    if self.screen() == Screen::Installing {
                        self.refresh();
                    } else {
                        self.open(Screen::Installing);
                    }
                    return vec![Effect::Sideload];
                }
                _ => self.say(l.pick(INSTALL_UNFINISHED), now),
            },
            Action::FlashPartition(k) => {
                if let Install::Choosing(choice) = &mut self.state.install {
                    choice.partition = k;
                    self.refresh();
                }
            }
            Action::FlashSlots(pick) => {
                if let Install::Choosing(choice) = &mut self.state.install {
                    choice.pick = pick;
                    self.refresh();
                }
            }
            Action::SystemSize(k) => {
                if let Install::SystemChoosing(choice) = &mut self.state.install {
                    choice.size = k;
                    self.refresh();
                }
            }
            Action::PickSystem(who) => {
                if self.state.systems_work.is_none() {
                    self.state.system_pick = Some(who);
                    self.state.asking_delete = false;
                    self.state.confirm = 0.0;
                    self.refresh();
                }
            }
            Action::RebootTo(target) => return vec![Effect::Reboot(target)],
            Action::EnterRecovery => return vec![Effect::EnterRecovery],
            Action::Dismiss => {
                self.stack = vec![(Screen::Home, 0.0, Focus::Unset)];
                self.settle_page();
                self.refresh();
                return vec![Effect::Dismiss];
            }
            Action::PickEntry(op) => {
                let path = self.entry_path();
                self.state.clip = Some(Clip { op, path });
                return self.back(now);
            }
            // A folder cannot go inside itself, and an entry moved to the
            // folder it is in stays put: the entry stays picked for another
            // folder.
            Action::Paste => {
                let clip = self.state.clip.clone().expect("only a picked entry is pasted");
                let here = self.state.folder_here(self.screen());
                let (path, folder) = (std::path::Path::new(&clip.path), std::path::Path::new(&here));
                let name = path.file_name().map_or_else(|| clip.path.clone(), |n| n.to_string_lossy().into_owned());
                if folder.starts_with(path) {
                    self.say(&tr!(l, "无法将\"{name}\"放入其自身中", "無法將「{name}」放入其本身中", "“{name}” can't go inside itself"), now);
                } else if clip.op == FileOp::Move && path.parent() == Some(folder) {
                    self.say(&tr!(l, "\"{name}\"已在该文件夹中", "「{name}」已在這個資料夾中", "“{name}” is already in this folder"), now);
                } else {
                    return vec![Effect::Paste { op: clip.op, path: clip.path, into: here, clash: None }];
                }
            }
            Action::Unpick => {
                self.state.clip = None;
                self.refresh();
            }
            Action::RenameEntry => {
                self.state.new_name = self.state.file.as_ref().expect("the file page shows an entry").name.clone();
                // 改名十次有九次要打汉字：进来就是中文键盘，要英文按中/英。
                self.state.pinyin.on = true;
                self.state.pinyin.composing.clear();
                self.state.pinyin.page = 0;
                self.state.layer = Layer::Shifted;
                self.open(Screen::Rename);
            }
            Action::EditEntry => return vec![Effect::OpenText(self.entry_path())],
            Action::InstallEntry => {
                let size = self.state.file.as_ref().expect("the file page shows an entry").size;
                return self.install_opened(self.entry_path(), size, now);
            }
            Action::SaveText => {
                let ed = self.state.editor.as_ref().expect("the editor page shows a file");
                return vec![Effect::SaveText { path: ed.path.clone(), text: ed.text() }];
            }
            Action::OpenPermissions => self.open(Screen::Permissions),
            Action::ModeBit(bit) => {
                let mode = self.state.file.as_ref().expect("the permissions page shows an entry").mode;
                return vec![Effect::SetMode { path: self.entry_path(), mode: (mode ^ bit) & 0o7777 }];
            }
            Action::AskDelete => {
                self.state.asking_delete = true;
                self.state.confirm = 0.0;
                self.refresh();
            }
            // Choosing several entries of the folder shown: a tick beside each,
            // and the bar at the foot carrying what can be done with them.
            Action::ChooseMany(on) => {
                self.state.choosing = on;
                self.state.chosen.clear();
                self.refresh();
            }
            Action::ChooseEntryAt(i) => {
                let folder = &self.state.files;
                if let Ok(entries) = &folder.entries
                    && let Some(e) = entries.get(i)
                {
                    let name = e.name.clone();
                    if !self.state.chosen.remove(&name) {
                        self.state.chosen.insert(name);
                    }
                    self.refresh();
                }
            }
            Action::ChooseAll => {
                let all: Vec<String> = self.state.files.entries.as_ref().map(|e| e.iter().map(|r| r.name.clone()).collect()).unwrap_or_default();
                if self.state.chosen.len() == all.len() {
                    self.state.chosen.clear();
                } else {
                    self.state.chosen = all.into_iter().collect();
                }
                self.refresh();
            }
            Action::PasteChosen(op) => {
                let into = self.state.folder_here(self.screen());
                let paths: Vec<String> = self.state.chosen.iter().map(|n| std::path::Path::new(&into).join(n).to_string_lossy().into_owned()).collect();
                if paths.is_empty() {
                    return Vec::new();
                }
                self.state.choosing = false;
                self.state.chosen.clear();
                self.refresh();
                return vec![Effect::PasteMany { op, paths, into, clash: None }];
            }
            // Deleting several is as destructive as deleting one, and asks the
            // same way: a swipe confirms, and nothing goes before it.
            Action::AskDeleteChosen => {
                let into = self.state.files.path.clone();
                let paths: Vec<String> = self.state.chosen.iter().map(|n| std::path::Path::new(&into).join(n).to_string_lossy().into_owned()).collect();
                if paths.is_empty() {
                    return Vec::new();
                }
                self.state.asking_many = Some(paths);
                self.state.confirm = 0.0;
                self.refresh();
            }
            Action::CancelDelete => {
                self.state.asking_delete = false;
                self.state.asking_many = None;
                self.state.confirm = 0.0;
                self.refresh();
            }
            Action::Resolve(choice) => {
                let (clip, into, _) = self.state.clash.take().expect("a choice answers a clash");
                self.leave(true);
                self.stack.pop();
                self.settle_page();
                self.refresh();
                // Skipping the one entry picked leaves nothing to do.
                if choice == ClashChoice::Skip {
                    self.state.clip = None;
                    return Vec::new();
                }
                return vec![Effect::Paste { op: clip.op, path: clip.path, into, clash: Some(choice) }];
            }
            Action::StopJob => {
                let job = self.state.job.as_mut().expect("only work under way is stopped");
                if job.outcome.is_none() && !job.stopping {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopJob];
                }
            }
            Action::StopRelabel => {
                let job = self.state.relabel.as_mut().expect("only work under way is stopped");
                if job.outcome.is_none() && !job.stopping {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopRelabel];
                }
            }
            Action::OpenFilesystem(k) => {
                self.state.fs_chosen = k;
                self.state.fs_action = FsAction::Repair;
                self.open(Screen::Filesystem);
            }
            Action::ChooseFs(choice) => {
                let row = self.fs_row().expect("the page of a partition shows one that was read");
                let action = match choice {
                    FsChoice::Repair => FsAction::Repair,
                    FsChoice::Remake(k) => FsAction::Remake(row.others[k].clone()),
                };
                self.state.fs_action = action;
                self.refresh();
            }
            Action::ScreenOff(k) => {
                self.state.screen_off = k;
                self.refresh();
            }
            Action::TimeZone(k) => {
                self.state.time_zone = k;
                self.state.follow_system_zone = false;
                self.refresh();
            }
            Action::Language(l) => {
                self.state.lang = l;
                self.state.language_chosen = true;
                self.refresh();
            }
            Action::BackupPart(k) => {
                self.state.backup[k] = !self.state.backup[k];
                self.refresh();
            }
            Action::BackupTo(place) => {
                self.state.backup_to = place;
                self.refresh();
            }
            // 分区页与镜像页：选一个分区、导出一份、选一份镜像、写回或删掉。
            Action::PickDump(k) => {
                self.state.dump_pick = if self.state.dump_pick == Some(k) { None } else { Some(k) };
                self.refresh();
            }
            Action::DumpNow => {
                if let Some(name) = self.dumped_name() {
                    if !self.state.place_open(self.state.backup_to) {
                        let l = self.state.lang;
                        self.say(
                            match self.state.backup_to {
                                Place::Storage => l.pick(["内部存储还没打开，先解密", "內部儲存空間還沒開啟，先解密", "Internal storage is not open yet. Decrypt first"]),
                                Place::UsbDrive => l.pick(["U 盘还没连上", "USB 隨身碟還沒連上", "No USB drive is connected yet"]),
                            },
                            now,
                        );
                    } else {
                        let path = self.dumped_path(&name, now);
                        return vec![Effect::DumpImage { name, path }];
                    }
                }
            }
            Action::ImagePick(k) => {
                self.state.image_pick = if self.state.image_pick == Some(k) { None } else { Some(k) };
                self.refresh();
            }
            Action::ImageAction(which) => {
                self.state.image_action = which;
                self.refresh();
            }
            Action::ImageRefresh => return vec![self.find_images()],
            // All a backup holds is ticked to restore at first.
            Action::OpenBackup(i) => {
                let entry = self.state.backups[i].clone();
                let parts = entry.held.as_ref().map_or([false; 4], |h| h.parts);
                self.state.restore = Some((entry, parts));
                self.state.asking_delete = false;
                self.open(Screen::RestoreBackup);
            }
            Action::RestorePart(k) => {
                let (_, parts) = self.state.restore.as_mut().expect("the restore page shows a backup");
                parts[k] = !parts[k];
                self.refresh();
            }
            Action::StopBackup => {
                let job = self.state.backup_job.as_mut().expect("only work under way is stopped");
                if job.outcome.is_none() && !job.stopping {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopBackup];
                }
            }
            Action::LogPart(k) => {
                self.state.logs[k] = !self.state.logs[k];
                self.refresh();
            }
            Action::LogsTo(k) => {
                self.state.logs_to = k;
                self.refresh();
            }
            Action::ExportLogs => {
                let ([recovery, kernel], storage) = (self.state.logs, self.state.logs_to == 1);
                if !recovery && !kernel {
                    self.say(l.pick(["请选择要导出的日志", "請選擇要匯出的記錄", "Choose the logs to export"]), now);
                } else if storage && !self.state.data.storage_open() {
                    self.say(l.pick(DECRYPT_FIRST), now);
                } else {
                    return vec![Effect::ExportLogs { recovery, kernel, storage }];
                }
            }
            Action::PickSlot(slot) => {
                self.state.next_slot = Some(slot);
                self.refresh();
            }
            // 「中/英」这一枚在哪一页都先在这里接管：终端、编辑器、重命名框各有
            // 自己的一套按键处理，以前它们把这一枚当没按，于是那几页看着像「切
            // 不过去」（2026-11-23）。
            Action::Type(Cap::Language) => return self.type_key(Cap::Language, now),
            // 终端与编辑器上的中文键盘也走拼音：按下去攒串、候选选出来落汉字。
            Action::Type(cap) if self.pinyin_takes(cap) && matches!(self.screen(), Screen::Terminal | Screen::Editor) => {
                return self.type_key(cap, now);
            }
            Action::Type(cap) if self.screen() == Screen::Terminal => return self.term_typed(cap),
            Action::Type(cap) if self.screen() == Screen::Editor => self.edit_typed(cap),
            Action::Type(cap) => return self.type_key(cap, now),
            Action::TermKey(key) if self.screen() == Screen::Editor => {
                let ed = self.state.editor.as_mut().expect("the editor page shows a file");
                match key {
                    TermKey::Tab => ed.insert('\t'),
                    TermKey::Left => ed.left(),
                    TermKey::Up => ed.up(),
                    TermKey::Down => ed.down(),
                    TermKey::Right => ed.right(),
                    TermKey::Esc | TermKey::Ctrl => unreachable!("the editor's row has no {key:?}"),
                }
                self.edits += 1;
                self.refresh_text();
            }
            Action::TermKey(key) => {
                let Some(term) = self.term.as_mut() else { return Vec::new() };
                let bytes = term.key(key);
                let running = term.running;
                self.refresh();
                if running && !bytes.is_empty() {
                    return vec![Effect::TermInput(bytes)];
                }
            }
            Action::ClearSearch => {
                self.state.search.clear();
                self.edits += 1;
                self.refresh();
            }
            Action::OpenEntry(i) => {
                let entry = self.state.files.entry(i).clone();
                match entry.kind {
                    FileKind::Folder { .. } | FileKind::Link { folder: true, .. } => {
                        let scroll = self.top().1;
                        self.state.files.leave(scroll);
                        return vec![Effect::ReadDir(self.state.files.inside(&entry.name))];
                    }
                    FileKind::File if matches!(files::file_type(&entry.name), FileType::Text | FileType::Code) => {
                        return vec![Effect::OpenText(self.state.files.inside(&entry.name))];
                    }
                    _ if installs(&entry) => {
                        let path = self.state.files.inside(&entry.name);
                        return self.install_opened(path, entry.size, now);
                    }
                    FileKind::File | FileKind::Link { folder: false, .. } | FileKind::Special => {
                        self.state.file = Some(entry);
                        self.open(Screen::FileInfo);
                    }
                }
            }
            // A shell already running goes to the folder; otherwise one starts
            // there.
            Action::TerminalHere => {
                let folder = self.state.files.path.clone();
                let running = self.term.as_ref().is_some_and(|term| term.running);
                self.open(Screen::Terminal);
                self.state.layer = Layer::Letters;
                if running {
                    return vec![Effect::ShellTo(folder)];
                }
                self.shell_folder = folder.clone();
                return vec![Effect::StartShell(folder)];
            }
            Action::ChooseEntry(i) => {
                let screen = self.screen();
                // A tap that lands as the page turns is nobody's mistake: the
                // place's page is the only one that chooses, and one that is
                // not showing has nothing to choose.
                let Screen::Choose(place) = screen else { return Vec::new() };
                let folder = self.state.folder(screen);
                let Ok(entries) = &folder.entries else { return Vec::new() };
                let Some(entry) = entries.get(i).cloned() else { return Vec::new() };
                let path = folder.inside(&entry.name);
                match entry.kind {
                    FileKind::Folder { .. } | FileKind::Link { folder: true, .. } => {
                        let scroll = self.top().1;
                        self.state.folder_mut(screen).leave(scroll);
                        return vec![Effect::ReadPlace(place, path)];
                    }
                    FileKind::File | FileKind::Link { folder: false, .. } => return self.install_file(place, path, entry.size, now),
                    FileKind::Special => unreachable!("a place lists no special files"),
                }
            }
            Action::OpenAncestor(depth) => {
                let screen = self.screen();
                if let Some(path) = self.state.folder(screen).ancestor(depth) {
                    return vec![match screen {
                        Screen::Choose(place) => Effect::ReadPlace(place, path),
                        _ => Effect::ReadDir(path),
                    }];
                }
            }
            Action::JoinNetwork(i) => {
                let w = self.state.wlan.as_ref().expect("networks show on a phone whose WLAN is brought up");
                // A scan may have changed the list since the page was drawn.
                let Some(network) = w.networks.get(i).cloned() else { return Vec::new() };
                if w.joined.as_ref().is_some_and(|j| j.ssid == network.ssid) {
                    return self.act(Action::OpenJoined, now);
                }
                match network.security {
                    Security::Wep => self.say(l.pick(["暂不支持 WEP 加密的网络", "尚不支援 WEP 加密的網路", "WEP networks aren't supported"]), now),
                    Security::Enterprise => {
                        self.say(l.pick(["暂不支持需要账号登录的网络", "尚不支援需要帳號登入的網路", "Networks that need an account sign-in aren't supported"]), now)
                    }
                    s if s.password() => self.ask_password(network),
                    _ => {
                        return vec![Effect::JoinWlan {
                            ssid: network.ssid,
                            name: network.name,
                            security: network.security,
                            password: Secret::default(),
                        }]
                    }
                }
            }
            Action::NetworkDetails(i) => {
                let w = self.state.wlan.as_ref().expect("networks show on a phone whose WLAN is brought up");
                let Some(network) = w.networks.get(i).cloned() else { return Vec::new() };
                self.state.details = Some(network);
                self.open(Screen::WlanNetwork);
            }
            Action::OpenJoined => {
                let w = self.state.wlan.as_ref().expect("a network is chosen on a phone whose WLAN is brought up");
                let Some(j) = w.joined.clone() else { return Vec::new() };
                // As found, where it still is; as it was chosen otherwise.
                let found = w.networks.iter().find(|n| n.ssid == j.ssid).cloned();
                let network = found.unwrap_or(Network {
                    ssid: j.ssid,
                    name: j.name,
                    signal: 0,
                    frequency: 0,
                    bands: Default::default(),
                    security: j.security,
                });
                if j.stage == Joining::WrongPassword {
                    self.ask_password(network);
                } else {
                    self.state.details = Some(network);
                    self.open(Screen::WlanNetwork);
                }
            }
            // 菜单里记住的那一张：点一下就发它的 SSID，密码由后台从名单里取。
            Action::JoinKept(k) => {
                if let Some(saved) = self.state.wlan_saved.get(k).cloned() {
                    return vec![Effect::JoinKept { ssid: saved.ssid, name: saved.name, security: saved.security }];
                }
            }
            Action::KeptDetails(k) => {
                if let Some(saved) = self.state.wlan_saved.get(k).cloned() {
                    // 名单里记着的那张在附近可能根本没扫到：详情页照样打开，
                    // 删掉它就在那里（2026-11-24 用户说已保存的进不去管理界面）。
                    self.state.details = Some(Network {
                        ssid: saved.ssid,
                        name: saved.name,
                        signal: 0,
                        frequency: 0,
                        bands: Bands::default(),
                        security: saved.security,
                    });
                    self.open(Screen::WlanNetwork);
                }
            }
            Action::ForgetKept(k) => {
                if let Some(saved) = self.state.wlan_saved.get(k) {
                    return vec![Effect::ForgetKept { ssid: saved.ssid.clone() }];
                }
            }
            Action::LeaveNetwork => {
                // 详情页是「记住的那张、但没连着」：删的是名单里的它，不是把
                // 现在连着的网络踢掉（2026-11-24）。
                let here = self.state.details.as_ref().map(|n| n.ssid.clone());
                let joined = self.state.wlan.as_ref().and_then(|w| w.joined.as_ref()).map(|j| j.ssid.clone());
                let mut fx = self.back(now);
                match here {
                    Some(ssid) if Some(ssid.clone()) != joined => fx.push(Effect::ForgetKept { ssid }),
                    _ => fx.push(Effect::LeaveWlan),
                }
                return fx;
            }
            Action::Rescan => return vec![Effect::ScanWlan],
            Action::AddNetwork => {
                self.state.adding = Some(Adding { name: String::new(), security: Security::Open, field: Field::Name, choosing: false });
                self.state.password.clear();
                self.state.show_password = false;
                self.state.layer = Layer::Letters;
                self.open(Screen::WlanAdd);
            }
            Action::TypeInto(field) => {
                let a = self.state.adding.as_mut().expect("the fields are on the page that adds a network");
                if a.field != field {
                    a.field = field;
                    self.refresh();
                }
            }
            Action::ChooseSecurity => {
                self.state.adding.as_mut().expect("the protection is on the page that adds a network").choosing = true;
                // The page draws the row held down while its list is open.
                self.row_press = None;
                self.row_art = None;
                self.refresh();
            }
            Action::PickSecurity(security) => {
                let a = self.state.adding.as_mut().expect("the protections open on the page that adds a network");
                a.security = security;
                a.choosing = false;
                // With no password to type, the keyboard types the name.
                if !security.password() {
                    a.field = Field::Name;
                }
                self.refresh();
            }
            Action::CloseChoices => {
                self.state.adding.as_mut().expect("the protections open on the page that adds a network").choosing = false;
                self.refresh();
            }
            // Asking takes a network, which the WLAN page joins.
            Action::CheckUpdate => {
                if !self.state.online() {
                    self.open(Screen::Wlan);
                    return self.arrive_at(Screen::Wlan);
                }
                if self.state.check != Check::Asking {
                    self.state.check = Check::Asking;
                    self.refresh();
                    return vec![Effect::CheckUpdate];
                }
            }
            // The package goes to internal storage, which the decrypt page
            // opens, over a network the WLAN page joins.
            Action::Download => {
                let release = self.state.newest().expect("the download is offered for the newest system").clone();
                if !self.state.data.storage_open() {
                    self.open(Screen::Decrypt);
                    return self.arrive_at(Screen::Decrypt);
                }
                if !self.state.online() {
                    self.open(Screen::Wlan);
                    return self.arrive_at(Screen::Wlan);
                }
                if !self.state.downloading() {
                    self.state.download = Some(DownloadJob { progress: None, rate: 0, outcome: None, stopping: false });
                    self.refresh();
                    return vec![Effect::Download(release)];
                }
            }
            Action::StopDownload => {
                let job = self.state.download.as_mut().expect("only a download under way is stopped");
                if job.outcome.is_none() && !job.stopping {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopDownload];
                }
            }
            Action::InstallDownloaded => {
                let name = self.state.newest().expect("the package installed is the newest system's").name.clone();
                let Have::Whole(size) = self.state.have else { unreachable!("only a whole package is offered to install") };
                return self.install_file(Place::Storage, format!("{DOWNLOADS}/{name}"), size, now);
            }
            // Asking takes a network, which the WLAN page joins.
            Action::CheckSkyward => {
                if !self.state.online() {
                    self.open(Screen::Wlan);
                    return self.arrive_at(Screen::Wlan);
                }
                if self.state.skyward_check != SkywardCheck::Asking {
                    self.state.skyward_check = SkywardCheck::Asking;
                    self.refresh();
                    return vec![Effect::CheckSkyward];
                }
            }
            // The build goes into the recovery partitions an install may
            // write too, so one waits for the other.
            Action::UpdateSkyward => {
                let build = self.state.skyward_newest().and_then(|n| n.build.clone()).expect("the update is offered for the newest release's build");
                if !self.state.online() {
                    self.open(Screen::Wlan);
                    return self.arrive_at(Screen::Wlan);
                }
                if !matches!(self.state.install, Install::Idle | Install::Finished { .. }) {
                    self.say(l.pick(INSTALL_UNFINISHED), now);
                    return Vec::new();
                }
                if !self.state.updating_skyward() {
                    self.state.skyward_job = Some(SkywardJob { stage: None, rate: 0, outcome: None, stopping: false });
                    self.refresh();
                    return vec![Effect::UpdateSkyward(build)];
                }
            }
            // Only the download stops: writing, once begun, goes on to its
            // end.
            Action::StopSkywardUpdate => {
                let job = self.state.skyward_job.as_mut().expect("only an update under way is stopped");
                if job.outcome.is_none() && !job.stopping && !matches!(job.stage, Some(SkywardStage::Write { .. })) {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopSkywardUpdate];
                }
            }
            // The OTG drive's files: listed on a page of their own, and an
            // entry in it opens as any other folder's does.
            // 快捷进去：路径直接换成 /sdcard（就是 /data/media/0），再读它。
            // 页面本来就是文件页，翻页带照旧从顶上起。
            Action::FilesSdCard => {
                let scroll = self.top().1;
                self.state.files.leave(scroll);
                return vec![Effect::ReadDir(super::SDCARD.to_string())];
            }
            Action::FilesOtg => {
                if self.state.place_open(Place::UsbDrive) {
                    self.state.otg = Folder::new("");
                    self.open(Screen::FilesOtg);
                    return vec![Effect::ReadOtg(String::new())];
                }
                self.say(l.pick(["未连接 U 盘", "未連接 USB 隨身碟", "No USB drive"]), now);
            }
            // 外置 SD 卡：先让 runtime 挂上（有卡才有得挂），挂好再列。
            Action::OpenOtgEntry(i) => {
                let entry = self.state.otg.entry(i).clone();
                match entry.kind {
                    FileKind::Folder { .. } | FileKind::Link { folder: true, .. } => {
                        self.state.otg.leave(self.top().1);
                        return vec![Effect::ReadOtg(self.state.otg.inside(&entry.name))];
                    }
                    _ => {
                        self.state.file_on_drive = true;
                        self.state.file = Some(entry);
                        self.open(Screen::FileInfo);
                    }
                }
            }
            // Across to the other side, the file kept picked: the page it
            // goes to is opened so that the folder it lands in can be walked
            // to, and the bar at the foot puts it down there.
            // 数据分区的应用：点一行看详情，卸载在详情里。
            // 搜索页这次找的是应用或模块：清空查询，进搜索页说清楚范围。
            Action::SearchFileOpen(i) => {
                self.open(Screen::Files);
                return self.act(Action::OpenEntry(i), now);
            }
            Action::SearchFiles => {
                self.state.search_scope = crate::app::SearchScope::Files;
                self.state.search.clear();
                return self.act(Action::Open(Screen::Search), now);
            }
            Action::SearchApps | Action::SearchModules => {
                self.state.search_scope = if matches!(action, Action::SearchApps) {
                    crate::app::SearchScope::Apps
                } else {
                    crate::app::SearchScope::Modules
                };
                self.state.search.clear();
                return self.act(Action::Open(Screen::Search), now);
            }
            Action::ModulePage(by) => {
                let page = self.state.module_page as i32 + by;
                self.state.module_page = page.max(0) as usize;
                self.refresh();
                return Vec::new();
            }
            Action::SysPage(by) => {
                let page = self.state.sys_page as i32 + by;
                self.state.sys_page = page.max(0) as usize;
                self.refresh();
                return Vec::new();
            }
            // 系统应用也点得开：详情页跟用户应用是同一页，只是没有卸载，
            // 换成把安装包导出到内部存储（2026-11-19）。
            Action::SysAppOpen(k) => {
                let Some(list) = &self.state.sys_apps else { return Vec::new() };
                if list.get(k).is_none() {
                    return Vec::new();
                }
                self.state.app_at = None;
                self.state.sys_at = Some(k);
                return self.act(Action::Open(Screen::DataAppInfo), now);
            }
            Action::SysAppExport(k) => {
                let Some(list) = &self.state.sys_apps else { return Vec::new() };
                let Some(a) = list.get(k) else { return Vec::new() };
                return vec![Effect::ExportApk { from: a.dir.clone(), name: a.package.clone() }];
            }
            // 停用/启用系统应用：包管理器的状态在系统的 data 里，recovery
            // 改不了；把请求排进队列，系统启动后由它自己执行（2026-11-23）。
            // 已停用列表里的第 k 条：排一个启用回来。
            Action::StoppedEnable(k) => {
                let Some(map) = &self.state.app_disabled else { return Vec::new() };
                let stopped: Vec<String> =
                    map.iter().filter(|(_, off)| **off).map(|(package, _)| package.clone()).collect();
                let Some(package) = stopped.get(k).cloned() else { return Vec::new() };
                self.state.app_state_queued.insert(package.clone(), false);
                return vec![Effect::QueueAppState { package, disable: false }];
            }
            Action::SysAppDisable(k) => {
                let Some(list) = &self.state.sys_apps else { return Vec::new() };
                let Some(a) = list.get(k) else { return Vec::new() };
                self.state.app_state_queued.insert(a.package.clone(), true);
                return vec![Effect::QueueAppState { package: a.package.clone(), disable: true }];
            }
            Action::SysAppEnable(k) => {
                let Some(list) = &self.state.sys_apps else { return Vec::new() };
                let Some(a) = list.get(k) else { return Vec::new() };
                self.state.app_state_queued.insert(a.package.clone(), false);
                return vec![Effect::QueueAppState { package: a.package.clone(), disable: false }];
            }
            Action::AppScope(scope) => {
                self.state.apps_scope = scope;
                self.state.app_page = 0;
                self.refresh();
                return Vec::new();
            }
            Action::AppPage(by) => {
                self.state.app_page_taps += 1;
                let page = self.state.app_page as i32 + by;
                self.state.app_page = page.max(0) as usize;
                self.refresh();
                return Vec::new();
            }
            Action::DataAppOpen(k) => {
                self.state.app_at = Some(k);
                self.state.sys_at = None;
                return self.act(Action::Open(Screen::DataAppInfo), now);
            }
            // 卸载要先滑动确认：这里只把卡片立起来，真删在 Confirm 里。
            Action::DataAppRemove(k) => {
                let Some(Ok(list)) = &self.state.apps else { return Vec::new() };
                if list.get(k).is_none() {
                    return Vec::new();
                }
                self.state.app_confirm = Some(k);
                self.refresh();
                return Vec::new();
            }
            // 列表点到一行：打开这一行的详情，那儿才有开关和卸载。
            Action::ModuleOpen(k) => {
                self.state.module_at = Some(k);
                return self.act(Action::Open(Screen::ModuleInfo), now);
            }
            // 选 KernelSU 列表里这个位置的 KMI：回到上一页，页上就写它。
            Action::KernelSuKmi(at) => {
                self.set_kernel_su_kmi(at);
                return self.back(now);
            }
            // 用 KMI 自带的内核模块，不用本地选的。
            Action::KernelSuModuleBuiltIn => {
                self.state.kernel_su_ko = None;
                self.refresh();
            }
            // 把备份的 KernelSU 原厂 boot 写回去。
            Action::RestoreKernelSu => {
                if self.state.data != Data::Open {
                    self.say(l.pick(["内部存储未打开，请先解密 data。", "內部儲存未開啟，請先解密 data。", "Internal storage is not open; decrypt data first."]), now);
                } else if self.state.kernel_su_job.as_ref().is_some_and(|j| j.outcome.is_none()) {
                    self.say(l.pick(["正在写入 KernelSU，请等待完成。", "正在寫入 KernelSU，請等待完成。", "Writing KernelSU; wait for it to finish."]), now);
                } else {
                    self.kernel_su_started();
                    return vec![Effect::RestoreKernelSu];
                }
            }
            // 把备份的 Magisk 原厂 boot 写回去。
            Action::RestoreMagisk => {
                if self.state.data != Data::Open {
                    self.say(l.pick(["内部存储未打开，请先解密 data。", "內部儲存未開啟，請先解密 data。", "Internal storage is not open; decrypt data first."]), now);
                } else if self.state.magisk_job.as_ref().is_some_and(|j| j.outcome.is_none()) {
                    self.say(l.pick(["正在写入 Magisk，请等待完成。", "正在寫入 Magisk，請等待完成。", "Writing Magisk; wait for it to finish."]), now);
                } else {
                    self.magisk_started();
                    return vec![Effect::RestoreMagisk];
                }
            }
            // 标记删除（或取消标记）：KSU/Magisk 都是看模块目录里有没有
            // `remove` 文件，有就在下次启动时清掉。
            Action::ModuleRemove(k) => {
                let Some(Ok(list)) = &self.state.modules else { return Vec::new() };
                if list.get(k).is_none() {
                    return Vec::new();
                }
                self.state.module_confirm = Some(k);
                self.refresh();
                return Vec::new();
            }
            // 文件页上那个 zip 当成模块装：zip 里必须有 module.prop。
            Action::InstallAsModule => {
                let Some(e) = &self.state.file else { return Vec::new() };
                let zip = e.name.to_lowercase().ends_with(".zip");
                if e.kind != FileKind::File || !zip {
                    self.say(l.pick(["只有 zip 能当模块安装", "只有 zip 能當模組安裝", "Only a zip can be a module"]), now);
                    return Vec::new();
                }
                let path = std::path::Path::new(&self.state.files.path).join(&e.name).to_string_lossy().into_owned();
                return vec![Effect::ModuleInstall(path)];
            }
            // 模块的开关交给 runtime 去做：写或删那个 disable 文件，再读一遍。
            Action::ModuleToggle(k) => {
                let Some(Ok(list)) = &self.state.modules else { return Vec::new() };
                let Some(m) = list.get(k) else { return Vec::new() };
                return vec![Effect::ModuleSwitch(m.id.clone(), !m.enabled)];
            }
            Action::AskPlace(op) => {
                self.state.place_ask = Some(op);
                self.refresh();
                return Vec::new();
            }
            Action::DismissPlace => {
                self.state.place_ask = None;
                self.refresh();
                return Vec::new();
            }
            Action::ToDrive(op) | Action::ToPhone(op) => {
                self.state.place_ask = None;
                let to_drive = matches!(action, Action::ToDrive(_));
                if to_drive && !self.state.place_open(Place::UsbDrive) {
                    self.say(l.pick(["请先连接 U 盘", "請先連接 USB 隨身碟", "Connect a USB drive first"]), now);
                    return Vec::new();
                }
                let path = self.entry_path();
                // The entry's own folder is no place to put it down again, and
                // a folder cannot go inside itself.
                let here = match to_drive {
                    true => self.state.otg.path.trim_matches('/').to_string(),
                    false => self.state.files.path.clone(),
                };
                let nothing_to_do = match to_drive {
                    true => false,
                    false => op == FileOp::Move && std::path::Path::new(&path).parent() == Some(std::path::Path::new(&here)),
                };
                if nothing_to_do {
                    let name = std::path::Path::new(&path).file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
                    self.say(&tr!(l, "\"{name}\"已在该文件夹中", "「{name}」已在這個資料夾中", "“{name}” is already in this folder"), now);
                    return Vec::new();
                }
                self.state.clip = Some(Clip { op, path });
                self.refresh();
                return match to_drive {
                    true => {
                        self.open(Screen::FilesOtg);
                        vec![Effect::ReadOtg(self.state.otg.path.clone())]
                    }
                    false => {
                        self.state.file_on_drive = false;
                        self.open(Screen::Files);
                        vec![Effect::ReadDir(self.state.files.path.clone())]
                    }
                };
            }
            Action::Unavailable(text) => self.say(text, now),
        }
        Vec::new()
    }

    /// The text the keyboard types into on this page.
    /// 录屏这一下：没在录就开始，在录就停。胶囊右边那个方块也走这里。
    pub(super) fn record_toggle(&mut self, now: u64) -> Vec<Effect> {
        if self.state.record.on {
            return vec![Effect::RecordStop];
        }
        if !self.state.place_open(Place::Storage) {
            let text = match self.state.lang {
                Lang::Hans => "录屏写在内部存储里：先完成「解密」。",
                Lang::Hant => "錄影寫在內部儲存裡：先完成「解密」。",
                Lang::En => "Recordings go to internal storage: unlock it first.",
            };
            self.say(text, now);
            return Vec::new();
        }
        vec![Effect::RecordStart]
    }

    /// 胶囊上的红点按一下：这一段停在这里，或者接着录。停着的这段时间不算
    /// 进已录时长，写进文件的时间轴也不留空档。
    pub(super) fn record_paused(&mut self, now: u64) -> Vec<Effect> {
        let r = &mut self.state.record;
        if !r.on {
            return Vec::new();
        }
        if r.paused {
            r.held += now.saturating_sub(r.paused_at);
            r.paused = false;
        } else {
            r.paused = true;
            r.paused_at = now;
        }
        let paused = r.paused;
        self.dirty = true;
        self.refresh();
        vec![Effect::RecordPause(paused), Effect::Haptic]
    }

    /// 换录屏的那一档：点一下下一档，转一圈回到第一档。正在录的时候不让改 ——
    /// 画面大小一变，编码线程与写进去的文件头就都得重来。
    fn record_cycle(&mut self, which: RecordSetting, now: u64) -> Vec<Effect> {
        if self.state.record.on {
            let text = match self.state.lang {
                Lang::Hans => "正在录制：停下这一段再改。",
                Lang::Hant => "正在錄影：停下這一段再改。",
                Lang::En => "Recording: stop this take before changing it.",
            };
            self.say(text, now);
            return Vec::new();
        }
        let r = &mut self.state.record;
        match which {
            RecordSetting::Scale => r.scale = (r.scale + 1) % crate::record::SCALES.len(),
            RecordSetting::Quality => r.quality = (r.quality + 1) % crate::record::QUALITIES.len(),
            RecordSetting::Fps => r.fps = (r.fps + 1) % crate::record::RATES.len(),
        }
        self.dirty = true;
        self.refresh();
        Vec::new()
    }

    /// 录屏开始了：页面与状态栏记下这一段。
    pub fn record_started(&mut self, file: String, spec: String, now: u64) {
        let chosen = (self.state.record.scale, self.state.record.quality, self.state.record.fps);
        self.state.record = Record {
            on: true,
            since: now,
            file,
            spec,
            elapsed: "0:00".into(),
            frames: 0,
            scale: chosen.0,
            quality: chosen.1,
            fps: chosen.2,
            // 上一段把胶囊拖到哪儿，这一段还从那儿开始。
            pill: self.state.record.pill,
            paused: false,
            paused_at: 0,
            held: 0,
        };
        self.dirty = true;
        self.refresh();
    }

    /// 录屏没能开始，为什么。
    pub fn record_failed(&mut self, why: &str, now: u64) {
        self.state.record.on = false;
        let text = match self.state.lang {
            Lang::Hans => format!("录屏没有开始：{why}"),
            Lang::Hant => format!("錄影沒有開始：{why}"),
            Lang::En => format!("Recording did not start: {why}"),
        };
        self.say(&text, now);
        self.dirty = true;
        self.refresh();
    }

    /// 录屏停了，文件收好了。暂停的那笔账一并清掉：下一段从头算，胶囊上也
    /// 不再挂着「已暂停」（2026-11-21）。
    pub fn record_stopped(&mut self, file: &str, secs: u64, now: u64) {
        self.state.record.on = false;
        self.state.record.elapsed.clear();
        self.state.record.paused = false;
        self.state.record.paused_at = 0;
        self.state.record.held = 0;
        let text = match self.state.lang {
            Lang::Hans => format!("已保存：{file}（{secs} 秒）"),
            Lang::Hant => format!("已儲存：{file}（{secs} 秒）"),
            Lang::En => format!("Saved: {file} ({secs} s)"),
        };
        self.say(&text, now);
        self.dirty = true;
        self.refresh();
    }

    /// 拼音串换成字：给了本页第几个就选它，否则选本页第一个；一条候选都没有
    /// 就把这串字母原样落下。用掉的那一段从串里去干净，剩下的继续出候选。
    /// 点的是个空槽就什么都不做，串留着。
    fn take_pinyin(&mut self, pick: Option<usize>, _now: u64) -> Vec<Effect> {
        if self.state.pinyin.composing.is_empty() {
            self.state.pinyin.page = 0;
            return Vec::new();
        }
        let (picks, pages) = self.state.pinyin_view();
        let per = crate::pinyin::PER_PAGE;
        let at = self.state.pinyin.page.min(pages - 1);
        let chosen = picks
            .get(at * per + pick.unwrap_or(0))
            .map(|p| (p.text.clone(), p.used));
        // 攒着的那一串要是拼不出任何字：没有候选时什么都不落，串留在候选条上，
        // ⌫ 或「重输」清掉即可。
        let (text, rest) = {
            let composing = &self.state.pinyin.composing;
            match chosen {
                Some((text, used)) => (text, composing[used.min(composing.len())..].to_string()),
                None if pick.is_some() => return Vec::new(),
                None => (composing.clone(), String::new()),
            }
        };
        // 字落到哪儿：终端把汉字按 UTF-8 发给 shell，编辑器插到光标处，其余页面
        // 写进各自那个输入框（2026-11-24）。
        let mut fx = Vec::new();
        match self.screen() {
            Screen::Terminal => {
                // 直接发：以前这里要 `self.term` 有值（或它的 `running` 为真），
                // 可那一份在界面这边往往还没置上，于是英文能打、中文选了候选却
                // 发不出去（2026-11-24 用户报「打不出字」）。shell 早结束了
                // 的话，后台会把它报出来。
                fx.push(Effect::TermInput(text.clone().into_bytes()));
            }
            Screen::Editor => {
                if let Some(ed) = self.state.editor.as_mut() {
                    for ch in text.chars() {
                        ed.insert(ch);
                    }
                }
            }
            _ => self.typing().push_str(&text),
        }
        self.state.pinyin.composing = rest;
        self.state.pinyin.page = 0;
        self.edits += 1;
        self.refresh();
        fx
    }

    /// 终端与编辑器上，候选条那几枚交给拼音处理；字母、空格、退格、
    /// 回车仍按它们自己的意思走（2026-11-24）。
    fn pinyin_takes(&self, cap: Cap) -> bool {
        // 中文键盘摆着就攒串，不看 `pinyin.on`：终端与编辑器进来时它是 false，
        // 是用户按「中/英」摆出来的（2026-11-24）。
        match cap {
            Cap::Pick(_) | Cap::Page(_) | Cap::Retype => true,
            // 有串的时候，空格是「选第一个」、回车是「落下」、⌫ 是「删一位」：
            // 这三枚不能被终端/编辑器自己吃掉，否则有候选也选不出来
            // （2026-11-24 用户报「打不出字」）。
            Cap::Space | Cap::Enter | Cap::Backspace => !self.state.pinyin.composing.is_empty(),
            _ => false,
        }
    }

    /// 这一页默认收不收汉字：决定「攒不攒拼音」。非中文页（密码、网址、终端、
    /// 编辑器）里，26 键的字母照旧直接落进框；但用户按过「中/英」把中文键盘摆上来
    /// 之后，攒的串照样要能打成中文（2026-11-24）。
    fn takes_chinese(&self) -> bool {
        match self.screen() {
            Screen::Decrypt | Screen::WlanPassword | Screen::Fetch => false,
            Screen::WlanAdd => matches!(self.state.adding.as_ref().map(|a| a.field), Some(Field::Name)),
            _ => true,
        }
    }

    /// 这一页的输入框。
    fn typing(&mut self) -> &mut String {
        match self.screen() {
            Screen::Search => &mut self.state.search,
            Screen::Decrypt => &mut self.state.code,
            Screen::Rename => &mut self.state.new_name,
            Screen::WlanPassword => &mut self.state.password,
            Screen::Fetch => &mut self.state.fetch_url,
            Screen::WlanAdd => {
                let a = self.state.adding.as_mut().expect("the page adds a network");
                match a.field {
                    Field::Name => &mut a.name,
                    Field::Password => &mut self.state.password,
                }
            }
            s => panic!("{s:?} has no text field"),
        }
    }

    /// A key of the on-screen keyboard. A capital lasts one letter; return
    /// opens the first result of a search, and runs a terminal's command.
    fn type_key(&mut self, cap: Cap, now: u64) -> Vec<Effect> {
        let (l, layer) = (self.state.lang, self.state.layer);
        // 上一页攒下的拼音串不跟着进非中文页（网址、密码会把汉字落进框）。
        if !self.takes_chinese() && !self.state.pinyin.on && !self.state.pinyin.composing.is_empty() {
            self.state.pinyin.composing.clear();
            self.state.pinyin.page = 0;
        }
        // 拼音条自己的三个键：切中英、翻页、选候选。
        match cap {
            Cap::Language => {
                let fx = self.take_pinyin(None, now);
                self.state.pinyin.on = !self.state.pinyin.on;
                // 中文 26 键大写、英文 26 键小写（2026-11-24）。
                self.state.layer = if self.state.pinyin.on { Layer::Shifted } else { Layer::Letters };
                self.edits += 1;
                self.refresh();
                return fx;
            }
            Cap::Page(step) => {
                let (_, pages) = self.state.pinyin_view();
                let at = self.state.pinyin.page.min(pages - 1) as i32 + step as i32;
                self.state.pinyin.page = at.clamp(0, pages as i32 - 1) as usize;
                self.refresh();
                return Vec::new();
            }
            Cap::Pick(k) => return self.take_pinyin(Some(k), now),
            // 右边那枚「重输」：攒着的拼音扔掉，框里已经落下的字不动。
            Cap::Retype => {
                self.state.pinyin.composing.clear();
                self.state.pinyin.page = 0;
                self.edits += 1;
                self.refresh();
                return Vec::new();
            }
            _ => {}
        }
        // 中文模式下攒的是 26 键上的字母（大写只是键上的字），交给 `pinyin.rs`
        // 拼成串再问解码器；这一页不收汉字（密码、网址）就照旧直接落字母。
        let _ = l;
        let spelling = self.state.pinyin.on
            && self.takes_chinese()
            && matches!(cap, Cap::Char(c) if c.is_ascii_lowercase());
        match cap {
            Cap::Char(ch) if spelling => {
                self.state.pinyin.composing.push(ch.to_ascii_lowercase());
                self.state.pinyin.page = 0;
                self.edits += 1;
                self.refresh();
            }
            Cap::Retype => {}
            Cap::Char(ch) => {
                let shifted = self.state.layer == Layer::Shifted;
                self.typing().push(if shifted { ch.to_ascii_uppercase() } else { ch });
                self.edits += 1;
                if shifted {
                    self.state.layer = Layer::Letters;
                }
            }
            // 空格先是「选第一个候选」，没有候选才落一个空格。
            Cap::Space if !self.state.pinyin.composing.is_empty() => return self.take_pinyin(None, now),
            Cap::Space => {
                self.typing().push(' ');
                self.edits += 1;
            }
            Cap::Backspace if !self.state.pinyin.composing.is_empty() => {
                self.state.pinyin.composing.pop();
                self.state.pinyin.page = 0;
                self.edits += 1;
                self.refresh();
            }
            Cap::Backspace => {
                self.typing().pop();
                self.edits += 1;
            }
            // 回车先把没选的拼音落下，不替人按到页面的按钮上。
            Cap::Enter if !self.state.pinyin.composing.is_empty() => return self.take_pinyin(None, now),
            Cap::Shift => {
                self.state.layer = if self.state.layer == Layer::Shifted { Layer::Letters } else { Layer::Shifted };
            }
            Cap::Show(layer) => self.state.layer = layer,
            // 拼音条的键在终端与编辑器上不画，落到这里就当没按。
            Cap::Pick(_) | Cap::Page(_) | Cap::Language => {}
            Cap::Enter => match self.screen() {
                Screen::Search => match self.state.search_scope {
                    crate::app::SearchScope::Features => {
                        if let Some(first) = search::find_on(&self.state.search, l, |screen| self.state.offers(screen)).first() {
                            return self.act(Action::Open(first.screen), now);
                        }
                    }
                    scope => {
                        let query = self.state.search.trim().to_lowercase();
                        let Some(first) = self.first_match(scope, &query) else { return Vec::new() };
                        return match scope {
                            crate::app::SearchScope::Apps => self.act(Action::DataAppOpen(first), now),
                            _ => self.act(Action::ModuleOpen(first), now),
                        };
                    }
                },
                // Return starts the download, or stops the one under way.
                Screen::Fetch if self.fetching() => return self.act(Action::FetchStop, now),
                Screen::Fetch => return self.act(Action::FetchStart, now),
                Screen::Decrypt if self.state.code.is_empty() => self.say(l.pick(["请输入锁屏密码", "請輸入螢幕鎖定密碼", "Enter the screen lock"]), now),
                Screen::Decrypt => {
                    let code = std::mem::take(&mut self.state.code);
                    return self.unlock(code);
                }
                // A name the folder cannot take is turned down here, in
                // the user's words; an unchanged one leaves the page.
                Screen::Rename => {
                    let name = self.state.new_name.clone();
                    let current = &self.state.file.as_ref().expect("the file page shows an entry").name;
                    let taken = self.state.files.entries.as_ref().is_ok_and(|entries| entries.iter().any(|e| e.name == name));
                    match name.as_str() {
                        "" => self.say(l.pick(["请输入名称", "請輸入名稱", "Enter a name"]), now),
                        "." | ".." => self.say(l.pick(["名称不能为\".\"或\"..\"", "名稱不能是「.」或「..」", "A name can't be “.” or “..”"]), now),
                        _ if name.contains('/') => self.say(l.pick(["名称不能包含\"/\"", "名稱不能包含「/」", "A name can't hold “/”"]), now),
                        _ if name == *current => return self.back(now),
                        _ if taken => self.say_taken(now),
                        _ => return vec![Effect::Rename { path: self.entry_path(), name }],
                    }
                }
                // A password the network's protection cannot take is turned
                // down here; one it can goes, and the WLAN page shows how
                // joining goes.
                Screen::WlanPassword => {
                    let network = self.state.joining.clone().expect("the password page is for a network");
                    match refused(l, network.security, &self.state.password) {
                        Some(why) => self.say(why, now),
                        None => {
                            let password = Secret(std::mem::take(&mut self.state.password));
                            let mut fx = self.back(now);
                            fx.push(Effect::JoinWlan { ssid: network.ssid, name: network.name, security: network.security, password });
                            return fx;
                        }
                    }
                }
                // So is a name no network can have; the keyboard then types
                // into the field turned down.
                Screen::WlanAdd => {
                    let a = self.state.adding.as_mut().expect("the page adds a network");
                    let (field, why) = match a.name.len() {
                        0 => (Field::Name, Some(l.pick(["请输入网络名称", "請輸入網路名稱", "Enter the network's name"]).to_string())),
                        n if n > SSID_MAX => (
                            Field::Name,
                            Some(tr!(l, "网络名称不能超过 {SSID_MAX} 个字符", "網路名稱不能超過 {SSID_MAX} 個字元", "A network's name can't be over {SSID_MAX} characters")),
                        ),
                        _ if a.security.password() => (Field::Password, refused(l, a.security, &self.state.password).map(str::to_string)),
                        _ => (Field::Name, None),
                    };
                    match why {
                        Some(why) => {
                            if a.field != field {
                                a.field = field;
                                self.refresh();
                            }
                            self.say(&why, now)
                        }
                        None => {
                            let (name, security) = (a.name.clone(), a.security);
                            let password = match security.password() {
                                true => Secret(std::mem::take(&mut self.state.password)),
                                false => Secret::default(),
                            };
                            let mut fx = self.back(now);
                            fx.push(Effect::JoinWlan { ssid: name.clone().into_bytes(), name, security, password });
                            return fx;
                        }
                    }
                }
                other => unreachable!("{other:?} shows no keyboard"),
            },
        }
        // What is typed shows above the keyboard, which stays as it was
        // while its keys do.
        if matches!(cap, Cap::Char(_) | Cap::Space | Cap::Backspace) && self.state.layer == layer {
            let t = self.theme();
            self.refresh_rows(0, (t.m.h as f32 - widgets::keyboard_height(&t)).floor() as u32);
        } else {
            self.refresh();
        }
        Vec::new()
    }

    /// What a swipe-to-confirm control slid to its end does.
    /// 在线那一版不比装着的旧吗：旧就拦下，一个字节都不写。
    fn root_would_downgrade(&self, magisk: bool) -> bool {
        let (installed, online) = if magisk {
            let installed = self.state.magisk.as_ref().and_then(|r| r.as_ref().ok()).and_then(|st| st.version.clone());
            (installed, self.state.latest_magisk.clone())
        } else {
            let installed = self.state.kernel_su.as_ref().and_then(|r| r.as_ref().ok()).and_then(|st| st.version.clone());
            (installed, self.state.latest_kernelsu.clone())
        };
        matches!(is_newer(installed.as_deref(), online.as_deref()), Some(false))
    }

    /// 手指把滑块从底滑回左端（没滑完）：对正在写的 Root 就是取消。
    pub(super) fn unconfirmed(&mut self, which: Confirm, _now: u64) -> Vec<Effect> {
        let running = |job: Option<&KernelSuJob>| job.is_some_and(|j| j.outcome.is_none());
        match which {
            Confirm::KernelSu if running(self.state.kernel_su_job.as_ref()) => vec![Effect::CancelRoot { magisk: false }],
            Confirm::Magisk if running(self.state.magisk_job.as_ref()) => vec![Effect::CancelRoot { magisk: true }],
            _ => Vec::new(),
        }
    }

    pub(super) fn confirmed(&mut self, which: Confirm, now: u64) -> Vec<Effect> {
        let (l, mut fx) = (self.state.lang, Vec::new());
        // Every slider taken across is answered with a click of the motor,
        // whatever it was asking: the finger is told the question was taken
        // without looking at the page it was taken on.
        if self.state.vibrate {
            fx.push(Effect::Haptic);
        }
        match which {
            Confirm::Avb => {
                if let Some((verity, verification)) = self.state.avb_pending.take() {
                    fx.push(Effect::AvbSet(verity, verification));
                }
            }
            // 把 KMI 写进当前槽：先确认 data 已开、没在写、已解锁、在线的不比
            // 装着的旧，才真的开始（只升不降，2026-11-26）。
            Confirm::KernelSu => {
                if self.state.data != Data::Open {
                    self.say(l.pick(["内部存储未打开，请先解密 data。", "內部儲存未開啟，請先解密 data。", "Internal storage is not open; decrypt data first."]), now);
                } else if self.state.kernel_su_job.as_ref().is_some_and(|j| j.outcome.is_none()) {
                    self.say(l.pick(["正在写入 KernelSU，请等待完成。", "正在寫入 KernelSU，請等待完成。", "Writing KernelSU; wait for it to finish."]), now);
                } else if self.state.kernel_su.as_ref().and_then(|r| r.as_ref().ok()).is_some_and(|st| !st.unlocked) {
                    self.say(l.pick(["已锁定", "已鎖定", "Locked"]), now);
                } else if self.root_would_downgrade(false) {
                    self.say(l.pick(["已是最新，不降级", "已是最新，不降級", "Already the newest; not downgraded"]), now);
                } else {
                    let kmi = self.kernel_su_kmi().map(|row| row.id.clone()).unwrap_or_default();
                    let module = self.state.kernel_su_ko.take();
                    self.kernel_su_started();
                    self.state.confirm = 1.0;
                    fx.push(Effect::InstallKernelSu { kmi, module });
                }
            }
            // 把 Magisk 写进当前槽：同一套前置检查。
            Confirm::Magisk => {
                if self.state.data != Data::Open {
                    self.say(l.pick(["内部存储未打开，请先解密 data。", "內部儲存未開啟，請先解密 data。", "Internal storage is not open; decrypt data first."]), now);
                } else if self.state.magisk_job.as_ref().is_some_and(|j| j.outcome.is_none()) {
                    self.say(l.pick(["正在写入 Magisk，请等待完成。", "正在寫入 Magisk，請等待完成。", "Writing Magisk; wait for it to finish."]), now);
                } else if self.state.magisk.as_ref().and_then(|r| r.as_ref().ok()).is_some_and(|st| !st.unlocked) {
                    self.say(l.pick(["已锁定", "已鎖定", "Locked"]), now);
                } else if self.root_would_downgrade(true) {
                    self.say(l.pick(["已是最新，不降级", "已是最新，不降級", "Already the newest; not downgraded"]), now);
                } else {
                    self.magisk_started();
                    self.state.confirm = 1.0;
                    fx.push(Effect::InstallMagisk);
                }
            }
            // 写回一份旧 vbmeta：没选中就什么都不做，选了就把路径交出去。
            Confirm::Vbmeta => {
                let chosen = self.state.vbmeta_choice.and_then(|k| {
                    self.state.vbmeta_kept.as_ref().and_then(|kept| kept.as_ref().ok()).and_then(|list| list.get(k))
                });
                if let Some(kept) = chosen {
                    fx.push(Effect::VbmetaPutBack(kept.path.clone()));
                }
            }
            // 写回一份镜像：没选中就什么都不做；写之前那一份会被整份核对。
            Confirm::ImageRestore => {
                if let Some(copy) = self.image_chosen() {
                    fx.push(Effect::RestoreImage {
                        place: self.state.backup_to,
                        file: copy.file.clone(),
                        partition: copy.partition.clone(),
                        sha256: copy.sha256.clone(),
                    });
                }
            }
            Confirm::ImageDelete => {
                if let Some(copy) = self.image_chosen() {
                    fx.push(Effect::DeleteImageBackup { place: self.state.backup_to, file: copy.file.clone() });
                }
            }
            Confirm::Reboot => fx.push(Effect::Reboot(TARGETS[self.state.reboot])),
            Confirm::Shutdown => fx.push(Effect::Reboot(Target::PowerOff)),
            // The slider taken across: the lock is kept from now on. The risk
            // is asked again the next time it is turned on.
            Confirm::KeepLock => {
                self.state.asking_remember = false;
                self.state.remember = true;
                self.refresh();
                fx.push(Effect::Remember(true));
            }
            Confirm::Wipe => {
                let chosen: Vec<Wipe> = WIPES.iter().zip(self.state.wipe).filter(|(_, on)| *on).map(|(&w, _)| w).collect();
                match chosen.is_empty() {
                    true => self.say(l.pick(["请选择要清除的内容", "請選擇要清除的內容", "Choose what to wipe"]), now),
                    false => fx.push(Effect::Wipe(chosen)),
                }
            }
            Confirm::Install | Confirm::InstallWipe => {
                if let Install::Confirming { version, .. } = &self.state.install {
                    self.state.install = Install::Updating { version: version.clone(), stage: None, overwriting: false };
                    fx.push(Effect::Install);
                    self.refresh();
                }
            }
            Confirm::RunScript => {
                if self.state.install == Install::ConfirmingScript {
                    self.state.install = Install::Scripting { stage: None, running: false };
                    fx.push(Effect::Install);
                    self.refresh();
                }
            }
            Confirm::Backup => {
                let s = &self.state;
                let refused = if !s.backup.contains(&true) {
                    Some(["请选择要备份的内容", "請選擇要備份的內容", "Choose what to back up"])
                } else if s.backup[2] && s.data != Data::Open {
                    Some(["备份 data 分区需先完成「解密」", "備份 data 分割區需先完成「解密」", "Decrypt before backing up the data partition"])
                } else if s.place_open(s.backup_to) {
                    None
                } else {
                    Some(match (s.backup_to, &s.usb_drive) {
                        (Place::Storage, _) if s.data == Data::Erased => DATA_ERASED,
                        (Place::Storage, _) => DECRYPT_FIRST,
                        (Place::UsbDrive, Some(UsbDrive::Unreadable(_))) => USB_DRIVE_UNREADABLE,
                        (Place::UsbDrive, _) => USB_DRIVE_WANTED,
                    })
                };
                let backup = Effect::Backup { parts: s.backup, place: s.backup_to, compress: s.compress, verify: s.verify };
                match refused {
                    Some(why) => self.say(l.pick(why), now),
                    None => fx.push(backup),
                }
            }
            Confirm::Restore => {
                let (entry, parts) = self.state.restore.as_ref().expect("the restore page shows a backup");
                let restore = Effect::Restore { place: entry.place, name: entry.name.clone(), parts: *parts };
                if !parts.contains(&true) {
                    self.say(l.pick(["请选择要恢复的内容", "請選擇要還原的內容", "Choose what to restore"]), now);
                } else if parts[2] && self.state.data != Data::Open {
                    self.say(l.pick(["恢复 data 分区需先完成「解密」", "還原 data 分割區需先完成「解密」", "Decrypt before restoring the data partition"]), now);
                } else {
                    fx.push(restore);
                }
            }
            Confirm::DeleteBackup => {
                let (entry, _) = self.state.restore.as_ref().expect("the restore page shows a backup");
                fx.push(Effect::DeleteBackup { place: entry.place, name: entry.name.clone() });
            }
            Confirm::Slot => match (&self.state.boot_slots, self.state.next_slot) {
                (Some(Ok(slots)), Some(pick)) if pick != slots.active => fx.push(Effect::SetSlot(pick)),
                (Some(Ok(slots)), _) => {
                    let slot = slots.active.letter();
                    let said = tr!(l, "下次已从槽位 {slot} 启动", "下次已從槽位 {slot} 啟動", "Slot {slot} already starts next");
                    self.say(&said, now);
                }
                _ => self.say(l.pick(["无法读取启动槽位", "無法讀取啟動槽位", "Couldn't read the boot slots"]), now),
            },
            Confirm::FactoryReset => fx.push(Effect::FactoryReset),
            Confirm::Merge => {
                if matches!(self.state.update, Standing::Waiting | Standing::Failed(_)) {
                    self.state.update = Standing::Merging(0.0);
                    self.refresh();
                    fx.push(Effect::Merge);
                }
            }
            // An update that would not merge is given up here, so that the
            // phone starts from its own partitions rather than coming back
            // to the recovery at every restart (2026-11-20).
            Confirm::GiveUp => {
                if matches!(self.state.update, Standing::Failed(_)) {
                    fx.push(Effect::GiveUp);
                }
            }
            Confirm::DeleteApp => {
                if let Some(k) = self.state.app_confirm.take()
                    && let Some(Ok(list)) = &self.state.apps
                        && let Some(a) = list.get(k) {
                            fx.push(Effect::DataAppRemove(a.package.clone(), a.dir.clone()));
                        }
                self.refresh();
            }
            Confirm::DeleteModule => {
                if let Some(k) = self.state.module_confirm.take()
                    && let Some(Ok(list)) = &self.state.modules
                        && let Some(m) = list.get(k) {
                            fx.push(Effect::ModuleRemove(m.id.clone(), !m.removing));
                        }
                self.refresh();
            }
            Confirm::DeleteEntry => match self.state.asking_many.take() {
                Some(paths) => {
                    self.state.choosing = false;
                    self.state.chosen.clear();
                    self.refresh();
                    fx.push(Effect::DeleteMany(paths));
                }
                None => fx.push(Effect::Delete(self.entry_path())),
            },
            // The partition has to be open; whether its users have unlocked
            // their own files makes no difference.
            Confirm::Relabel => match self.state.data {
                Data::Open | Data::Locked { .. } => fx.push(Effect::Relabel),
                Data::Opening => self.say(l.pick(DATA_OPENING), now),
                Data::Erased => self.say(l.pick(DATA_ERASED), now),
                Data::Closed | Data::Failed(_) => {
                    self.say(l.pick(["请先在「解密」中打开 data 分区", "請先在「解密」中開啟 data 分割區", "Open the data partition in Decrypt first"]), now)
                }
            },
            // The data partition is reached through its own key whether or
            // not it is open, but not while it is being opened or erased.
            Confirm::Filesystem => {
                let row = self.fs_row().expect("the page of a partition shows one that was read");
                let data = row.name == DATA_PARTITION;
                match (&self.state.fs_action, &self.state.data) {
                    (_, Data::Opening) if data => self.say(l.pick(DATA_OPENING), now),
                    (FsAction::Repair, Data::Erased) if data => self.say(l.pick(DATA_ERASED), now),
                    (action, _) => fx.push(Effect::Filesystem { name: row.name.clone(), action: action.clone() }),
                }
            }
            Confirm::Flash => {
                if let Install::Choosing(choice) = &self.state.install {
                    let partition = choice.partitions[choice.partition];
                    fx.push(Effect::Flash { partition, pick: choice.pick });
                    self.state.install = Install::Flashing { partition, stage: None };
                    self.refresh();
                }
            }
            Confirm::SwitchSystem => {
                if let (Some(Ok(systems)), Some(pick)) = (&self.state.systems, self.state.system_pick)
                    && pick != starting(systems)
                {
                    fx.push(Effect::SwitchSystem(pick));
                    self.state.systems_work = Some(SystemsWork::Switching(pick));
                    self.state.system_pick = None;
                    self.refresh();
                }
            }
            Confirm::RemoveSystem => {
                fx.push(Effect::RemoveSystem);
                self.state.systems_work = Some(SystemsWork::Removing);
                self.state.asking_delete = false;
                self.state.system_pick = None;
                self.refresh();
            }
            Confirm::InstallSystem => {
                if let Install::SystemChoosing(choice) = &self.state.install {
                    fx.push(Effect::InstallSystem { size: choice.sizes[choice.size] });
                    self.state.install = Install::Systeming { name: choice.name.clone(), stage: None };
                    self.refresh();
                }
            }
        }
        fx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::testing::*;

    #[test]
    fn search_finds_by_pinyin_typed_on_its_own_keyboard() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Search, &f, &i);
        type_text(&mut app, &f, &i, "bf");
        assert_eq!(app.state.search, "bf");
        let opens: Vec<Action> = app.frame(&f, &i).hits.iter().map(|h| h.action).filter(|a| matches!(a, Action::Open(_))).collect();
        assert_eq!(opens, [Action::Open(Screen::Backup)]);
        tap(&mut app, &f, &i, Action::Type(Cap::Backspace));
        assert_eq!(app.state.search, "b");
        tap(&mut app, &f, &i, Action::ClearSearch);
        assert_eq!(app.state.search, "");
        type_text(&mut app, &f, &i, "xpsj");
        tap(&mut app, &f, &i, Action::Type(Cap::Enter));
        assert_eq!(app.screen(), Screen::ScreenOff, "return opens the first result");
        app.key(Key::Back, 0);
        assert_eq!(app.screen(), Screen::Search);
    }

    /// 中文 26 键：键上显示大写，按下去是小写字母，攒成拼音串；切到英文就是
    /// 小写 26 键。
    #[test]
    fn chinese_types_letters_while_the_keys_show_capitals() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Search, &f, &i);
        if !app.state.pinyin.on {
            app.act(Action::Type(Cap::Language), 0);
        }
        app.state.layer = Layer::Shifted;
        tap(&mut app, &f, &i, Action::Type(Cap::Char('n')));
        assert_eq!(app.state.pinyin.composing, "n", "26 键的字母攒成拼音");
        assert_eq!(app.state.layer, Layer::Shifted, "中文模式的键一直是大写");
        assert_eq!(app.state.search, "", "还没选词，框里不落字");
        tap(&mut app, &f, &i, Action::Type(Cap::Language));
        assert!(!app.state.pinyin.on, "切到英文");
        assert_eq!(app.state.layer, Layer::Letters, "英文是小写 26 键");
    }

    /// 中/英 那一枚在哪一页都管用：密码页以前把它当没按，切不过去。
    #[test]
    fn the_language_key_answers_on_a_page_that_takes_no_chinese() {
        let (f, i) = (fonts(), info());
        let screen = Screen::Fetch; // 网址页：以前这一页把中/英当没按
        let mut app = opened(screen, &f, &i);
        let was_on = app.state.pinyin.on;
        tap(&mut app, &f, &i, Action::Type(Cap::Language));
        assert_ne!(app.state.pinyin.on, was_on, "中/英切得动");
        // 中文 26 键大写，英文 26 键小写（2026-11-24）。
        assert_eq!(app.state.layer, if app.state.pinyin.on { Layer::Shifted } else { Layer::Letters });
        tap(&mut app, &f, &i, Action::Type(Cap::Language));
        assert_eq!(app.state.pinyin.on, was_on, "再切就回来");
    }

    #[test]
    fn shift_makes_one_capital_and_the_layers_switch() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Search, &f, &i);
        tap(&mut app, &f, &i, Action::Type(Cap::Shift));
        type_text(&mut app, &f, &i, "ab");
        assert_eq!(app.state.search, "Ab");
        tap(&mut app, &f, &i, Action::Type(Cap::Show(Layer::Digits)));
        type_text(&mut app, &f, &i, "1");
        tap(&mut app, &f, &i, Action::Type(Cap::Show(Layer::Symbols)));
        type_text(&mut app, &f, &i, "|");
        tap(&mut app, &f, &i, Action::Type(Cap::Space));
        assert_eq!(app.state.search, "Ab1| ");
        tap(&mut app, &f, &i, Action::Type(Cap::Show(Layer::Letters)));
        assert_eq!(app.state.layer, Layer::Letters);
    }

    #[test]
    fn a_key_shows_held_down_while_touched() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Search, &f, &i);
        let (x, y) = spot(&mut app, &f, &i, Action::Type(Cap::Char('q')));
        app.touch(Touch::Down(x, y), &f, 0);
        assert_eq!(app.state.key_held(), Some(Cap::Char('q')));
        app.touch(Touch::Up, &f, 0);
        assert_eq!(app.state.held, None);
        assert_eq!(app.state.search, "q");
    }
}
