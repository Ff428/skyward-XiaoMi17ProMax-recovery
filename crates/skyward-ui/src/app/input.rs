//! Fingers and keys: what a touch on the glass and a key going down or
//! coming up do.

use super::*;

/// 候选条在哪儿：键盘顶再往上一截，只有中文正在打拼音时才有。
/// How many steps the strength slider has across its span: each one a finger
/// crosses while dragging it is a tap of the motor.
const VIBE_STEPS: f32 = 20.0;

/// The step of the strength slider a `vibration` share of its span stands on.
fn vibe_step(vibration: f32) -> i32 {
    (vibration.clamp(0.0, 1.0) * VIBE_STEPS).round() as i32
}

impl App {
    /// 这一页是不是一屏一页的列表：是的话上下滑就翻一页，返回该发哪个动作。
    fn page_action(&self, step: i32) -> Option<Action> {
        match self.screen() {
            Screen::DataApps if matches!(&self.state.apps, Some(Ok(_))) => Some(Action::AppPage(step)),
            Screen::SystemApps => Some(Action::SysPage(step)),
            Screen::Modules if matches!(&self.state.apps, Some(Ok(_))) => Some(Action::ModulePage(step)),
            _ => None,
        }
    }

    pub fn touch(&mut self, ev: Touch, f: &Fonts, now: u64) -> Vec<Effect> {
        // 复制/移动先问一句到哪儿去：两个格子吃掉这一下，点别处就收起。
        if self.state.place_ask.is_some() && self.display_on {
            if let Touch::Down(x, y) = ev {
                let hit = self.place_hits.iter().find(|(r, _)| r.contains(x, y)).map(|(_, a)| *a);
                self.state.place_ask = None;
                self.dirty = true;
                if let Some(action) = hit {
                    return self.act(action, now);
                }
            }
            return Vec::new();
        }
        // Two taps on the dark panel light it, as on the phone's own system; a
        // single one does nothing, so a stray touch in a pocket does not.
        if !self.display_on {
            if matches!(ev, Touch::Down(..)) {
                match self.tap_at {
                    Some(at) if now.saturating_sub(at) <= DOUBLE_TAP_MS => {
                        self.tap_at = None;
                        self.wake(now);
                    }
                    _ => self.tap_at = Some(now),
                }
            }
            return Vec::new();
        }
        // 排序页上拎着一条走：手指落到第几格，它就挪到第几格；松手就放下，
        // 摆法随即存进设置（2026-11-28）。拎着的时候不上滑、也不翻页。
        if let Drag::EntryDrag(drag) = self.drag {
            return match ev {
                Touch::Move(_, y) => {
                    // 触摸来得比屏幕快：一条手指能一秒来一两百个 Move，每个都
                    // 重排 + 整页重画就掉帧。12 ms 内只认第一个（2026-11-28）。
                    if self.drag_at.is_some_and(|at| now.saturating_sub(at) < DRAG_STEP_MS) {
                        return Vec::new();
                    }
                    self.drag_at = Some(now);
                    // 拖到屏幕上下那一条边上，页面自己跟着滚：不然排序页三屏
                    // 长，往上根本拖不到「单独放在首页」那一段（2026-11-28）。
                    let theme = self.theme();
                    let edge = theme.m.dp(90.0);
                    let step = theme.m.row_h / 2.0;
                    let floor = self.h as f32;
                    // 到了头就别再算：算出来没变还照样重画，就是白掉帧。
                    let was = self.top().1;
                    let at = if y < edge {
                        (was - step).max(0.0)
                    } else if y > floor - edge {
                        (was + step).min(self.max_scroll())
                    } else {
                        was
                    };
                    let scrolled = (at - was).abs() > 0.5;
                    if scrolled {
                        self.top_mut().1 = at;
                    }
                    let page_y = y + at;
                    // 拎着的那一条整行（图标、文字、热区）跟着手指走。
                    self.state.home_drag = Some((drag.entry, page_y - drag.row_h / 2.0));
                    // 手指跑到第一行上面（「单独放在首页」那一段）就是放进顶置。
                    let moved = if page_y < drag.first_y {
                        self.move_entry_to_top(drag.entry)
                    } else {
                        let span = drag.total.max(1) as f32;
                        let at = (((page_y - drag.first_y) / drag.row_h).floor()).clamp(0.0, span - 1.0) as usize;
                        self.move_entry_to_row(drag.entry, at)
                    };
                    if moved {
                        self.state.home_pick = None;
                        self.row_press = None;
                        self.row_art = None;
                        self.refresh();
                        // 震动别连着来：一格一次，80 ms 内不再震第二下。
                        if self.haptic_at.is_none_or(|at| now.saturating_sub(at) >= HAPTIC_STEP_MS) {
                            self.haptic_at = Some(now);
                            vec![Effect::Haptic]
                        } else {
                            Vec::new()
                        }
                    } else {
                        // 浮起那张卡挪了才要重画；手指在同一个像素里抖不算。
                        let moved_ghost = match self.state.home_drag {
                            Some((_, was)) => (was - (page_y - drag.row_h / 2.0)).abs() > 0.9,
                            None => true,
                        };
                        if moved_ghost || scrolled {
                            self.dirty = true;
                        }
                        Vec::new()
                    }
                }
                Touch::Up | Touch::Down(..) => {
                    self.drag = Drag::Idle;
                    self.drag_at = None;
                    self.haptic_at = None;
                    self.state.home_drag = None;
                    self.row_press = None;
                    self.row_art = None;
                    self.refresh();
                    Vec::new()
                }
            };
        }
        // The card asking to delete is over everything: a finger on its swipe
        // works that, and one anywhere on the card does nothing to the page
        // under it.
        if (self.state.asking_delete
            || self.state.asking_many.is_some()
            || self.state.asking_remember
            || self.state.app_confirm.is_some()
            || self.state.module_confirm.is_some())
            && let Some(track) = self.asking_track
            && let Some((card, which)) = self.asking
        {
            let (x, y) = match ev {
                Touch::Down(x, y) | Touch::Move(x, y) => (x, y),
                Touch::Up => (0.0, 0.0),
            };
            let _ = (x, y);
            match ev {
                Touch::Down(x, y) if track.contains(x, y) => {
                    let travel = (track.w - track.h).max(1.0);
                    let grab = (x - track.x).clamp(0.0, track.h);
                    self.state.confirm = ((x - grab - track.x) / travel).clamp(0.0, 1.0);
                    self.dirty = true;
                    self.drag = Drag::Confirm { which, track, grab };
                    return Vec::new();
                }
                Touch::Down(x, y) if card.contains(x, y) => {
                    // A press on the card that is not on the swipe does
                    // nothing: the card is a question, not a page.
                    self.drag = Drag::Held;
                    return Vec::new();
                }
                Touch::Move(..) if matches!(self.drag, Drag::Confirm { .. }) => {
                    if let Drag::Confirm { which: _, track, grab } = self.drag {
                        let travel = (track.w - track.h).max(1.0);
                        self.state.confirm = ((x - grab - track.x) / travel).clamp(0.0, 1.0);
                        self.dirty = true;
                    }
                    return Vec::new();
                }
                // A press anywhere else puts the card away, the page under
                // it untouched.
                Touch::Down(..) if self.state.asking_remember => {
                    self.state.asking_remember = false;
                    self.dirty = true;
                    return Vec::new();
                }
                // 应用、模块的卸载卡：点卡片外面就收起来，什么都不做。
                Touch::Down(..) if self.state.app_confirm.is_some() || self.state.module_confirm.is_some() => {
                    self.state.app_confirm = None;
                    self.state.module_confirm = None;
                    self.dirty = true;
                    return Vec::new();
                }
                Touch::Up if matches!(self.drag, Drag::Confirm { .. }) => {
                    let Drag::Confirm { which, .. } = self.drag else { unreachable!() };
                    let done = self.state.confirm > SLID;
                    self.state.confirm = 0.0;
                    self.drag = Drag::Idle;
                    self.dirty = true;
                    if done {
                        return self.confirmed(which, now);
                    }
                    // 滑回左端：正在写的 Root 就此收手。
                    return self.unconfirmed(which, now);
                }
                _ => {}
            }
        }
        // 录屏的计时胶囊浮在所有页面之上，手指也是它先拿：拖着它走就是挪位置，
        // 按一下红点就是暂停或接着录（2026-11-21）。
        if self.state.record.on
            && let Some(effects) = self.record_touch(ev, now)
        {
            return effects;
        }
        // The control centre takes every finger while it is out or being
        // pulled; the lock keeps it shut.
        let pulling = matches!(self.drag, Drag::Shade { .. });
        if self.lock.is_none() && (self.shade > 0.0 || pulling) {
            return self.shade_touch(ev, now);
        }
        // A finger that comes down near the top edge may be starting a pull,
        // but a tap there is the page's -- the back arrow and the first rows
        // are up there -- so the pull begins only once it has moved down.
        if self.lock.is_none() {
            match ev {
                Touch::Down(_, y) if y <= self.theme().m.dp(40.0) => self.shade_from = Some(y),
                Touch::Move(_, y) => {
                    if let Some(start) = self.shade_from
                        && y - start > self.theme().m.dp(12.0)
                    {
                        self.shade_from = None;
                        self.drag = Drag::Shade { start, from: 0.0 };
                        return self.shade_touch(Touch::Move(0.0, y), now);
                    }
                }
                Touch::Up => self.shade_from = None,
                _ => {}
            }
        }
        // While the lock shows, only its knob answers: nothing a touch meant
        // for the page behind it reaches it.
        if self.lock.is_some() {
            let track = self.lock_rect.unwrap_or_else(|| self.lock_track());
            match ev {
                Touch::Down(x, y) => {
                    let on_knob = x >= track.x && x <= track.x + track.w && y >= track.y && y <= track.y + track.h;
                    if let Some(lock) = &mut self.lock {
                        lock.held = on_knob;
                    }
                }
                Touch::Move(x, _) => {
                    let span = (track.w - track.h).max(1.0);
                    if let Some(lock) = &mut self.lock
                        && lock.held
                    {
                        lock.drag = ((x - (track.x + track.h / 2.0)) / span).clamp(0.0, 1.0);
                        self.dirty = true;
                    }
                }
                Touch::Up => {
                    let unlocked = self.lock.as_ref().is_some_and(|l| l.held && l.drag >= UNLOCK_AT);
                    if let Some(lock) = &mut self.lock {
                        lock.held = false;
                        if !unlocked {
                            lock.drag = 0.0;
                        }
                    }
                    if unlocked {
                        self.lock = None;
                        self.refresh();
                        // The lock gave way: the same click a slider gives.
                        if self.state.vibrate {
                            return vec![Effect::Haptic];
                        }
                    } else {
                        self.dirty = true;
                    }
                }
            }
            return Vec::new();
        }
        // A finger on the charging screen sends it away for the page, and
        // does nothing else until it lifts.
        if self.charge_shown() {
            if matches!(ev, Touch::Down(..)) {
                self.dismiss_charging(now);
                self.drag = Drag::Waking;
            }
            return Vec::new();
        }
        // 唤醒之后立刻落下的那一下，先只当作「让屏醒着」；但手指要是接着滑，
        // 就不能把它整根吞掉（息屏很久之后直接长滑解锁，2026-11-23）。
        if let Some((wx, wy)) = self.waking {
            match ev {
                Touch::Move(x, y) => {
                    if (x - wx).hypot(y - wy) < WAKE_DRAG_DP {
                        return Vec::new();
                    }
                    self.waking = None;
                    // 这一根手指就是唤醒的那一下：别再按「刚醒」处理，
                    // 把它的起点补成一条按下，后面的滑动照常走。
                    self.woke_at = None;
                    let mut fx = self.touch(Touch::Down(wx, wy), f, now);
                    fx.extend(self.touch(ev, f, now));
                    return fx;
                }
                Touch::Up => self.waking = None,
                _ => {}
            }
        }
        // A finger landing just after the panel lit only keeps it awake.
        if matches!(ev, Touch::Down(..)) && self.just_woke(now) {
            if let Touch::Down(x, y) = ev {
                self.waking = Some((x, y));
            }
            self.stir(now);
            self.drag = Drag::Waking;
            return Vec::new();
        }
        // A finger on the glass puts the keys' light out, whatever a key
        // held down was doing comes to nothing, and a page sliding comes to
        // rest.
        if matches!(ev, Touch::Down(..)) {
            if self.keys {
                self.keys = false;
                self.adjusting = false;
                self.dirty = true;
            }
            self.drop_button();
            self.end_turn();
        }
        self.stir(now);
        let t = self.theme();
        let m = t.m;
        let (slop, edge) = (m.dp(8.0), m.dp(20.0));
        let scroll = self.top().1;
        let mut fx = Vec::new();
        let pressing = self.pressing();
        // 候选项那条：手指落在上面不是点它，而是可以左右滑着换页；没滑才算
        // 点了正对的那个。位置不另算 —— 就用这一帧画出来的那几格的命中框，
        // 免得「画在那儿、点不中」（2026-11-24 终端里点候选没反应就是位置各算
        // 各的）。
        // 候选项那条：手指落在上面不是点它，而是可以左右滑着换页；没滑才算
        // 点了正对的那个。这一条的位置按「键盘顶上方那一段」认，不另算框 ——
        // 以前绘制与命中的 top 各算各的，真机上点候选怎么都点不中
        // （2026-11-24）。
        if let Touch::Down(x, y) = ev
            && self.state.pinyin.on
            && !self.state.pinyin.composing.is_empty()
            && y < m.h as f32 - widgets::keyboard_height(&t) + m.dp(4.0)
            && y > m.h as f32 - widgets::keyboard_height(&t) - m.dp(130.0)
        {
            let pad = m.dp(6.0);
            let per = crate::pinyin::PER_PAGE;
            let cell = (m.w as f32 - pad * 2.0) / per as f32;
            let k = ((x - pad) / cell.max(1.0)).floor();
            let tap = (k >= 0.0 && (k as usize) < self.state.pinyin_page().len()).then_some(k as usize);
            self.drag = Drag::Candidates { x0: x, step: 0, tap };
            return fx;
        }
        match ev {
            Touch::Down(x, y) => {
                self.tracker.clear();
                self.tracker.add(now, y);
                let side = if x < edge {
                    Some(true)
                } else if x > m.w as f32 - edge {
                    Some(false)
                } else {
                    None
                };
                // The first page has nothing to go back to: its edges are
                // like the rest of it.
                let at_edge = side.filter(|_| self.stack.len() > 1);
                // A swipe in from an edge goes back over whatever the page is
                // doing: a finger landing there leaves a gliding or springing
                // page alone until it drags the page, and presses nothing.
                if at_edge.is_some() && self.in_motion(now) {
                    self.drag = Drag::Press { x, y, action: None, edge: at_edge, at: now };
                    return fx;
                }
                // A terminal's screen or the editor's text gliding stops
                // under a finger landing on it, which holds it to scroll on.
                let glided = self.glide.take().filter(|(glide, ..)| !glide.at(now).1);
                let locked = self.frame.as_ref().is_some_and(|fr| fr.no_scroll);
                let history = self.frame.as_ref().and_then(|fr| fr.scrollback).filter(|(r, _)| !locked && r.contains(x, y + scroll));
                if let (Some(_), Some((_, pitch))) = (glided, history) {
                    self.drag = Drag::History { last: y, carry: 0.0, pitch };
                    return fx;
                }
                // Anywhere else it stops such a page and holds it; the
                // control under the finger is not pressed.
                if self.catch(now) {
                    self.drag = if locked { Drag::Held } else { Drag::Scroll { last: y } };
                    return fx;
                }
                // The `<` stays put while the page scrolls; under the bar
                // only it answers. A pinned strip answers where it shows,
                // over the rows beneath it, and so does a docked one.
                let back = widgets::back_rect(&t);
                let strip = self.frame.as_ref().and_then(|fr| fr.pinned.as_ref()).map(|p| (p, pinned_top(&t, p, scroll)));
                let on_strip = strip.filter(|&(p, top)| y >= top && y < top + p.canvas.h as f32);
                let dock = self.frame.as_ref().and_then(|fr| fr.dock.as_ref()).map(|d| (d, self.h as f32 - d.canvas.h as f32));
                let on_dock = dock.filter(|&(_, top)| y >= top);
                let button = self.frame.as_ref().and_then(|fr| fr.button).map(|(_, a)| (a, widgets::button_rect(&t)));
                // 标题旁那几枚圆钮（文件管理的排序、多选、U 盘、搜索）：命中框由
                // 绘制那一帧给，放在 `<` 之后、圆钮之前（2026-11-24）。
                let bar_chip = self
                    .bar_chip_rects
                    .iter()
                    .find(|(r, _)| r.contains(x, y))
                    .map(|(r, a)| (*a, *r));
                // 记下落在翻页键上的点击坐标，便于核对点的是哪一个键。
                let hit = if self.frame.as_ref().is_some_and(|fr| fr.back) && back.contains(x, y) {
                    Some((Action::Back, back))
                } else if bar_chip.is_some() {
                    bar_chip
                } else if let Some(button) = button.filter(|(_, r)| r.contains(x, y)) {
                    Some(button)
                } else if let Some((d, top)) = on_dock {
                    d.hits.iter().find(|h| h.rect.contains(x, y - top)).map(|h| (h.action, Rect { y: h.rect.y + top, ..h.rect }))
                } else if let Some((p, top)) = on_strip {
                    p.hits.iter().find(|h| h.rect.contains(x, y - top)).map(|h| (h.action, Rect { y: h.rect.y + top, ..h.rect }))
                } else if self.bar.on && y < widgets::bar_bottom(&t) {
                    None
                } else {
                    // A search bar folding away answers only on what is
                    // left of it.
                    let fr = self.frame.as_ref();
                    fr.and_then(|fr| fr.hit(x, y + scroll).filter(|h| fr.shown(h.rect, scroll).is_some_and(|r| r.contains(x, y))))
                        .map(|h| (h.action, h.rect))
                };
                let action = hit.map(|h| h.0);
                // 翻页键上的点击：把坐标记下来，日志里能看出点的是哪一个。
                if let Some(a) = action {
                    match a {
                        Action::AppPage(by) => self.state.page_tap = Some((x, y, by)),
                        Action::ModulePage(by) => self.state.page_tap = Some((x, y, by)),
                        _ => {}
                    }
                } else if matches!(self.screen(), Screen::DataApps | Screen::Modules)
                    && y >= t.m.title_top - t.m.dp(20.0)
                    && y <= t.m.title_top + t.m.title_px + t.m.dp(20.0)
                {
                    // 点在翻页那条带上却什么也没命中：多半是那一刻按键还没画出来。
                    self.state.page_miss = Some((x, y));
                    // 右半边当「下一页」，左半边当「上一页」：等键出来再补这一下。
                    self.state.page_pending = Some(if x > self.w as f32 * 0.5 { 1 } else { -1 });
                }
                // A row of the page darkens behind its words a moment after
                // the finger lands on it, if the finger stays.
                let row = hit
                    .filter(|&(a, r)| self.frame.as_ref().is_some_and(|fr| fr.hits.contains(&Hit { rect: r, action: a })))
                    .filter(|&(a, r)| row_wide(&t, r) && !matches!(a, Action::Open(Screen::About) | Action::Confirm(_) | Action::TypeInto(_)))
                    .map(|(_, r)| r);
                self.drag = match hit {
                    Some((Action::Slider(which), track)) => {
                        self.set_slider(&t, which, track, x);
                        // The step the finger lands on is silent; the ones it
                        // is dragged across are the taps.
                        if which == Slider::Vibration {
                            self.vibe_step = vibe_step(self.state.vibration);
                        }
                        Drag::Slider(which, track)
                    }
                    Some((Action::Confirm(which), track)) if self.on_knob(track, x) => {
                        let knob_x = track.x + (track.w - track.h) * self.state.confirm;
                        Drag::Confirm { which, track, grab: x - knob_x }
                    }
                    Some((Action::Pattern, _)) => {
                        self.state.pattern.clear();
                        self.trace(&t, x, y + scroll);
                        Drag::Pattern
                    }
                    _ => {
                        if let Some(rect) = row {
                            self.row_press = Some(RowPress { rect, rise: now + ROW_PRESS_DELAY_MS, fall: None });
                            self.row_art = None;
                        }
                        Drag::Press { x, y, action, edge: at_edge, at: now }
                    }
                };
            }
            Touch::Move(x, y) => match std::mem::replace(&mut self.drag, Drag::Idle) {
                // 拎着一条走的时候不会走到这儿（上面先接住了）；放回去。
                drag @ Drag::EntryDrag(_) => {
                    self.drag = drag;
                }
                Drag::Press { x: x0, y: y0, action, edge: at_edge, at } => {
                    self.tracker.add(now, y);
                    let (dx, dy) = (x - x0, y - y0);
                    self.drag = match at_edge {
                        Some(from_left) if dx.abs() > slop && dx.abs() > dy.abs() => {
                            self.dirty = true;
                            Drag::Edge { start: x0, from_left, armed: false, x, cy: y0 - m.dp(CUE_LIFT_DP) }
                        }
                        // 应用管理、系统应用、模块管理都是一屏一页：手指**往上滑是
                        // 下一页**，往下滑回到上一页（2026-11-23 用户按录屏纠正）。
                        _ if dy.abs() > slop && dy.abs() > dx.abs() && self.page_action(if dy < 0.0 { 1 } else { -1 }).is_some() => {
                            let action = self.page_action(if dy < 0.0 { 1 } else { -1 }).expect("just asked");
                            fx.extend(self.act(action, now));
                            Drag::Pager
                        }
                        // 这一页带着键盘：上下滑不滚页面，手指只打字（2026-11-24）。
                        _ if dy.abs() > slop && self.frame.as_ref().is_some_and(|fr| fr.no_scroll) => Drag::Held,
                        _ if dy.abs() > slop => {
                            let history = self.frame.as_ref().and_then(|fr| fr.scrollback).filter(|(r, _)| r.contains(x0, y0 + scroll));
                            if let Some((_, pitch)) = history {
                                self.scroll_history(y0, y, 0.0, pitch)
                            } else {
                                // A page still gliding or springing back is
                                // caught where it has got to; one at rest
                                // follows the finger from where it came down.
                                if !self.catch(now) {
                                    self.drag_by(y0 - y);
                                }
                                Drag::Scroll { last: y }
                            }
                        }
                        _ => Drag::Press { x: x0, y: y0, action, edge: at_edge, at },
                    };
                    if !matches!(self.drag, Drag::Press { .. }) {
                        self.let_row_go(now);
                    }
                }
                Drag::Scroll { last } => {
                    self.tracker.add(now, y);
                    self.drag_by(last - y);
                    self.drag = Drag::Scroll { last: y };
                }
                Drag::Edge { start, from_left, armed, cy, .. } => {
                    let travel = if from_left { x - start } else { start - x };
                    let ready = travel > m.dp(BACK_SWIPE_DP);
                    // A tap of the motor as the `<` appears, as the system
                    // gives.
                    if ready && !armed && self.state.vibrate {
                        fx.push(Effect::Haptic);
                    }
                    self.drag = Drag::Edge { start, from_left, armed: ready, x, cy };
                    self.dirty = true;
                }
                Drag::Candidates { x0, step, tap } => {
                    // 每挪过一条候选的宽就翻一页：向左滑是下一页。
                    let gap = m.dp(70.0).max(1.0);
                    let walked = ((x - x0) / gap).floor() as i32;
                    if walked == step {
                        let dragged = (x - x0).abs() > m.dp(8.0);
                        self.drag = Drag::Candidates { x0, step, tap: if dragged { None } else { tap } };
                    } else {
                        let step_now = walked;
                        let dir: i8 = if step_now < step { 1 } else { -1 };
                        for _ in 0..(step_now - step).abs() {
                            fx.extend(self.act(Action::Type(widgets::Cap::Page(dir)), now));
                        }
                        self.drag = Drag::Candidates { x0, step: step_now, tap: None };
                    }
                }
                Drag::Slider(which, track) => {
                    self.set_slider(&t, which, track, x);
                    // The strength is felt while it is dragged, not only once
                    // the finger is lifted: each step the slider crosses taps
                    // the motor at the strength it has just been set to, so
                    // that the lightest and the strongest are told apart by
                    // ear before anything is left with (2026-11-19).
                    if which == Slider::Vibration && self.state.vibrate {
                        let step = vibe_step(self.state.vibration);
                        if step != self.vibe_step {
                            self.vibe_step = step;
                            fx.push(Effect::Haptic);
                        }
                    }
                    self.drag = Drag::Slider(which, track);
                }
                Drag::Confirm { which, track, grab } => {
                    let travel = (track.w - track.h).max(1.0);
                    self.state.confirm = ((x - grab - track.x) / travel).clamp(0.0, 1.0);
                    self.repaint_confirm(&t, f, which, track);
                    self.drag = Drag::Confirm { which, track, grab };
                }
                Drag::Pattern => {
                    self.trace(&t, x, y + scroll);
                    self.drag = Drag::Pattern;
                }
                Drag::History { last, carry, pitch } => {
                    self.tracker.add(now, y);
                    self.drag = self.scroll_history(last, y, carry, pitch);
                }
                // 胶囊自己收着这一下（`record_touch` 先看过），落到这里只把
                // 状态照原样放回去。
                Drag::RecordPill { grab, moved, dot, stop } => self.drag = Drag::RecordPill { grab, moved, dot, stop },
                // The sheet is pulled by its own handler, which sees every
                // finger while it is out.
                Drag::Shade { start, from } => self.drag = Drag::Shade { start, from },
                Drag::Waking => self.drag = Drag::Waking,
                Drag::Held => self.drag = Drag::Held,
                // 已经翻过一页：这根手指剩下的移动不再管。
                Drag::Pager => self.drag = Drag::Pager,
                // 拎着的那一枚在 `Touch::Move` 开头就走了，落不到这里。
                Drag::TileDrag(_) => {}
                Drag::Idle => {}
            },
            Touch::Up => match std::mem::replace(&mut self.drag, Drag::Idle) {
                Drag::Press { action: Some(a), .. } => {
                    if let Some(p) = &mut self.row_press {
                        p.lift(now);
                    }
                    let mut done = self.act(a, now);
                    // Every control taken gives a tap of the motor, as the
                    // phone's own system does: the finger is told it landed on
                    // something, whatever the control then does with it. One
                    // that clicks for itself is not clicked twice (2026-11-19).
                    if self.state.vibrate && !done.iter().any(|e| matches!(e, Effect::Haptic)) {
                        done.insert(0, Effect::Haptic);
                    }
                    fx.extend(done);
                }
                Drag::Press { x, y, action: None, .. } if self.screen() == Screen::Editor => self.place_caret(f, x, y + scroll),
                Drag::History { pitch, .. } => {
                    // A finger lifted while still moving sends the rows
                    // gliding on.
                    let speed = self.tracker.velocity(now);
                    if speed.abs() >= m.dp(MIN_FLING_DP) {
                        let speed = speed.clamp(-m.dp(MAX_FLING_DP), m.dp(MAX_FLING_DP));
                        self.glide = Some((Fling::new(now, 0.0, speed, self.px_per_dp * 160.0), pitch, 0));
                    }
                }
                Drag::Scroll { .. } if self.pull != 0.0 => {
                    // A page let go past its end springs back.
                    self.bounce = Some(Spring::new(now, motion::rubber(self.pull, self.h as f32), 0.0, 0.4));
                    self.pull = 0.0;
                    self.dirty = true;
                }
                Drag::Scroll { .. } => {
                    // A finger lifted while still moving sends the page
                    // gliding on. One lifted slower leaves it where it is,
                    // but for a search bar part folded, which settles
                    // folded or unfolded; so does one lifted in the fold
                    // whose glide would end in it too.
                    let speed = -self.tracker.velocity(now);
                    let folding = |s: f32| self.frame.as_ref().is_some_and(|fr| fr.settle(s) != s);
                    let glide = (speed.abs() >= m.dp(MIN_FLING_DP)).then(|| {
                        let speed = speed.clamp(-m.dp(MAX_FLING_DP), m.dp(MAX_FLING_DP));
                        Fling::new(now, scroll, speed, self.px_per_dp * 160.0)
                    });
                    match glide {
                        Some(glide) if !(folding(scroll) && folding(glide.at(u64::MAX).0)) => self.fling = Some(glide),
                        _ => self.settle_fold(now),
                    }
                }
                Drag::Edge { armed: true, .. } => fx.extend(self.back(now)),
                Drag::Edge { start, from_left, x, cy, .. } => {
                    // Short of going back, the cue sinks back into the edge.
                    let travel = if from_left { x - start } else { start - x };
                    let depth = widgets::back_cue_depth(&t, travel);
                    self.cue = Some((from_left, cy, Spring::new(now, depth, 0.0, 0.15)));
                    self.dirty = true;
                }
                Drag::Confirm { which, track, .. } => {
                    let done = self.state.confirm > SLID;
                    self.state.confirm = 0.0;
                    self.repaint_confirm(&t, f, which, track);
                    if done {
                        fx.extend(self.confirmed(which, now));
                    } else {
                        fx.extend(self.unconfirmed(which, now));
                    }
                }
                Drag::Pattern => fx.extend(self.drawn(now)),
                // 候选条上松手：没滑过就是点了那一个候选，字落进框里。
                Drag::Candidates { tap: Some(k), .. } => {
                    let mut done = self.act(Action::Type(widgets::Cap::Pick(k)), now);
                    if self.state.vibrate && !done.iter().any(|e| matches!(e, Effect::Haptic)) {
                        done.insert(0, Effect::Haptic);
                    }
                    fx.extend(done);
                }
                Drag::Candidates { .. } => {}
                // Let go, the strength slider taps the motor as strongly as
                // it was left at.
                Drag::Slider(Slider::Vibration, _) if self.state.vibrate => fx.push(Effect::Haptic),
                _ => {}
            },
        }
        if self.pressing() != pressing {
            self.dirty = true;
        }
        // A key of the on-screen keyboard, or a choice of a list open over
        // the page, shows held down while the finger rests on it.
        let held = match self.drag {
            Drag::Press { action: Some(a @ (Action::Type(_) | Action::PickSecurity(_))), .. } => Some(a),
            _ => None,
        };
        if held != self.state.held {
            let was = std::mem::replace(&mut self.state.held, held);
            // Only the rows of what is let go and what is held change.
            let rects: Option<Vec<Rect>> = self
                .frame
                .as_ref()
                .and_then(|fr| [was, held].into_iter().flatten().map(|a| fr.hits.iter().find(|h| h.action == a).map(|h| h.rect)).collect());
            match rects {
                Some(rects) => rects.into_iter().for_each(|r| self.refresh_rows(r.y.floor() as u32, (r.y + r.h).ceil() as u32)),
                None => self.refresh(),
            }
        }
        fx
    }

    /// The button a finger holds down, which shows held at once: `<`, the
    /// search page's round button, or the home page's search bar.
    pub(super) fn pressing(&self) -> Option<Action> {
        let button = self.frame.as_ref().and_then(|fr| fr.button).map(|(_, a)| a);
        match self.drag {
            Drag::Press { action: Some(a @ (Action::Back | Action::CloseSearch | Action::Open(Screen::About))), .. } => Some(a),
            Drag::Press { action: Some(a), .. } if Some(a) == button => Some(a),
            _ => None,
        }
    }

    /// The finger left the row it held, scrolling the page or swiping: a
    /// row not darkening yet never does, and one darkening fades back.
    fn let_row_go(&mut self, now: u64) {
        let Some(p) = self.row_press else { return };
        if now < p.rise {
            self.row_press = None;
            self.row_art = None;
        } else if p.fall.is_none() {
            self.row_press = Some(RowPress { fall: Some(now), ..p });
        }
        self.dirty = true;
    }

    /// No touchscreen answers: the keys work the pages, their light shown
    /// from now on.
    pub fn touchscreen_missing(&mut self) {
        self.keys = true;
        self.dirty = true;
    }

    /// A panel came up after the keys had been said to work the page instead.
    pub fn set_touchscreen(&mut self, present: bool) {
        self.keys = !present;
        self.dirty = true;
    }

    /// A button went down. With the keys working the page, the volume keys
    /// move the light from control to control and the power key works the
    /// one lit as it comes back up; otherwise the first volume press lights
    /// a control, and the power key turns the panel off. The power key and
    /// volume down together take a screenshot instead: whichever of the two
    /// goes down first waits [`CHORD_MS`] for the other, or until it is let
    /// go, and then does what it does alone, as pressed when it went down.
    pub fn key(&mut self, key: Key, now: u64) -> Vec<Effect> {
        // 问「到哪儿去」的时候，返回键只说「算了」。
        if self.state.place_ask.is_some() {
            self.state.place_ask = None;
            self.dirty = true;
            return Vec::new();
        }
        // Any button stops a gliding page, and brings a sliding one to rest.
        self.fling = None;
        self.end_turn();
        if let Some((first, at)) = self.chord
            && matches!((first, key), (Key::Power, Key::VolumeDown) | (Key::VolumeDown, Key::Power))
            && now < at + CHORD_MS
        {
            return self.screenshot(key, now);
        }
        let mut fx = self.let_chord_go(now);
        if !self.display_on {
            // The power key lights the dark panel, and does only that, as on
            // the phone's own system; the other keys leave it dark.
            if key == Key::Power {
                self.wake(now);
            }
            self.button = Some((key, Press::Spent));
            return fx;
        }
        // A key other than power sends the charging screen away for the
        // page, and does nothing else.
        if key != Key::Power && self.charge_shown() {
            self.dismiss_charging(now);
            self.button = Some((key, Press::Spent));
            return fx;
        }
        // A button pressed just after the panel lit only keeps it awake, so
        // a power press meant to wake it does not turn it back off.
        if self.just_woke(now) {
            self.stir(now);
            self.button = Some((key, Press::Spent));
            return fx;
        }
        self.stir(now);
        if matches!(key, Key::Power | Key::VolumeDown) {
            self.chord = Some((key, now));
            return fx;
        }
        fx.extend(self.key_down(key, now, now));
        fx
    }

    /// The power key and volume down went down together: what the panel
    /// shows is saved as a picture, with a tap of the motor, and neither key
    /// does anything more until let go.
    fn screenshot(&mut self, key: Key, now: u64) -> Vec<Effect> {
        self.chord = None;
        self.stir(now);
        self.button = Some((key, Press::Spent));
        let mut fx = Vec::new();
        if self.state.vibrate {
            fx.push(Effect::Haptic);
        }
        fx.push(Effect::Screenshot { storage: self.state.place_open(Place::Storage) });
        fx
    }

    /// The key waiting for its partner in a screenshot does what it does
    /// alone `now`, as pressed when it went down.
    pub(super) fn let_chord_go(&mut self, now: u64) -> Vec<Effect> {
        match self.chord.take() {
            Some((key, at)) => self.key_down(key, at, now),
            None => Vec::new(),
        }
    }

    /// What a button that went down `at` does on its own, `now`: a key
    /// held is timed from when it went down, the screen goes off from now.
    fn key_down(&mut self, key: Key, at: u64, _now: u64) -> Vec<Effect> {
        match key {
            Key::Back => {
                self.button = Some((key, Press::Spent));
                self.act(Action::Back, at)
            }
            Key::VolumeUp | Key::VolumeDown => {
                let down = key == Key::VolumeDown;
                self.button = Some((key, Press::Volume { down, next: at + REPEAT_DELAY_MS }));
                self.volume(down, at)
            }
            // A press before the page it lands on is drawn is lost, rather
            // than taken for one on nothing.
            Key::Power if self.keys && self.frame.is_none() => {
                self.button = Some((key, Press::Spent));
                Vec::new()
            }
            Key::Power => {
                self.dirty = true;
                match self.keys.then(|| self.lit()).flatten() {
                    // Held on a swipe-to-confirm control, the key slides
                    // its knob along as a finger would.
                    Some(Spot { stop: Stop::Hit(Action::Confirm(which)), rect, .. }) => {
                        self.button = Some((key, Press::Slide { which, track: rect, at }))
                    }
                    Some(spot) => self.button = Some((key, Press::Power { spot, at })),
                    // 屏亮着、没按在控件上：先按住看着。够久就是重启菜单
                    // （`tick` 里到点开），早点松手才是息屏。
                    None => {
                        self.button = Some((key, Press::PowerHold { at }));
                    }
                }
                Vec::new()
            }
        }
    }

    /// A button came back up: the power key works the control it went down
    /// on, or confirms with the knob it slid far enough. Let go before its
    /// partner in a screenshot came, the key waiting for it first does what
    /// it does alone.
    /// A finger on the control centre, or one pulling it down from the top.
    fn shade_touch(&mut self, ev: Touch, now: u64) -> Vec<Effect> {
        let t = self.theme();
        match ev {
            Touch::Down(x, y) => {
                // A new finger starts the sheet afresh: nothing of the last
                // pull's inertia carries into it.
                self.shade_settle = None;
                self.shade_speed = 0.0;
                self.shade_last = None;
                // A finger anywhere the sheet does not reach puts it away:
                // the page under it was being touched, not the sheet.
                let tall = shade_height(&t, &self.state.tiles, self.shade_editing);
                let sheet = tall * self.shade;
                if self.shade > 0.0 && y > sheet {
                    self.close_shade();
                    self.drag = Drag::Idle;
                    return Vec::new();
                }
                if self.shade > 0.0
                    && let Some((rect, hit)) = self.shade_hits.iter().find(|(r, _)| r.contains(x, y)).copied()
                {
                    match hit {
                        ShadeHit::Brightness | ShadeHit::Vibration => {
                            let which = if hit == ShadeHit::Brightness { Slider::Brightness } else { Slider::Vibration };
                            self.shade_slider(which, rect, x);
                            // The step the finger lands on is silent; the ones
                            // it is dragged across tap the motor, so the
                            // strength is heard as it is set (2026-11-19).
                            if which == Slider::Vibration {
                                self.vibe_step = vibe_step(self.state.vibration);
                            }
                            self.drag = Drag::Slider(which, rect);
                        }
                        // 编辑态按住一枚磁贴就拎起来，跟着手指走（2026-11-22）。
                        ShadeHit::Tile(tile) if self.shade_editing => {
                            let drag = TileDrag { tile, x, y };
                            self.shade_drag = Some(drag);
                            self.drag = Drag::TileDrag(drag);
                            self.dirty = true;
                            return Vec::new();
                        }
                        _ => self.shade_press = Some((hit, y)),
                    }
                    return Vec::new();
                }
                self.drag = Drag::Shade { start: y, from: self.shade };
                Vec::new()
            }
            Touch::Move(x, y) => {
                // 拎着的那一枚磁贴跟着手指走（2026-11-22）。
                if let Drag::TileDrag(mut drag) = self.drag {
                    drag.x = x;
                    drag.y = y;
                    self.shade_drag = Some(drag);
                    self.drag = Drag::TileDrag(drag);
                    self.dirty = true;
                    return Vec::new();
                }
                if let Drag::Slider(which, track) = self.drag {
                    self.shade_slider(which, track, x);
                    let mut fx = Vec::new();
                    if which == Slider::Vibration && self.state.vibrate {
                        let step = vibe_step(self.state.vibration);
                        if step != self.vibe_step {
                            self.vibe_step = step;
                            fx.push(Effect::Haptic);
                        }
                    }
                    return fx;
                }
                // A finger that came down on a tile and then moved was pulling
                // the sheet: the tile is not pressed after all.
                if let Some((_, down)) = self.shade_press
                    && (y - down).abs() > t.m.dp(14.0)
                {
                    self.shade_press = None;
                    self.drag = Drag::Shade { start: down, from: self.shade };
                }
                if let Drag::Shade { start, from } = self.drag {
                    let pulled = from * SHADE_PULL_DP + (y - start);
                    let eased = if pulled > 0.0 { pulled * (1.0 - 0.22 * (pulled / SHADE_PULL_DP).min(1.0)) } else { pulled };
                    self.shade = (eased / SHADE_PULL_DP).clamp(0.0, 1.0);
                    if let Some((was_at, was_y)) = self.shade_last {
                        let dt = now.saturating_sub(was_at) as f32 / 1000.0;
                        if dt > 0.0005 {
                            self.shade_speed = self.shade_speed * 0.55 + ((y - was_y) / dt) * 0.45;
                        }
                    }
                    self.shade_last = Some((now, y));
                    self.shade_settle = None;
                    self.dirty = true;
                }
                Vec::new()
            }
            Touch::Up => {
                let mut fx = Vec::new();
                // 拎着的那一枚松手：落进手指指着的那一格，摆法随即存下来
                //（2026-11-22）。
                if let Drag::TileDrag(drag) = self.drag {
                    self.drag = Drag::Idle;
                    self.shade_drag = None;
                    let grid = TileGrid::of(&t, self.shade_top());
                    let at = grid.slot(drag.x, drag.y, self.state.tiles.len());
                    if let Some(from) = self.state.tiles.iter().position(|&t| t == drag.tile) {
                        self.state.tiles.remove(from);
                        self.state.tiles.insert(at.min(self.state.tiles.len()), drag.tile);
                        self.dirty = true;
                        fx.push(Effect::KeepTiles(self.state.tiles.clone()));
                        fx.push(Effect::Haptic);
                    }
                    return fx;
                }
                if let Some((hit, _)) = self.shade_press.take() {
                    match hit {
                        // 时间行最右端那支笔：进编辑态；编辑时同一个位置是勾。
                        ShadeHit::TilesEdit => {
                            self.shade_editing = true;
                            self.dirty = true;
                            fx.push(Effect::Haptic);
                        }
                        ShadeHit::TilesDone => {
                            self.shade_editing = false;
                            self.shade_drag = None;
                            self.dirty = true;
                        }
                        // 编辑时那枚小叉：把这枚拿下来。
                        ShadeHit::TileRemove(tile) => {
                            if self.state.tile_remove(tile) {
                                self.dirty = true;
                                fx.push(Effect::KeepTiles(self.state.tiles.clone()));
                                fx.push(Effect::Haptic);
                            }
                        }
                        // 候选区里的一枚：摆到最后一格。
                        ShadeHit::TileAdd(tile) => {
                            if self.state.tile_add(tile) {
                                self.dirty = true;
                                fx.push(Effect::KeepTiles(self.state.tiles.clone()));
                                fx.push(Effect::Haptic);
                            }
                        }
                        ShadeHit::Tile(tile) => match tile {
                            Tile::Wlan => {
                                let on = self.state.wlan.as_ref().is_some_and(|w| w.power == Power::On);
                                fx.push(Effect::Wlan(!on));
                            }
                            Tile::Screenshot => {
                                self.close_shade();
                                self.shot_when = Some(now + 130);
                            }
                            Tile::Record => {
                                self.close_shade();
                                fx.extend(self.act(Action::Record, now));
                            }
                            Tile::Vibrate => fx.extend(self.act(Action::Toggle(Switch::Vibrate), now)),
                            Tile::FileTransfer => fx.extend(self.act(Action::Toggle(Switch::FileTransfer), now)),
                            Tile::Language => {
                                // 三语轮转：简中 → 繁中 → English → 简中。
                                let next = match self.state.lang {
                                    Lang::Hans => Lang::Hant,
                                    Lang::Hant => Lang::En,
                                    Lang::En => Lang::Hans,
                                };
                                fx.extend(self.act(Action::Language(next), now));
                            }
                            other => {
                                if let Some(screen) = other.screen() {
                                    self.close_shade();
                                    fx.extend(self.act(Action::Open(screen), now));
                                }
                            }
                        },
                        ShadeHit::Brightness | ShadeHit::Vibration => {}
                    }
                    return fx;
                }
                if matches!(self.drag, Drag::Slider(..)) {
                    self.drag = Drag::Idle;
                }
                if matches!(self.drag, Drag::Shade { .. }) {
                    let dragged = self.shade_last.is_some();
                    self.drag = Drag::Idle;
                    // Let go, it carries on a little the way it was going, so a
                    // flick opens it and a slow drag settles where it got to.
                    let heading = match dragged {
                        true => self.shade + self.shade_speed / 1400.0,
                        false => self.shade,
                    };
                    let target = if heading >= SHADE_KEEPS { SHADE_PULL_DP } else { 0.0 };
                    let already = self.shade_settle.is_some_and(|(_, _, _, _, heading)| heading == target);
                    if !already {
                        let from = self.shade * SHADE_PULL_DP - target;
                        let speed = self.shade_speed.clamp(-2600.0, 2600.0);
                        self.shade_settle = Some((now, from, speed, SHADE_PERIOD, target));
                    }
                    self.shade_last = None;
                    self.shade_speed = 0.0;
                }
                fx
            }
        }
    }

    /// Set one of the sheet's sliders from where a finger is.
    fn shade_slider(&mut self, which: Slider, track: Rect, x: f32) {
        let level = ((x - track.x) / track.w).clamp(0.0, 1.0);
        match which {
            Slider::Brightness => self.state.brightness = level.max(0.02),
            Slider::Vibration => self.state.vibration = level,
            // The battery's limit is a whole degree, and it is set on the
            // charge page rather than in the control centre.
            Slider::ThermalLimit => {}
        }
        self.dirty = true;
    }

    /// Shut the control centre at once, for a toggle that opens a page.
    fn close_shade(&mut self) {
        self.shade = 0.0;
        self.shade_settle = None;
        self.shade_last = None;
        self.shade_speed = 0.0;
        // 收起来就退出编辑，拎着的那一枚也放回去：下次拉下来是平常那个样子
        //（2026-11-22）。
        self.shade_editing = false;
        self.shade_drag = None;
        self.dirty = true;
    }

    /// 面板拉下来的顶端在屏幕的哪儿；收着时在屏幕上方（负的）。
    pub(super) fn shade_top(&self) -> f32 {
        let t = self.theme();
        let tall = shade_height(&t, &self.state.tiles, self.shade_editing);
        tall * self.shade - tall
    }

    pub fn key_up(&mut self, key: Key, now: u64) -> Vec<Effect> {
        // A power key let go on the lock opens the lock, or the reboot page
        // where it was held long enough.
        if self.lock.is_some() && key == Key::Power {
            let held = self.lock_press.take().map_or(0, |at| now.saturating_sub(at));
            if held >= POWER_HOLD_MS {
                // Held, it opens the reboot page, and the lock goes with it.
                self.lock = None;
                self.open(Screen::Reboot);
                self.refresh();
                return Vec::new();
            }
            // A press that lit the panel leaves the lock showing -- the guard
            // just after a wake keeps it from going straight out again -- and
            // a second press, with the panel up, puts it out as it always
            // does. The knob is what opens it, not the key.
        }
        let mut fx = match self.chord {
            Some((first, _)) if first == key => self.let_chord_go(now),
            _ => Vec::new(),
        };
        let Some((down, press)) = self.button else { return fx };
        if down != key {
            return fx;
        }
        self.button = None;
        self.dirty = true;
        match press {
            Press::Power { spot, .. } => fx.extend(self.work(spot, now)),
            // 按住没够久：这一下就是息屏，和以前一样（并且震两下）。
            Press::PowerHold { at } => {
                if now.saturating_sub(at) < POWER_HOLD_MS {
                    self.sleep(now);
                    fx.push(Effect::Ticks);
                }
            }
            Press::Slide { which, track, .. } => {
                let done = self.state.confirm > SLID;
                self.let_knob_go(which, track);
                if done {
                    fx.extend(self.confirmed(which, now));
                } else {
                    self.say(self.state.lang.pick(["请按住电源键", "請按住電源鍵", "Hold the power key"]), now);
                }
            }
            Press::Volume { .. } | Press::Spent => {}
        }
        fx
    }

    /// Whatever the button held down was doing comes to nothing until it is
    /// let go: a knob it slid goes back.
    pub(super) fn drop_button(&mut self) {
        let Some((key, press)) = self.button else { return };
        if let Press::Slide { which, track, .. } = press {
            self.let_knob_go(which, track);
        }
        self.button = Some((key, Press::Spent));
    }

    /// Let go of a swipe-to-confirm control's knob, which goes back to the
    /// start of its track.
    fn let_knob_go(&mut self, which: Confirm, track: Rect) {
        self.state.confirm = 0.0;
        self.knob = Some((which, track));
        self.dirty = true;
    }

    fn on_knob(&self, track: Rect, x: f32) -> bool {
        let knob_x = track.x + (track.w - track.h) * self.state.confirm;
        x >= knob_x && x <= knob_x + track.h
    }

    fn repaint_confirm(&mut self, t: &Theme, f: &Fonts, which: Confirm, track: Rect) {
        self.paint_knob(t, f, which, track);
        self.dirty = true;
    }

    /// Draw the swipe-to-confirm control at `track` on the page again, its
    /// knob where it has come to.
    pub(super) fn paint_knob(&mut self, t: &Theme, f: &Fonts, which: Confirm, track: Rect) {
        let (label, danger) = pages::confirm_label(which, &self.state);
        if let Some(fr) = self.frame.as_mut() {
            fr.canvas.fill_rect(track.x, track.y, track.w, track.h, t.pal.bg);
            widgets::confirm(&mut fr.canvas, t, f, track.y, label, danger, self.state.confirm);
        }
        self.layer = None;
    }

    /// 手指落在录屏的计时胶囊上：拖着走就是挪它，点一下左边的红点是暂停或接着
    /// 录，点一下右边的方块就结束这一段。`None` 是这一下不归它。位置按上一帧
    /// 画出来的那一枚算，拖到哪儿就停在哪儿，推到屏幕边上就贴着边。
    fn record_touch(&mut self, ev: Touch, now: u64) -> Option<Vec<Effect>> {
        let pill = self.record_pill?;
        let m = self.theme().m;
        match ev {
            Touch::Down(x, y) if pill.body.contains(x, y) => {
                self.drag = Drag::RecordPill {
                    grab: (x - pill.body.x, y - pill.body.y),
                    moved: false,
                    dot: pill.dot.contains(x, y),
                    stop: pill.stop.contains(x, y),
                };
                Some(Vec::new())
            }
            Touch::Move(x, y) if matches!(self.drag, Drag::RecordPill { .. }) => {
                let Drag::RecordPill { grab, moved, dot, stop } = self.drag else { unreachable!() };
                let (nx, ny) = (x - grab.0, y - grab.1);
                let far = (nx - pill.body.x).abs() + (ny - pill.body.y).abs() > m.dp(4.0);
                let (w, h) = (m.w as f32, m.h as f32);
                self.state.record.pill = Some(((nx.min(w - pill.body.w)).max(0.0), (ny.min(h - pill.body.h)).max(0.0)));
                self.drag = Drag::RecordPill { grab, moved: moved || far, dot, stop };
                self.dirty = true;
                Some(Vec::new())
            }
            Touch::Up if matches!(self.drag, Drag::RecordPill { .. }) => {
                let Drag::RecordPill { moved, dot, stop, .. } = self.drag else { unreachable!() };
                self.drag = Drag::Idle;
                if moved {
                    return Some(vec![Effect::Haptic]);
                }
                // 右边的方块：结束这一段，和录屏页的「停止录制」是同一条路。
                if stop {
                    let mut effects = self.record_toggle(now);
                    effects.push(Effect::Haptic);
                    return Some(effects);
                }
                match dot {
                    true => Some(self.record_paused(now)),
                    false => Some(vec![Effect::Haptic]),
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::testing::*;
    use crate::color::over;

    #[test]
    fn a_row_held_darkens_behind_its_words_a_moment_after_the_finger_lands() {
        let (f, i) = (fonts(), info());
        let t = Theme::light(W, H, PPD);
        let mut app = App::new(Screen::Home, W, H, PPD);
        let mut c = Canvas::new(W, H);
        let row = app.frame(&f, &i).hits.iter().find(|h| h.action == Action::Open(Screen::Backup)).expect("on the home page").rect;
        // The card between the row's words and its arrow.
        let (x, y) = (W as f32 * 0.6, row.y + row.h / 2.0);
        app.compose(&mut c, &f, &i, "12:00", 0);
        let ink = darkest(&c, row);
        assert_eq!(grey_at(&c, x, y), t.pal.card.r);

        app.touch(Touch::Down(x, y), &f, 1000);
        let rise = 1000 + ROW_PRESS_DELAY_MS;
        assert_eq!(app.deadline(1000), Some(rise), "nothing moves until the row starts darkening");
        app.compose(&mut c, &f, &i, "12:00", 1000);
        assert!(app.row_art.is_none(), "a finger that may yet scroll does not wait for the row drawn held down");
        app.compose(&mut c, &f, &i, "12:00", rise);
        assert_eq!(grey_at(&c, x, y), t.pal.card.r, "it has not started yet");
        app.compose(&mut c, &f, &i, "12:00", rise + 30);
        let partway = grey_at(&c, x, y);
        assert!(partway < 255 && partway > 225, "darkening after 30 ms: {partway}");
        app.compose(&mut c, &f, &i, "12:00", 2000);
        assert_eq!(grey_at(&c, x, y), 225, "as dark as the stock settings' row held down");
        assert_eq!(darkest(&c, row), ink, "its words keep their ink");
        assert_ne!(app.deadline(2000), Some(0), "held still, nothing moves");

        // The finger moves on to scroll: the row fades back.
        app.touch(Touch::Move(x, y + t.m.dp(30.0)), &f, 2100);
        assert_eq!(app.deadline(2100), Some(0));
        app.touch(Touch::Up, &f, 2200);
        app.tick(3000);
        assert!(app.row_press.is_none() && app.row_art.is_none(), "faded, it is let go");
        assert_eq!(app.screen(), Screen::Home, "a scroll opens nothing");
    }

    #[test]
    fn a_row_scrolled_away_from_before_it_darkens_never_shows_held() {
        let (f, i) = (fonts(), info());
        let t = Theme::light(W, H, PPD);
        let mut app = App::new(Screen::Home, W, H, PPD);
        let row = app.frame(&f, &i).hits.iter().find(|h| h.action == Action::Open(Screen::Backup)).expect("on the home page").rect;
        let (x, y) = (W as f32 * 0.6, row.y + row.h / 2.0);
        app.touch(Touch::Down(x, y), &f, 1000);
        app.touch(Touch::Move(x, y + t.m.dp(30.0)), &f, 1050);
        assert!(app.row_press.is_none());
    }

    #[test]
    fn a_row_tapped_quickly_shows_held_from_the_lift_and_fades_back() {
        let (f, i) = (fonts(), info());
        let t = Theme::light(W, H, PPD);
        let mut app = App::new(Screen::Settings, W, H, PPD);
        let mut c = Canvas::new(W, H);
        let row = app.frame(&f, &i).hits.iter().find(|h| h.action == Action::Toggle(Switch::Vibrate)).expect("on the settings page").rect;
        let (x, y) = (W as f32 * 0.6, row.y + row.h / 2.0);
        tap_at(&mut app, &f, &i, Action::Toggle(Switch::Vibrate), 1000);
        assert!(!app.state.vibrate, "the tap worked the switch");
        app.compose(&mut c, &f, &i, "12:00", 1040);
        let held = grey_at(&c, x, y);
        assert!(held < t.pal.card.r, "held from the lift: {held}");
        app.tick(1040 + QUICK_TAP_MS + 1000);
        assert!(app.row_press.is_none(), "and let go once faded");
        app.compose(&mut c, &f, &i, "12:00", 2200);
        assert_eq!(grey_at(&c, x, y), t.pal.card.r);
    }

    #[test]
    fn the_back_control_and_the_search_buttons_darken_at_once_under_a_finger() {
        let (f, i) = (fonts(), info());
        let t = Theme::light(W, H, PPD);
        let mut c = Canvas::new(W, H);

        // `<`: a disc behind it, the `<` lighter on it.
        let mut app = opened(Screen::Mount, &f, &i);
        let (cx, cy, r) = widgets::back_disc(&t);
        let (bx, by) = widgets::back_rect(&t).centre();
        let around = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);
        app.compose(&mut c, &f, &i, "12:00", 5000);
        assert_eq!(grey_at(&c, cx + r * 0.6, cy), t.pal.bg.r);
        assert_eq!(darkest(&c, around), 0);
        app.touch(Touch::Down(bx, by), &f, 6000);
        app.compose(&mut c, &f, &i, "12:00", 6000);
        let disc = over(t.pal.button_press, t.pal.bg).r;
        assert_eq!(grey_at(&c, cx + r * 0.6, cy), disc, "darker at once");
        let arrow = darkest(&c, around);
        assert!(arrow.abs_diff(85) <= 2, "the stock `<` held down is 85, not {arrow}");
        // A finger moving off lets it go.
        app.touch(Touch::Move(bx, by + t.m.dp(40.0)), &f, 6050);
        app.compose(&mut c, &f, &i, "12:00", 6050);
        assert_eq!(grey_at(&c, cx + r * 0.6, cy), t.pal.bg.r);
        app.touch(Touch::Up, &f, 6100);
        assert_eq!(app.screen(), Screen::Mount);

        // The search page's round button.
        let mut app = App::new(Screen::Search, W, H, PPD);
        app.frame(&f, &i);
        let (_, button) = widgets::search_layout(&t);
        let (x, y) = button.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        app.compose(&mut c, &f, &i, "12:00", 1000);
        assert_eq!(grey_at(&c, x, y + button.h * 0.35), over(t.pal.button_press, t.pal.search).r);
        let cross = darkest(&c, button);
        assert!(cross.abs_diff(70) <= 2, "the stock cross held down is 70, not {cross}");

    }

    #[test]
    fn swipe_from_the_left_edge_goes_back() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Reboot, &f, &i);
        app.touch(Touch::Down(10.0, 1600.0), &f, 0);
        app.touch(Touch::Move(90.0, 1605.0), &f, 0);
        app.touch(Touch::Move(320.0, 1610.0), &f, 0);
        app.touch(Touch::Up, &f, 0);
        assert_eq!(app.screen(), Screen::Home);
    }

    #[test]
    fn a_short_edge_swipe_does_not_go_back() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Reboot, &f, &i);
        app.touch(Touch::Down(10.0, 1600.0), &f, 0);
        app.touch(Touch::Move(80.0, 1600.0), &f, 0);
        app.touch(Touch::Up, &f, 0);
        assert_eq!(app.screen(), Screen::Reboot);
    }

    #[test]
    fn the_first_page_has_no_edge_swipe_back() {
        let f = fonts();
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.touch(Touch::Down(10.0, 1600.0), &f, 0);
        app.touch(Touch::Move(80.0, 1600.0), &f, 8);
        assert!(app.touch(Touch::Move(400.0, 1600.0), &f, 16).is_empty(), "no tap of the motor");
        assert!(!matches!(app.drag, Drag::Edge { .. }), "no cue");
        app.touch(Touch::Up, &f, 24);
        assert_eq!(app.screen(), Screen::Home);
        assert_eq!(app.deadline(24), app.idle_due(), "nothing left on screen to settle");
    }

    #[test]
    fn the_back_cue_taps_once_as_it_arms_and_sinks_when_let_go() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Reboot, &f, &i);
        let ready = app.theme().m.dp(BACK_SWIPE_DP);
        app.touch(Touch::Down(10.0, 1600.0), &f, 0);
        assert!(app.touch(Touch::Move(80.0, 1600.0), &f, 8).is_empty());
        assert_eq!(app.touch(Touch::Move(20.0 + ready, 1600.0), &f, 16), vec![Effect::Haptic]);
        assert!(app.touch(Touch::Move(60.0 + ready, 1600.0), &f, 24).is_empty(), "one tap while armed");
        app.touch(Touch::Move(40.0, 1600.0), &f, 32);
        app.touch(Touch::Up, &f, 40);
        assert_eq!(app.screen(), Screen::Reboot, "pulled back in, it does not go back");
        assert_eq!(app.deadline(40), Some(0), "the cue is sinking back");
        app.tick(1000);
        assert_eq!(app.deadline(1000), app.idle_due());
    }

    #[test]
    fn the_back_cue_stays_quiet_with_touch_vibration_off() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Reboot, &f, &i);
        app.state.vibrate = false;
        app.touch(Touch::Down(10.0, 1600.0), &f, 0);
        app.touch(Touch::Move(80.0, 1600.0), &f, 8);
        assert!(app.touch(Touch::Move(400.0, 1600.0), &f, 16).is_empty());
    }

    #[test]
    fn the_back_cue_keeps_to_the_height_the_swipe_began_at() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Reboot, &f, &i);
        let at = 1600.0 - app.theme().m.dp(CUE_LIFT_DP);
        app.touch(Touch::Down(10.0, 1600.0), &f, 0);
        app.touch(Touch::Move(80.0, 1610.0), &f, 8);
        app.touch(Touch::Move(200.0, 1300.0), &f, 16);
        app.touch(Touch::Move(150.0, 1900.0), &f, 24);
        let Drag::Edge { cy, .. } = app.drag else { panic!("still swiping in from the edge") };
        assert_eq!(cy, at, "the finger wandered up and down");
        app.touch(Touch::Move(40.0, 2100.0), &f, 32);
        app.touch(Touch::Up, &f, 40);
        assert_eq!(app.cue.as_ref().map(|c| c.1), Some(at), "it sinks back where it was");
    }
}
