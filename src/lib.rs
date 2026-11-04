// Pulls in gpui-mobile's platform symbols (JNI / iOS FFI).
#[cfg(any(target_os = "ios", target_os = "android"))]
extern crate gpui_mobile;

pub mod platform;

{% if wear_os %}
use gpui::{App, Context, IntoElement, Render, Window, WindowOptions, div, prelude::*, px};
{% if wear_widget %}
pub use platform::{
  is_android_emulator, is_screen_round, is_watch, is_watch_viewport, set_widget_data,
  watch_inscribed_padding,
};
{% else %}
pub use platform::{
  is_android_emulator, is_screen_round, is_watch, is_watch_viewport, watch_inscribed_padding,
};
{% endif %}
{% else %}
use gpui::{App, Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
pub use platform::is_android_emulator;
{% endif %}

struct Home;

impl Render for Home {
{% if wear_os %}
  fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    let bounds = window.bounds();
    let watch = is_watch_viewport(bounds);
    let round = is_screen_round() || (!cfg!(target_os = "android") && watch);
    let safe_x = if round {
      watch_inscribed_padding(bounds.size.width)
    } else {
      px(16.0)
    };
    let safe_y = if round {
      bounds.size.height * 0.25
    } else {
      px(24.0)
    };

    div()
      .id("home-scroll")
      .size_full()
      .overflow_y_scroll()
      .px(safe_x)
      .py(safe_y)
      .flex()
      .flex_col()
      .items_center()
      .justify_center()
      .gap_3()
      .child("Hello from {{app_name}}")
  }
{% else %}
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .size_full()
      .flex()
      .items_center()
      .justify_center()
      .child("Hello from {{app_name}}")
  }
{% endif %}
}

/// Positron's shadcn palette (zinc + blue), dark only.
fn apply_theme(cx: &mut App) {
  use gpui_kit::component::{Theme, ThemeMode, ThemeRegistry};
  ThemeRegistry::global_mut(cx)
    .load_themes_from_str(include_str!("theme.json"))
    .expect("parse theme.json");
{% if wear_os %}
  let mut dark = ThemeRegistry::global(cx).themes()["Positron Dark"].clone();
  if is_watch() {
    let colors = &mut std::rc::Rc::make_mut(&mut dark).colors;
    colors.background = Some("#000000".into());
    colors.list = Some("#000000".into());
  }
{% else %}
  let dark = ThemeRegistry::global(cx).themes()["Positron Dark"].clone();
{% endif %}
  Theme::global_mut(cx).dark_theme = dark;
  Theme::change(ThemeMode::Dark, None, cx);
}

fn open_root(cx: &mut App) {
  gpui_kit::init(cx);
  apply_theme(cx);
  let options = WindowOptions {
    // Matches StartupWMClass in packaging/app.desktop
    app_id: Some("{{app_id}}".into()),
    ..Default::default()
  };
{% if wear_os %}
  let window_handle = cx
    .open_window(options, |window, cx| {
      let content = cx.new(|_| Home);
      cx.new(|cx| gpui_kit::component::Root::new(content, window, cx))
    })
    .expect("open root window");
  platform::spawn_rotary_listener(window_handle.into(), cx);
{% else %}
  cx.open_window(options, |window, cx| {
    let content = cx.new(|_| Home);
    cx.new(|cx| gpui_kit::component::Root::new(content, window, cx))
  })
  .expect("open root window");
{% endif %}
  cx.activate(true);
}

/// Desktop entry point (Linux, macOS, Windows), used by `src/main.rs`.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
pub fn run_desktop() {
  gpui_kit::application().run(open_root);
}

/// Called from App.swift before `gpui_ios_run_demo()`.
#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn gpui_ios_register_app() {
  gpui_mobile::ios::ffi::set_app_callback(Box::new(open_root));
}

/// Entry point called by android-activity on its native thread.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: android_activity::AndroidApp) {
  use gpui_mobile::android::jni;

  android_logger::init_once(
    android_logger::Config::default()
      .with_max_level(log::LevelFilter::Info)
      .with_tag("{{crate_name}}"),
  );
  jni::install_panic_hook();
  platform::configure_android_gpu();
  let _platform = jni::init_platform(&app);
  let Some(shared) = jni::shared_platform() else {
    log::error!("shared_platform() returned None");
    return;
  };
  // Blocks on the Android event loop; callback runs once the surface exists.
  gpui::Application::with_platform(shared.into_rc()).run(open_root);
}

#[cfg(test)]
mod tests {
  // Placeholder so the test pipeline has something to run; replace with real tests.
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
