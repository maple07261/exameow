use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Default)]
pub struct Controls(Mutex<Option<Arc<Session>>>);

struct Session {
    window: WebviewWindow,
    alive: AtomicBool,
    topmost: AtomicBool,
    held: AtomicBool,
    shortcut: Mutex<Option<Shortcut>>,
}

pub fn start(window: &WebviewWindow) {
    let session = Arc::new(Session {
        window: window.clone(), alive: AtomicBool::new(true),
        topmost: AtomicBool::new(true), held: AtomicBool::new(false),
        shortcut: Mutex::new(None),
    });
    *window.state::<Controls>().0.lock().unwrap() = Some(session.clone());
    let app = window.app_handle().clone();
    window.on_window_event(move |event| {
        if !matches!(event, WindowEvent::Destroyed) { return; }
        session.alive.store(false, Ordering::SeqCst);
        let shortcut = session.shortcut.lock().unwrap().take();
        if let Some(shortcut) = shortcut {
            if let Err(e) = app.global_shortcut().unregister(shortcut) {
                eprintln!("[answer-float] Could not unregister shortcut: {e}");
            }
        }
        let controls = app.state::<Controls>();
        let mut current = controls.0.lock().unwrap();
        if current.as_ref().is_some_and(|s| Arc::ptr_eq(s, &session)) { *current = None; }
    });
}

fn session(window: &WebviewWindow) -> Result<Arc<Session>, String> {
    if window.label() != "answer-float" { return Err("Only the answer window may change its controls".into()); }
    window.state::<Controls>().0.lock().unwrap().clone()
        .filter(|s| s.alive.load(Ordering::SeqCst)).ok_or_else(|| "Answer window is closed".into())
}

async fn on_ui<T: Send + 'static>(app: &AppHandle, action: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    let (send, receive) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || { let _ = send.send(action()); }).map_err(|e| e.to_string())?;
    receive.await.map_err(|e| e.to_string())?
}

fn apply_layer(session: &Session, topmost: bool) -> Result<(), String> {
    if !session.alive.load(Ordering::SeqCst) { return Err("Answer window is closed".into()); }
    #[cfg(target_os = "windows")]
    crate::answer_float_windows::set_topmost(&session.window, topmost)?;
    #[cfg(not(target_os = "windows"))]
    {
        // Clear the opposite level before setting the desired one. No focus call.
        if topmost {
            session.window.set_always_on_bottom(false).map_err(|e| e.to_string())?;
            session.window.set_always_on_top(true).map_err(|e| e.to_string())?;
        } else {
            session.window.set_always_on_top(false).map_err(|e| e.to_string())?;
            session.window.set_always_on_bottom(true).map_err(|e| e.to_string())?;
        }
    }
    session.topmost.store(topmost, Ordering::SeqCst);
    let _ = session.window.emit("answer-float:layer", topmost);
    Ok(())
}

pub async fn set_layer(window: WebviewWindow, topmost: bool) -> Result<bool, String> {
    let session = session(&window)?;
    on_ui(window.app_handle(), move || {
        let old = session.topmost.load(Ordering::SeqCst);
        if let Err(e) = apply_layer(&session, topmost) {
            let _ = apply_layer(&session, old);
            return Err(e);
        }
        Ok(topmost)
    }).await
}

pub async fn configure_shortcut(window: WebviewWindow, value: String) -> Result<String, String> {
    let session = session(&window)?;
    let value = value.trim().to_string();
    if value.len() > 128 { return Err("Shortcut is too long".into()); }
    let shortcut: Shortcut = value.parse().map_err(|e| format!("Invalid shortcut: {e}"))?;
    let app = window.app_handle().clone();
    on_ui(window.app_handle(), move || {
        if !session.alive.load(Ordering::SeqCst) { return Err("Answer window is closed".into()); }
        let old = *session.shortcut.lock().unwrap();
        if old == Some(shortcut) { return Ok(value); }
        let weak = Arc::downgrade(&session);
        // Register first: a conflict leaves the previous working shortcut intact.
        app.global_shortcut().on_shortcut(shortcut, move |app, _, event| {
            let Some(session) = weak.upgrade() else { return; };
            if !session.alive.load(Ordering::SeqCst) { return; }
            if event.state == ShortcutState::Released {
                session.held.store(false, Ordering::SeqCst);
                return;
            }
            if session.held.swap(true, Ordering::SeqCst) { return; }
            let _ = app.run_on_main_thread(move || {
                let old = session.topmost.load(Ordering::SeqCst);
                if let Err(e) = apply_layer(&session, !old) {
                    let _ = apply_layer(&session, old);
                    let _ = session.window.emit("answer-float:control-error", e);
                }
            });
        }).map_err(|e| e.to_string())?;
        if let Some(old) = old {
            if let Err(e) = app.global_shortcut().unregister(old) {
                let _ = app.global_shortcut().unregister(shortcut);
                return Err(e.to_string());
            }
        }
        *session.shortcut.lock().unwrap() = Some(shortcut);
        session.held.store(false, Ordering::SeqCst);
        Ok(value)
    }).await
}
