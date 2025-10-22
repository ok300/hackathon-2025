use crate::{PubkyApp, ViewState};

use eframe::egui::{Context, Ui};
use egui::CollapsingHeader;
use egui_commonmark::CommonMarkViewer;
use pubky::{PublicKey, PublicStorage};

pub(crate) fn update(
    app: &mut PubkyApp,
    pk: &PublicKey,
    public_storage: &PublicStorage,
    ctx: &Context,
    ui: &mut Ui,
) {
    ui.heading(egui::RichText::new("View Wiki Post").size(20.0));
    ui.add_space(25.0);

    // Page details in a subtle frame
    egui::Frame::NONE
        .fill(egui::Color32::from_gray(248))
        .inner_margin(10.0)
        .corner_radius(4.0)
        .show(ui, |ui| {
            CollapsingHeader::new(egui::RichText::new("📋 Page details").size(13.0))
                .default_open(false)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(format!("Page ID: {}", &app.selected_wiki_page_id)).size(12.0).color(egui::Color32::DARK_GRAY));
                    ui.label(egui::RichText::new(format!("User ID: {}", &app.selected_wiki_user_id)).size(12.0).color(egui::Color32::DARK_GRAY));
                });
        });

    ui.add_space(15.0);

    // Add "Share Page Link" button with tooltip support
    let share_button = ui.button(
        egui::RichText::new("🔗 Share Page Link").size(13.0)
    );

    // Show tooltip when hovering after copy
    if app.show_copy_tooltip {
        share_button.show_tooltip_text("✓ Copied to clipboard!");
    }

    if share_button.clicked() {
        ctx.copy_text(app.selected_wiki_page_id.clone());
        app.show_copy_tooltip = true;
    }

    // Reset tooltip if button is not being hovered
    if !share_button.hovered() && app.show_copy_tooltip {
        app.show_copy_tooltip = false;
    }

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(15.0);

    // Display content in a framed scrollable area
    egui::Frame::NONE
        .fill(egui::Color32::WHITE)
        .inner_margin(12.0)
        .corner_radius(6.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(220)))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(380.0)
                .show(ui, |ui| {
            // Try to fetch content if empty
            if app.selected_wiki_content.is_empty()
                && !app.selected_wiki_page_id.is_empty()
                && !app.selected_wiki_user_id.is_empty()
            {
                let public_storage_clone = public_storage.clone();
                let path_clone = app.selected_wiki_page_id.clone();
                let user_id = app.selected_wiki_user_id.clone();

                let path = format!("pubky{user_id}/pub/wiki.app/{path_clone}");

                // Synchronously fetch the content
                let get_path_fut = public_storage_clone.get(&path);
                match app.rt.block_on(get_path_fut) {
                    Ok(response) => {
                        let response_text_fut = response.text();
                        match app.rt.block_on(response_text_fut) {
                            Ok(text) => {
                                app.selected_wiki_content = text;
                            }
                            Err(e) => {
                                app.selected_wiki_content = format!("Error reading content: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        app.selected_wiki_content = format!("Error fetching path {path}: {}", e);
                    }
                }
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                CommonMarkViewer::new().max_image_width(Some(512)).show(
                    ui,
                    &mut app.cache,
                    &app.selected_wiki_content.as_str(),
                );
            });

            // Intercept link clicks by checking the output commands
            let clicked_urls: Vec<String> = ui.ctx().output_mut(|o| {
                let mut urls = Vec::new();
                // Drain commands to prevent external opening and capture URLs
                o.commands.retain(|cmd| {
                    if let egui::output::OutputCommand::OpenUrl(open_url) = cmd {
                        log::info!("Intercepted link click: {}", open_url.url);
                        urls.push(open_url.url.to_string());
                        false // Remove this command to prevent external opening
                    } else {
                        true // Keep other commands
                    }
                });
                urls
            });

            // Navigate to clicked URLs
            for url in clicked_urls {
                let mut parts = url.split('/'); // Split on the '/' character
                if let (Some(user_pk), Some(page_id)) = (parts.next(), parts.next()) {
                    app.navigate_to_view_wiki_page(user_pk, page_id);
                } else {
                    log::warn!("Invalid Pubky Wiki link: {url}");
                };
            }
            });
        });

    ui.add_space(25.0);

    // Check if this is the user's own page
    let is_own_page = app.selected_wiki_user_id == pk.to_string();

    ui.horizontal(|ui| {
        // Show Edit button only for own pages
        if is_own_page {
            let edit_btn = egui::Button::new(
                egui::RichText::new("✏ Edit").size(14.0)
            ).min_size(egui::vec2(100.0, 32.0));
            
            if ui.add(edit_btn).clicked() {
                app.navigate_to_edit_selected_wiki_page();
            }
            
            ui.add_space(8.0);
        }

        // Go back button
        let back_btn = egui::Button::new(
            egui::RichText::new("← Go back").size(14.0)
        ).min_size(egui::vec2(100.0, 32.0));
        
        if ui.add(back_btn).clicked() {
            app.selected_wiki_page_id.clear();
            app.selected_wiki_content.clear();
            app.view_state = ViewState::WikiList;
        }
    });
}
