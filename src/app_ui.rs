//! 主界面：顶栏、海报墙、详情、设置页。布局对齐交互原型。

use eframe::egui::{self, Color32, Frame, Margin, RichText, Rounding, Stroke, Vec2};

use crate::models::MovieSort;
use crate::touch_keyboard;
use crate::ui::poster_grid;
use crate::config::ThemeMode;
use crate::ui::theme::{self, colors};

const PAD: f32 = 14.0;
const DETAIL_PAD: f32 = 14.0;
const TOP_H: f32 = 52.0;
const TOOL_H: f32 = 48.0;
const SECTION_H: f32 = 46.0;
const DETAIL_W: f32 = 268.0;
const CTRL_H: f32 = 36.0;
const FIELD_H: f32 = 44.0;
const DETAIL_BTN_H: f32 = 30.0;

use super::{BrowseMode, HomeTab, PosterLauncherApp};

impl PosterLauncherApp {
    pub(super) fn draw_ui(&mut self, ctx: &egui::Context) {
        self.draw_confirm_delete(ctx);
        self.draw_confirm_watched(ctx);

        // 底栏要先占位，否则中间画面会画到状态栏下面，底部按钮会被挡住一半。
        self.draw_status(ctx);

        if self.show_settings {
            self.draw_settings(ctx);
        } else {
            self.draw_top_bar(ctx);
            self.draw_toolbar(ctx);
            self.draw_detail(ctx);
            self.draw_grid(ctx);
            if self.show_history {
                self.draw_history(ctx);
            }
        }
    }

    fn draw_top_bar(&mut self, ctx: &egui::Context) {
        let mut search_enter = false;
        let mut clear_search = false;
        let mut library_id = None;

        egui::TopBottomPanel::top("top_bar")
            .exact_height(TOP_H)
            .frame(bar_frame(colors().top))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    let title_max = (ui.ctx().screen_rect().width() * 0.24).clamp(108.0, 220.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        brand_mark(ui);
                        brand_titles(ui, title_max);
                    });

                    let selected = self.selected_library_title();
                    ui.scope(|ui| {
                        ui.style_mut().spacing.interact_size.y = CTRL_H;
                        egui::ComboBox::from_id_source("library_selector")
                            .selected_text(selected)
                            .width(160.0)
                            .show_ui(ui, |ui| {
                            for section in &self.libraries {
                                if ui
                                    .selectable_value(
                                        &mut self.config.selected_library_id,
                                        section.id.clone(),
                                        &section.title,
                                    )
                                    .clicked()
                                {
                                    library_id = Some(section.id.clone());
                                }
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if icon_button(ui, "⚙", "设置").clicked() {
                            self.show_settings = true;
                            self.show_history = false;
                        }
                        let history = history_button(ui);
                        ui.ctx().data_mut(|data| {
                            data.insert_temp(egui::Id::new("history_button_rect"), history.rect);
                        });
                        if history.clicked() {
                            self.show_history = !self.show_history;
                        }
                        if let Some(mode) = theme_switch(ui, self.config.theme) {
                            self.config.theme = mode;
                            theme::set_mode(mode);
                            theme::apply(ui.ctx());
                        }

                        let search_width = (ui.available_width() - 8.0).clamp(0.0, 320.0);
                        if search_width >= 120.0 {
                        ui.scope(|ui| {
                            ui.allocate_ui_with_layout(
                                Vec2::new(search_width, CTRL_H),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    Frame::none()
                                        .fill(colors().field)
                                        .stroke(Stroke::new(1.0, colors().stroke))
                                        .rounding(Rounding::same(10.0))
                                        .inner_margin(Margin::symmetric(12.0, 0.0))
                                        .show(ui, |ui| {
                                            ui.allocate_ui_with_layout(
                                                Vec2::new(search_width - 26.0, CTRL_H),
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    search_glyph(ui);
                                                    // egui 0.27 的 hint 固定画在输入框左上角，不跟 vertical_align 走。
                                                    let hint_shape = ui.painter().add(egui::Shape::Noop);
                                                    let response = ui.add(
                                                        centered_field(
                                                            egui::TextEdit::singleline(&mut self.search_text)
                                                                .desired_width(f32::INFINITY)
                                                                .frame(false)
                                                                .margin(Margin::symmetric(4.0, 0.0))
                                                                .id_source("search_text"),
                                                            CTRL_H,
                                                        ),
                                                    );
                                                    if self.search_text.is_empty() {
                                                        paint_field_hint(
                                                            ui,
                                                            hint_shape,
                                                            response.rect.shrink2(Vec2::new(4.0, 0.0)),
                                                            "搜索整座片库...",
                                                        );
                                                    }
                                                    if response.gained_focus() {
                                                        touch_keyboard::show_touch_keyboard();
                                                    }
                                                    search_enter = response.lost_focus()
                                                        && ui.input(|input| input.key_pressed(egui::Key::Enter));
                                                    if !self.search_text.is_empty()
                                                        && put_button(
                                                            ui,
                                                            egui::Button::new(
                                                                RichText::new("×").size(18.0).color(colors().muted),
                                                            )
                                                            .frame(false),
                                                        )
                                                        .clicked()
                                                    {
                                                        clear_search = true;
                                                    }
                                                },
                                            );
                                        });
                            },
                        );
                        });
                        }
                    });
                });
                edge_line(ui, true);
            });

        if let Some(id) = library_id {
            self.config.selected_library_id = id;
            self.home_tab = HomeTab::All;
            self.active_search_query = None;
            self.search_text.clear();
            self.show_history = false;
            self.save_config_silently();
            self.reload_movies();
        } else if clear_search {
            self.search_text.clear();
            self.active_search_query = None;
            self.home_tab = HomeTab::All;
            self.reload_movies();
        } else if search_enter {
            self.home_tab = HomeTab::All;
            self.show_history = false;
            self.apply_search();
        }
    }

    fn draw_toolbar(&mut self, ctx: &egui::Context) {
        let mut next_tab = None;
        let mut next_sort = None;

        egui::TopBottomPanel::top("toolbar")
            .exact_height(TOOL_H)
            .frame(bar_frame(colors().bg))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if tab_button(ui, "全部电影", self.home_tab == HomeTab::All).clicked() {
                        next_tab = Some(HomeTab::All);
                    }
                    if tab_button(ui, "继续观看", self.home_tab == HomeTab::Continue).clicked() {
                        next_tab = Some(HomeTab::Continue);
                    }
                    if tab_button(ui, "最近添加", self.home_tab == HomeTab::Recent).clicked() {
                        next_tab = Some(HomeTab::Recent);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let sort_enabled = self.home_tab != HomeTab::Continue;
                        ui.add_enabled_ui(sort_enabled, |ui| {
                            ui.style_mut().spacing.interact_size.y = CTRL_H;
                            let mut sort_mode = self.sort_mode;
                            egui::ComboBox::from_id_source("sort_mode")
                                .selected_text(sort_mode.label())
                                .width(120.0)
                                .show_ui(ui, |ui| {
                                    for mode in [MovieSort::Newest, MovieSort::Oldest, MovieSort::Title]
                                    {
                                        if ui
                                            .selectable_value(&mut sort_mode, mode, mode.label())
                                            .clicked()
                                        {
                                            next_sort = Some(mode);
                                        }
                                    }
                                });
                        });
                        ui.label(RichText::new("排序方式").size(12.0).color(colors().muted));
                    });
                });
                edge_line(ui, true);
            });

        if let Some(tab) = next_tab {
            self.activate_tab(tab);
        } else if let Some(sort) = next_sort {
            self.set_sort(sort);
        }
    }

    fn draw_grid(&mut self, ctx: &egui::Context) {
        let mut toggle_newest = None;
        let mut load_more = false;
        let can_load_more = self.browse_mode == BrowseMode::Library
            && !self.is_loading_movies
            && self.total_size > 0
            && self.items.len() < self.total_size;
        egui::CentralPanel::default()
            .frame(Frame::none().fill(colors().bg))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                header_band(ui, "grid_header", SECTION_H, PAD, |ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    ui.label(
                        RichText::new(self.section_title())
                            .size(20.0)
                            .strong()
                            .color(colors().text),
                    );
                    ui.label(
                        RichText::new(format!("共 {} 部", self.section_count()))
                            .size(12.0)
                            .color(colors().muted),
                    );
                    if can_load_more && ui.add(load_more_button()).clicked() {
                        load_more = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;
                        ui.add_space(PAD);
                        if ui.available_width() > 132.0 {
                            ui.label(RichText::new("轻触海报查看详情").size(12.0).color(colors().muted));
                        }
                        let sort_enabled = self.home_tab != HomeTab::Continue;
                        let mut newest = self.sort_mode == MovieSort::Newest;
                        let toggled = ui
                            .scope(|ui| {
                                // 全局圆角是 9，勾选框本身很小，收成方框才看得出是 checkbox。
                                let visuals = ui.visuals_mut();
                                visuals.widgets.inactive.rounding = Rounding::same(3.0);
                                visuals.widgets.hovered.rounding = Rounding::same(3.0);
                                visuals.widgets.active.rounding = Rounding::same(3.0);
                                visuals.widgets.open.rounding = Rounding::same(3.0);
                                ui.add_enabled(
                                    sort_enabled,
                                    egui::Checkbox::new(&mut newest, "最新优先"),
                                )
                            })
                            .inner
                            .changed();
                        if toggled && sort_enabled {
                            toggle_newest = Some(newest);
                        }
                    });
                });

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(PAD);
                            let width = (ui.available_width() - PAD).max(120.0);
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                if self.items.is_empty() {
                                    self.draw_empty_grid(ui);
                                } else {
                                    let (columns, card_w) = poster_grid::grid_metrics(width);
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    let mut select_key = None;
                                    let mut first_row = true;
                                    for row in (0..self.items.len()).collect::<Vec<_>>().chunks(columns) {
                                        if !first_row {
                                            ui.add_space(12.0);
                                        }
                                        first_row = false;
                                        ui.horizontal_top(|ui| {
                                            ui.spacing_mut().item_spacing.x = 10.0;
                                            for index in row {
                                                let item = self.items[*index].clone();
                                                self.ensure_poster_texture(ctx, &item);
                                                let clicked = {
                                                    let texture = self.textures.get(&item.rating_key);
                                                    poster_grid::draw_poster_card(
                                                        ui,
                                                        card_w,
                                                        texture,
                                                        &item.title,
                                                        item.year,
                                                        self.selected_item_key.as_deref()
                                                            == Some(item.rating_key.as_str()),
                                                        item.view_count.unwrap_or_default() > 0,
                                                        continue_ratio(&item),
                                                    )
                                                    .clicked()
                                                };
                                                if clicked {
                                                    select_key = Some(item.rating_key);
                                                }
                                            }
                                        });
                                    }
                                    if let Some(key) = select_key {
                                        self.selected_item_key = Some(key);
                                        self.show_history = false;
                                    }
                                }

                                if can_load_more {
                                    ui.add_space(24.0);
                                    ui.vertical_centered(|ui| {
                                        if ui.add(load_more_button()).clicked() {
                                            load_more = true;
                                        }
                                    });
                                }
                                ui.add_space(16.0);
                            });
                        });
                    });
            });

        if load_more {
            self.load_more_movies();
        }
        if let Some(newest) = toggle_newest {
            if newest {
                self.set_sort(MovieSort::Newest);
            } else if self.sort_mode == MovieSort::Newest {
                self.set_sort(MovieSort::Oldest);
            }
        }
    }

    fn draw_empty_grid(&self, ui: &mut egui::Ui) {
        ui.add_space(72.0);
        ui.vertical_centered(|ui| {
            if self.is_loading_movies || self.is_loading_sections {
                ui.spinner();
                ui.label(RichText::new("正在加载影片…").color(colors().muted));
            } else if self.active_search_query.is_some() {
                ui.label(RichText::new("没有找到影片").size(17.0).strong());
                ui.label(RichText::new("换个关键词试试看，或清空搜索条件。").color(colors().muted));
            } else if !self.config.is_ready() {
                ui.label(RichText::new("还没有连接 Plex").size(17.0).strong());
                ui.label(RichText::new("打开右上角设置，填写服务器地址和 Token。").color(colors().muted));
            } else {
                ui.label(RichText::new("这个片库还没有影片").size(17.0).strong());
                ui.label(RichText::new("可以换一个片库，或到设置里重新加载。").color(colors().muted));
            }
        });
    }

    fn draw_detail(&mut self, ctx: &egui::Context) {
        let item = self.selected_item().cloned();
        if self.selected_item_key.is_some() && item.is_none() {
            self.selected_item_key = None;
        }

        let mut close = false;
        let mut play = false;
        let mut refresh = false;
        let mut mark = false;
        let mut delete = false;

        egui::SidePanel::right("details_panel")
            .exact_width(DETAIL_W)
            .resizable(false)
            .frame(Frame::none().fill(colors().panel).stroke(Stroke::new(1.0, colors().line)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let header = header_band(ui, "detail_header", SECTION_H, DETAIL_PAD, |ui| {
                    ui.label(RichText::new("影片详情").size(14.0).color(colors().muted));
                    if item.is_some() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(DETAIL_PAD - 8.0);
                            if put_button(
                                ui,
                                egui::Button::new(RichText::new("×").size(16.0).color(colors().muted))
                                    .frame(false)
                                    .min_size(Vec2::new(28.0, 28.0)),
                            )
                            .clicked()
                            {
                                close = true;
                            }
                        });
                    }
                });
                ui.painter().hline(
                    header.x_range(),
                    header.bottom() - 0.5,
                    Stroke::new(1.0, colors().line),
                );

                let Some(item) = item.as_ref() else {
                    return;
                };

                let watched = item.view_count.unwrap_or_default() > 0;
                let poster_busy = self.poster_jobs.contains(&item.rating_key);
                let marking =
                    self.marking_watched_item_key.as_deref() == Some(item.rating_key.as_str());
                let deleting = self.deleting_item_key.as_deref() == Some(item.rating_key.as_str());

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .scroll_bar_visibility(
                        egui::containers::scroll_area::ScrollBarVisibility::AlwaysHidden,
                    )
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.add_space(18.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.add_space(DETAIL_PAD);
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.set_width((ui.available_width() - DETAIL_PAD).max(40.0));

                                ui.label(RichText::new(&item.title).size(20.0).strong().color(colors().text));
                                ui.add_space(12.0);
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;
                                    ui.label(
                                        RichText::new(
                                            item.year
                                                .map(|year| year.to_string())
                                                .unwrap_or_else(|| "—".to_owned()),
                                        )
                                        .size(12.0)
                                        .color(colors().muted),
                                    );
                                    ui.label(RichText::new("·").size(12.0).color(colors().muted));
                                    ui.label(
                                        RichText::new(if watched { "已观看" } else { "未观看" })
                                            .size(12.0)
                                            .color(colors().muted),
                                    );
                                });

                                if let Some(ratio) = continue_ratio(item) {
                                    ui.add_space(14.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "上次看到 {}%",
                                            (ratio * 100.0).round() as i32
                                        ))
                                        .size(12.0)
                                        .color(colors().muted),
                                    );
                                    ui.add_space(8.0);
                                    let width = ui.available_width();
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::new(width, 4.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        rect,
                                        Rounding::same(2.0),
                                        colors().track,
                                    );
                                    let mut fill = rect;
                                    fill.max.x = rect.min.x + rect.width() * ratio;
                                    ui.painter().rect_filled(fill, Rounding::same(2.0), colors().accent);
                                }

                                ui.add_space(16.0);
                                let play_label = if continue_ratio(item).is_some() {
                                    "继续播放"
                                } else {
                                    "播放"
                                };
                                if put_button(
                                    ui,
                                    egui::Button::new(
                                        RichText::new(play_label).size(13.0).color(colors().accent_text),
                                    )
                                    .fill(colors().accent)
                                    .min_size(Vec2::new(88.0, DETAIL_BTN_H))
                                    .rounding(Rounding::same(8.0)),
                                )
                                .clicked()
                                {
                                    play = true;
                                }

                                ui.add_space(20.0);
                                ui.label(RichText::new("剧情简介").size(13.0).color(colors().text));
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(item.summary.as_deref().unwrap_or("暂无简介。"))
                                        .color(colors().summary)
                                        .size(13.0),
                                );

                                ui.add_space(18.0);
                                hairline(ui);
                                ui.add_space(14.0);
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;
                                    ui.style_mut().wrap = Some(false);
                                    let action_size = Vec2::new(108.0, DETAIL_BTN_H);
                                    let poster_label = if poster_busy { "更新中" } else { "更新海报" };
                                    if ui
                                        .add_enabled(
                                            !poster_busy,
                                            egui::Button::new(RichText::new(poster_label).size(12.0))
                                                .min_size(action_size)
                                                .rounding(Rounding::same(7.0)),
                                        )
                                        .clicked()
                                    {
                                        refresh = true;
                                    }
                                    let watched_label = if marking {
                                        "标记中"
                                    } else if watched {
                                        "已观看"
                                    } else {
                                        "标记已看"
                                    };
                                    if ui
                                        .add_enabled(
                                            !marking,
                                            egui::Button::new(RichText::new(watched_label).size(12.0))
                                                .min_size(action_size)
                                                .rounding(Rounding::same(7.0)),
                                        )
                                        .clicked()
                                    {
                                        mark = true;
                                    }
                                });

                                ui.add_space(14.0);
                                hairline(ui);
                                ui.add_space(14.0);
                                let delete_label = if deleting { "正在删除…" } else { "删除影片" };
                                if ui
                                    .add_enabled_ui(!deleting, |ui| {
                                        put_button(
                                            ui,
                                            egui::Button::new(
                                                RichText::new(delete_label).size(12.0).color(colors().danger),
                                            )
                                            .fill(colors().danger_bg)
                                            .min_size(Vec2::new(84.0, DETAIL_BTN_H))
                                            .rounding(Rounding::same(7.0)),
                                        )
                                    })
                                    .inner
                                    .clicked()
                                {
                                    delete = true;
                                }
                            });
                        });
                        ui.add_space(24.0);
                    });
            });

        if close {
            self.selected_item_key = None;
        }
        if let Some(item) = item {
            if play {
                self.play_item(&item);
            }
            if refresh {
                self.refresh_poster(&item);
            }
            if mark {
                self.request_mark_watched(item.clone());
            }
            if delete {
                self.request_delete_item(item);
            }
        }
    }

    fn draw_history(&mut self, ctx: &egui::Context) {
        let mut pick = None;
        let mut remove = None;
        let mut close = false;

        let parent = ctx.screen_rect();
        let anchor = ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new("history_button_rect")));
        let (mut right, mut top) = if let Some(rect) = anchor {
            (rect.right(), rect.bottom() + 6.0)
        } else {
            (parent.right() - PAD, TOP_H + 6.0)
        };
        right = right.clamp(parent.left() + 180.0, parent.right() - 8.0);
        top = top.clamp(parent.top() + 8.0, parent.bottom() - 140.0);
        let width = 285.0_f32.min(right - parent.left() - 8.0).max(180.0);
        let max_body = (parent.bottom() - 40.0 - top - 64.0).clamp(96.0, parent.height() * 0.5);
        egui::Window::new("搜索历史")
            .id(egui::Id::new("search_history_popup"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .pivot(egui::Align2::RIGHT_TOP)
            .fixed_pos(egui::pos2(right, top))
            .constrain_to(parent)
            .default_width(width)
            .max_width(width)
            .max_height(max_body + 56.0)
            .frame(
                Frame::popup(ctx.style().as_ref())
                    .fill(colors().window)
                    .rounding(Rounding::same(13.0))
                    .inner_margin(Margin::same(16.0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                ui.set_width((width - 32.0).max(160.0));
                ui.horizontal(|ui| {
                    ui.label(RichText::new("搜索历史").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new("关闭").min_size(Vec2::new(64.0, 32.0)))
                            .clicked()
                        {
                            close = true;
                        }
                    });
                });
                hairline(ui);
                if self.config.search_history.is_empty() {
                    ui.label(RichText::new("还没有保存的搜索记录").color(colors().muted));
                } else {
                    egui::ScrollArea::vertical().max_height(max_body).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;
                        ui.set_width((width - 32.0).max(160.0));
                        for (index, entry) in self.config.search_history.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;
                                let row_w = (ui.available_width() - 44.0).max(80.0);
                                if ui
                                    .add(
                                        egui::Button::new(entry)
                                            .min_size(Vec2::new(row_w, 36.0))
                                            .rounding(Rounding::same(8.0)),
                                    )
                                    .clicked()
                                {
                                    pick = Some(entry.clone());
                                }
                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("×").color(colors().danger))
                                            .min_size(Vec2::splat(36.0)),
                                    )
                                    .clicked()
                                {
                                    remove = Some(index);
                                }
                            });
                        }
                    });
                }
            });

        if close {
            self.show_history = false;
        }
        if let Some(index) = remove {
            self.delete_search_history_at(index);
        }
        if let Some(query) = pick {
            self.search_text = query;
            self.home_tab = HomeTab::All;
            self.show_history = false;
            self.apply_search();
        }
    }

    fn draw_settings(&mut self, ctx: &egui::Context) {
        let mut back = false;
        let mut save = false;
        let mut reload = false;
        let mut focused = false;

        egui::CentralPanel::default()
            .frame(Frame::none().fill(colors().settings_bg))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let (header, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), TOP_H),
                    egui::Sense::hover(),
                );
                let mut header_ui = ui.child_ui_with_id_source(
                    header,
                    egui::Layout::left_to_right(egui::Align::Center),
                    "settings_header",
                );
                header_ui.spacing_mut().item_spacing.x = 12.0;
                header_ui.add_space(PAD);
                if header_ui.add(settings_close_button()).clicked() {
                    back = true;
                }
                header_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.add_space(PAD);
                    if let Some(mode) = theme_switch(ui, self.config.theme) {
                        self.config.theme = mode;
                        theme::set_mode(mode);
                        theme::apply(ui.ctx());
                    }
                });
                edge_line(&header_ui, true);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.add_space(28.0);
                    let width = (ui.available_width() - PAD * 2.0).clamp(280.0, 760.0);
                    let side = ((ui.available_width() - width) * 0.5).max(PAD);
                    ui.horizontal(|ui| {
                        ui.add_space(side);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.set_width(width);
                            ui.label(RichText::new("连接与播放设置").size(26.0).strong());
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("仅在需要调整服务器或播放器时修改这些选项。")
                                    .color(colors().muted),
                            );
                            ui.add_space(24.0);
                            Frame::none()
                                .fill(colors().card)
                                .stroke(Stroke::new(1.0, colors().card_line))
                                .rounding(Rounding::same(15.0))
                                .inner_margin(Margin::same(26.0))
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    ui.spacing_mut().item_spacing.x = 10.0;
                                    focused |= labeled_field(
                                        ui,
                                        "Plex 服务器地址",
                                        &mut self.config.server_url,
                                        false,
                                        "例如：http://192.168.0.79:32400",
                                    );
                                    focused |= labeled_field(
                                        ui,
                                        "Plex Token",
                                        &mut self.config.token,
                                        true,
                                        "令牌会隐藏显示，只在点击保存后写入本机配置。",
                                    );
                                    focused |= labeled_field(
                                        ui,
                                        "VLC 可执行文件路径",
                                        &mut self.config.vlc_path,
                                        false,
                                        "Windows 上通常是 C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",
                                    );
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 10.0;
                                        ui.style_mut().wrap = Some(false);
                                        let settings_h = 42.0;
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new("保存设置").strong().color(colors().accent_text),
                                                )
                                                .fill(colors().accent)
                                                .min_size(Vec2::new(120.0, settings_h)),
                                            )
                                            .clicked()
                                        {
                                            save = true;
                                        }
                                        if ui
                                            .add(
                                                egui::Button::new("重新加载片库")
                                                    .min_size(Vec2::new(140.0, settings_h)),
                                            )
                                            .clicked()
                                        {
                                            reload = true;
                                        }
                                    });
                                });
                        });
                    });
                    ui.add_space(48.0);
                });
            });

        ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(focused));
        if back {
            self.show_settings = false;
        }
        if save {
            self.save_config();
        }
        if reload {
            self.load_sections();
        }
    }

    fn draw_status(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(32.0)
            .frame(bar_frame(colors().top))
            .show(ctx, |ui| {
                edge_line(ui, false);
                ui.horizontal_centered(|ui| {
                    let (dot, label) = if self.error_text.is_some() {
                        (colors().danger, "连接异常")
                    } else if self.config.is_ready() && !self.libraries.is_empty() {
                        (colors().online, "Plex 已连接")
                    } else if self.is_loading_sections {
                        (colors().accent, "正在连接")
                    } else {
                        (colors().muted, "尚未连接")
                    };
                    let (dot_rect, _) =
                        ui.allocate_exact_size(Vec2::splat(12.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 4.0, dot);
                    ui.label(RichText::new(label).size(11.0).color(colors().muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(error) = &self.error_text {
                            ui.label(RichText::new(error).size(11.0).color(Color32::LIGHT_RED));
                        } else {
                            ui.label(RichText::new(&self.status_text).size(11.0).color(colors().muted));
                        }
                    });
                });
            });
    }

    fn draw_confirm_delete(&mut self, ctx: &egui::Context) {
        let Some(item) = self.pending_delete_item.clone() else {
            return;
        };
        let mut cancel = false;
        let mut confirm = false;
        let budget = dialog_budget(ctx);
        paint_parent_veil(ctx);
        egui::Window::new("从 Plex 删除影片")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .constrain_to(ctx.screen_rect())
            .default_width(budget.width)
            .max_width(budget.width)
            .max_height(budget.height)
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.set_width(budget.inner_width);
                ui.label(RichText::new("从 Plex 删除影片").size(18.0).strong().color(colors().text));
                ui.add_space(14.0);
                ui.label(
                    RichText::new(format!("确定删除《{}》吗？", item.title)).color(colors().muted),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("此操作会要求 Plex 删除影片及其媒体文件，且无法撤销。")
                        .color(colors().muted),
                );
                ui.add_space(20.0);
                dialog_actions(ui, "确认删除", true, &mut confirm, &mut cancel);
            });
        if cancel {
            self.pending_delete_item = None;
        }
        if confirm {
            self.confirm_delete_item();
        }
    }

    fn draw_confirm_watched(&mut self, ctx: &egui::Context) {
        let Some(item) = self.pending_mark_watched_item.clone() else {
            return;
        };
        let mut cancel = false;
        let mut confirm = false;
        let budget = dialog_budget(ctx);
        paint_parent_veil(ctx);
        egui::Window::new("更改观看状态")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .constrain_to(ctx.screen_rect())
            .default_width(budget.width)
            .max_width(budget.width)
            .max_height(budget.height)
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.set_width(budget.inner_width);
                let mark_watched = item.view_count.unwrap_or_default() == 0;
                let prompt = if mark_watched {
                    format!("确定将《{}》标记为已看吗？", item.title)
                } else {
                    format!("确定将《{}》标记为未看吗？", item.title)
                };
                ui.label(RichText::new("更改观看状态").size(18.0).strong().color(colors().text));
                ui.add_space(14.0);
                ui.label(RichText::new(prompt).color(colors().muted));
                ui.add_space(20.0);
                dialog_actions(ui, "确认更改", false, &mut confirm, &mut cancel);
            });
        if cancel {
            self.pending_mark_watched_item = None;
        }
        if confirm {
            self.confirm_mark_watched();
        }
    }
}

fn continue_ratio(item: &crate::models::MediaItem) -> Option<f32> {
    let (offset, duration) = (item.view_offset?, item.duration?);
    if duration == 0 || offset < 60_000 {
        return None;
    }
    let ratio = offset as f32 / duration as f32;
    if ratio >= 0.95 {
        return None;
    }
    Some(ratio.clamp(0.02, 1.0))
}

fn bar_frame(fill: Color32) -> Frame {
    Frame::none()
        .fill(fill)
        .inner_margin(Margin::symmetric(PAD, 0.0))
}

fn dialog_frame(ctx: &egui::Context) -> Frame {
    Frame::popup(ctx.style().as_ref())
        .inner_margin(Margin::same(22.0))
        .rounding(Rounding::same(15.0))
}

struct DialogBudget {
    width: f32,
    height: f32,
    inner_width: f32,
}

/// 确认框宽高按父窗口计算，宽不超过 440，并且留出边距。
fn dialog_budget(ctx: &egui::Context) -> DialogBudget {
    let parent = ctx.screen_rect().size();
    let width = (parent.x - PAD * 2.0).clamp(260.0, 440.0);
    let height = (parent.y - 64.0).max(180.0);
    DialogBudget {
        width,
        height,
        inner_width: (width - 44.0).max(200.0),
    }
}

fn paint_parent_veil(ctx: &egui::Context) {
    let rect = ctx.screen_rect();
    egui::Area::new(egui::Id::new("modal_veil"))
        .order(egui::Order::Middle)
        .fixed_pos(rect.min)
        .interactable(true)
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                rect,
                Rounding::ZERO,
                colors().veil,
            );
            ui.allocate_response(rect.size(), egui::Sense::click());
        });
}

fn dialog_actions(ui: &mut egui::Ui, confirm_label: &str, danger: bool, confirm: &mut bool, cancel: &mut bool) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let confirm_button = if danger {
                egui::Button::new(RichText::new(confirm_label).color(Color32::WHITE))
                    .fill(colors().danger_fill)
                    .min_size(Vec2::new(96.0, 40.0))
            } else {
                egui::Button::new(RichText::new(confirm_label).color(colors().accent_text).strong())
                    .fill(colors().accent)
                    .min_size(Vec2::new(96.0, 40.0))
            };
            if ui.add(confirm_button).clicked() {
                *confirm = true;
            }
            if ui
                .add(egui::Button::new("取消").min_size(Vec2::new(80.0, 40.0)))
                .clicked()
            {
                *cancel = true;
            }
        });
    });
}

fn header_band(
    ui: &mut egui::Ui,
    id: &str,
    height: f32,
    inset: f32,
    add: impl FnOnce(&mut egui::Ui),
) -> egui::Rect {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), egui::Sense::hover());
    let mut child = ui.child_ui_with_id_source(
        rect,
        egui::Layout::left_to_right(egui::Align::Center),
        id,
    );
    child.add_space(inset);
    add(&mut child);
    rect
}

fn edge_line(ui: &egui::Ui, bottom: bool) {
    let clip = ui.clip_rect();
    let y = if bottom {
        clip.bottom() - 0.5
    } else {
        clip.top() + 0.5
    };
    ui.painter()
        .hline(clip.x_range(), y, Stroke::new(1.0, colors().line));
}

fn hairline(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter()
        .hline(rect.x_range(), rect.center().y, Stroke::new(1.0, colors().line));
}

fn load_more_button() -> egui::Button<'static> {
    egui::Button::new("加载更多电影")
        .min_size(Vec2::new(148.0, 32.0))
        .rounding(Rounding::same(9.0))
}

fn settings_close_button() -> egui::Button<'static> {
    egui::Button::new("关闭").min_size(Vec2::new(72.0, 32.0))
}

/// 横向排列时直接加入，让同一行按真实高度居中对齐。
/// 纵向排列时只借用居中布局排文字，外层仍从当前行顶往下占位。
fn put_button(ui: &mut egui::Ui, button: egui::Button<'_>) -> egui::Response {
    if ui.layout().is_horizontal() {
        ui.add(button)
    } else {
        ui.allocate_ui_with_layout(
            Vec2::ZERO,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| ui.add(button),
        )
        .inner
    }
}


fn brand_titles(ui: &mut egui::Ui, max_width: f32) {
    let title_font = egui::FontId::proportional(17.0);
    let sub_font = egui::FontId::proportional(10.0);
    let (title, sub) = ui.fonts(|fonts| {
        (
            fonts.layout_no_wrap("Plex Poster Launcher".to_owned(), title_font, colors().text),
            fonts.layout_no_wrap("私人电影放映室".to_owned(), sub_font, colors().muted),
        )
    });
    // 字框比可见字形高，负间距让两行标题的可见间距大约为 3px。
    let gap = -5.0;
    let title_size = title.size();
    let width = title_size.x.max(sub.size().x).min(max_width);
    let height = title_size.y + gap + sub.size().y;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);
    painter.galley(rect.left_top(), title, colors().text);
    painter.galley(egui::pos2(rect.left(), rect.top() + title_size.y + gap), sub, colors().muted);
}

fn brand_mark(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, Rounding::same(11.0), colors().brand);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "▶",
        egui::FontId::proportional(18.0),
        colors().brand_ink,
    );
}

fn search_glyph(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
    let center = rect.center() + Vec2::new(-1.5, -1.5);
    ui.painter()
        .circle_stroke(center, 5.0, Stroke::new(1.6, colors().muted));
    let start = center + Vec2::new(3.6, 3.6);
    ui.painter().line_segment(
        [start, start + Vec2::new(4.5, 4.5)],
        Stroke::new(1.6, colors().muted),
    );
}

fn history_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(CTRL_H), egui::Sense::click());
    let fill = if response.hovered() {
        colors().hover
    } else {
        colors().field
    };
    ui.painter()
        .rect_filled(rect, Rounding::same(10.0), fill);
    let center = rect.center();
    ui.painter()
        .circle_stroke(center, 7.0, Stroke::new(1.5, colors().text));
    ui.painter().line_segment(
        [center, center + Vec2::new(0.0, -4.5)],
        Stroke::new(1.5, colors().text),
    );
    ui.painter().line_segment(
        [center, center + Vec2::new(3.5, 2.0)],
        Stroke::new(1.5, colors().text),
    );
    response.on_hover_text("搜索历史")
}

fn theme_switch(ui: &mut egui::Ui, current: ThemeMode) -> Option<ThemeMode> {
    let mut next = None;
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        // 外层从右往左排，先放白天，看起来才是夜间在左。
        if theme_chip(ui, "白天", current == ThemeMode::Day).clicked() {
            next = Some(ThemeMode::Day);
        }
        if theme_chip(ui, "夜间", current == ThemeMode::Night).clicked() {
            next = Some(ThemeMode::Night);
        }
    });
    next.filter(|mode| *mode != current)
}

fn theme_chip(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    let palette = colors();
    let (fill, text) = if active {
        (palette.accent, palette.accent_text)
    } else {
        (palette.field, palette.muted)
    };
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(text))
            .fill(fill)
            .min_size(Vec2::new(64.0, CTRL_H))
            .rounding(Rounding::same(9.0)),
    )
}

fn icon_button(ui: &mut egui::Ui, text: &str, tip: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(18.0))
            .min_size(Vec2::splat(CTRL_H))
            .rounding(Rounding::same(10.0)),
    )
    .on_hover_text(tip)
}

fn tab_button(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    let color = if active { colors().tab_text } else { colors().muted };
    let fill = if active { colors().tab_active } else { Color32::TRANSPARENT };
    ui.add(
        egui::Button::new(RichText::new(label).color(color))
            .fill(fill)
            .frame(active)
            .min_size(Vec2::new(96.0, CTRL_H))
            .rounding(Rounding::same(9.0)),
    )
}

/// 单行输入框拉到指定高度，文字在框内垂直居中，避免字形冒到边框上面。
fn centered_field(edit: egui::TextEdit<'_>, height: f32) -> egui::TextEdit<'_> {
    edit.min_size(Vec2::new(0.0, height))
        .vertical_align(egui::Align::Center)
}

/// 把占位文字画在输入框内容区的垂直正中。先占一个形状槽，再填回去，光标才能盖在上面。
fn paint_field_hint(ui: &egui::Ui, shape: egui::layers::ShapeIdx, rect: egui::Rect, text: &str) {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let color = ui.visuals().weak_text_color();
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, color);
    let pos = egui::Align2::LEFT_CENTER
        .align_size_within_rect(galley.size(), rect)
        .min;
    ui.painter()
        .set(shape, egui::Shape::galley(pos, galley, color));
}

fn labeled_field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    password: bool,
    hint: &str,
) -> bool {
    ui.label(RichText::new(label).strong());
    ui.add_space(8.0);
    let response = ui.add(
        centered_field(
            egui::TextEdit::singleline(value)
                .password(password)
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(12.0, 0.0)),
            FIELD_H,
        ),
    );
    if response.gained_focus() {
        touch_keyboard::show_touch_keyboard();
    }
    ui.add_space(8.0);
    ui.label(RichText::new(hint).size(11.0).color(colors().muted));
    ui.add_space(20.0);
    response.has_focus()
}
