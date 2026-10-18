use crate::event::{Event, WindowEvent};
use crate::{DASHBOARD_ID, EventLoopProxy, EventLoopWindowTarget};
use fig_remote_ipc::figterm::FigtermState;
use muda::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub async fn build_tray(_: &EventLoopWindowTarget, _: &FigtermState) -> tray_icon::Result<TrayIcon> {
    TrayIconBuilder::new()
        .with_icon(get_icon(true))
        .with_icon_as_template(true)
        .with_menu(Box::new(get_context_menu(true)))
        .build()
}
pub fn get_icon(_: bool) -> Icon {
    // A simple terminal prompt mark, independent of AWS artwork.
    let mut rgba = vec![0u8; 22 * 22 * 4];
    for y in 0..22usize {
        for x in 0..22usize {
            if ((4..=10).contains(&x) && (y.abs_diff(11) == x - 3 || y.abs_diff(11) == x - 4))
                || ((12..=18).contains(&x) && (15..=16).contains(&y))
            {
                rgba[(y * 22 + x) * 4 + 3] = 255;
            }
        }
    }
    Icon::from_rgba(rgba, 22, 22).expect("Valid terminal icon")
}
pub fn get_context_menu(_: bool) -> Menu {
    let menu = Menu::new();
    let settings = MenuItem::with_id("settings", "Termcomplete Settings…", true, None);
    let quit = MenuItem::with_id("quit", "Quit Termcomplete", true, None);
    menu.append_items(&[&settings, &quit]).expect("Valid tray menu");
    menu
}
pub fn handle_event(event: &MenuEvent, proxy: &EventLoopProxy) {
    match event.id().0.as_str() {
        "quit" => {
            let _ = proxy.send_event(Event::ControlFlow(tao::event_loop::ControlFlow::Exit));
        }
        "settings" => {
            let _ = proxy.send_event(Event::WindowEvent {
                window_id: DASHBOARD_ID,
                window_event: WindowEvent::Show,
            });
        }
        _ => {}
    }
}
