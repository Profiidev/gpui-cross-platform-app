{% if wear_os -%}
use gpui::{AnyWindowHandle, App, Bounds, Pixels, px};
#[cfg(target_os = "android")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "android")]
static IS_WATCH: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "android")]
static IS_SCREEN_ROUND: AtomicBool = AtomicBool::new(false);

/// Returns `true` when running on a Wear OS watch (`PackageManager.FEATURE_WATCH`).
pub fn is_watch() -> bool {
  #[cfg(target_os = "android")]
  {
    IS_WATCH.load(Ordering::Relaxed)
  }
  #[cfg(not(target_os = "android"))]
  {
    false
  }
}

/// Returns `true` when the device has a circular display (`Configuration.isScreenRound()`).
pub fn is_screen_round() -> bool {
  #[cfg(target_os = "android")]
  {
    IS_SCREEN_ROUND.load(Ordering::Relaxed)
  }
  #[cfg(not(target_os = "android"))]
  {
    false
  }
}

{% endif -%}
/// Returns `true` when running on an Android emulator (`ro.kernel.qemu` or `ranchu`/`goldfish` hardware).
pub fn is_android_emulator() -> bool {
  #[cfg(target_os = "android")]
  {
    unsafe extern "C" {
      fn __system_property_get(
        name: *const std::ffi::c_char,
        value: *mut std::ffi::c_char,
      ) -> std::ffi::c_int;
    }
    let mut buf = [0u8; 92];
    let len = unsafe {
      __system_property_get(
        c"ro.kernel.qemu".as_ptr(),
        buf.as_mut_ptr().cast::<std::ffi::c_char>(),
      )
    };
    if len > 0 && &buf[..len as usize] == b"1" {
      return true;
    }
    let len = unsafe {
      __system_property_get(
        c"ro.hardware".as_ptr(),
        buf.as_mut_ptr().cast::<std::ffi::c_char>(),
      )
    };
    if len > 0 {
      let hw = std::str::from_utf8(&buf[..len as usize]).unwrap_or("");
      if hw.contains("ranchu") || hw.contains("goldfish") {
        return true;
      }
    }
    false
  }
  #[cfg(not(target_os = "android"))]
  {
    false
  }
}
{% if wear_os %}
/// Returns `true` when running on a watch or when the viewport has a compact square watch aspect ratio.
pub fn is_watch_viewport(bounds: Bounds<Pixels>) -> bool {
  is_watch() || (bounds.size.width == bounds.size.height && bounds.size.width < px(500.0))
}

/// Safe inset for static views inside a circular bezel: `D * (1 - sqrt(2)/2) / 2` (~0.1464 * D).
pub fn watch_inscribed_padding(diameter: Pixels) -> Pixels {
  diameter * 0.1464
}

/// Spawns the Wear OS rotary crown / bezel scroll listener on Android (no-op on other platforms).
pub fn spawn_rotary_listener(window_handle: AnyWindowHandle, cx: &mut App) {
  #[cfg(target_os = "android")]
  rotary::spawn_listener(window_handle, cx);
  #[cfg(not(target_os = "android"))]
  let _ = (window_handle, cx);
}
{% endif %}
{% if wear_widget %}
/// Updates the text displayed by the Wear OS Watch Face Complication service.
pub fn set_widget_data(
  title: impl Into<String>,
  value: impl Into<String>,
  subtitle: impl Into<String>,
) {
  #[cfg(target_os = "android")]
  widget::set_data(title.into(), value.into(), subtitle.into());
  #[cfg(not(target_os = "android"))]
  let _ = (title, value, subtitle);
}
{% endif %}
/// Selects the OpenGL ES backend when running on an Android emulator to avoid ranchu Vulkan driver bugs.
#[cfg(target_os = "android")]
pub fn configure_android_gpu() {
  use gpui_mobile::android::host::{GpuBackend, set_gpu_backend};
  if is_android_emulator() {
    log::info!(
      "Android emulator detected; selecting OpenGL ES backend to avoid ranchu Vulkan bugs"
    );
    set_gpu_backend(GpuBackend::Gl);
  }
}
{% if wear_os %}
#[cfg(target_os = "android")]
mod rotary {
  use gpui::{
    AnyWindowHandle, App, Modifiers, PlatformInput, ScrollDelta, ScrollWheelEvent, TouchPhase,
    point, px,
  };
  use std::{
    future::Future,
    pin::Pin,
    sync::{Mutex, OnceLock},
    task::{Context, Poll, Waker},
  };

  struct RotaryState {
    pending_delta: f32,
    waker: Option<Waker>,
  }

  static STATE: OnceLock<Mutex<RotaryState>> = OnceLock::new();

  fn state() -> &'static Mutex<RotaryState> {
    STATE.get_or_init(|| {
      Mutex::new(RotaryState {
        pending_delta: 0.0,
        waker: None,
      })
    })
  }

  pub fn push_delta(delta: f32) {
    if let Ok(mut guard) = state().lock() {
      guard.pending_delta += delta;
      if let Some(waker) = guard.waker.take() {
        waker.wake();
      }
    }
  }

  fn next_delta() -> impl Future<Output = f32> {
    struct NextDelta;
    impl Future for NextDelta {
      type Output = f32;
      fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<f32> {
        let mut guard = state().lock().unwrap();
        if guard.pending_delta.abs() > f32::EPSILON {
          let delta = guard.pending_delta;
          guard.pending_delta = 0.0;
          Poll::Ready(delta)
        } else {
          guard.waker = Some(cx.waker().clone());
          Poll::Pending
        }
      }
    }
    NextDelta
  }

  pub fn spawn_listener(window_handle: AnyWindowHandle, cx: &mut App) {
    cx.spawn(async move |cx| {
      loop {
        let delta = next_delta().await;
        let _ = window_handle.update(cx, |_, window, cx| {
          let scale = window.scale_factor();
          let bounds = window.bounds();
          let center = point(bounds.size.width / 2.0, bounds.size.height / 2.0);
          window.dispatch_event(
            PlatformInput::ScrollWheel(ScrollWheelEvent {
              position: center,
              delta: ScrollDelta::Pixels(point(px(0.0), px(delta / scale))),
              modifiers: Modifiers::default(),
              touch_phase: TouchPhase::Moved,
            }),
            cx,
          );
          window.refresh();
        });
      }
    })
    .detach();
  }
}

/// JNI bridge for device form-factor detection (`FEATURE_WATCH` and `Configuration.isScreenRound()`).
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_GpuiInputActivity_nativeSetDeviceFormFactor(
  _env: *mut std::ffi::c_void,
  _class: *mut std::ffi::c_void,
  watch: u8,
  round: u8,
) {
  IS_WATCH.store(watch != 0, Ordering::Relaxed);
  IS_SCREEN_ROUND.store(round != 0, Ordering::Relaxed);
}

/// JNI bridge for Wear OS rotary crown / bezel scroll events from `GpuiInputActivity`.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_GpuiInputActivity_nativeRotaryScroll(
  _env: *mut std::ffi::c_void,
  _class: *mut std::ffi::c_void,
  delta: f32,
) {
  log::info!("nativeRotaryScroll: delta={delta}");
  rotary::push_delta(delta);
}

/// Intercepts `AInputQueue_finishEvent` so `AMOTION_EVENT_ACTION_SCROLL` (e.g. Wear OS rotary
/// crown events) is marked unhandled by the native input queue and forwarded by Android to
/// `GpuiInputActivity.onGenericMotionEvent`.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn AInputQueue_finishEvent(
  queue: *mut std::ffi::c_void,
  event: *mut std::ffi::c_void,
  mut handled: std::ffi::c_int,
) {
  unsafe extern "C" {
    fn AInputEvent_getType(event: *const std::ffi::c_void) -> i32;
    fn AMotionEvent_getAction(event: *const std::ffi::c_void) -> i32;
    fn dlsym(
      handle: *mut std::ffi::c_void,
      symbol: *const std::ffi::c_char,
    ) -> *mut std::ffi::c_void;
  }
  const AINPUT_EVENT_TYPE_MOTION: i32 = 2;
  const AMOTION_EVENT_ACTION_MASK: i32 = 0xff;
  const AMOTION_EVENT_ACTION_SCROLL: i32 = 8;
  const RTLD_NEXT: *mut std::ffi::c_void = (-1isize) as *mut std::ffi::c_void;

  if !event.is_null() && unsafe { AInputEvent_getType(event) } == AINPUT_EVENT_TYPE_MOTION {
    let action = unsafe { AMotionEvent_getAction(event) } & AMOTION_EVENT_ACTION_MASK;
    if action == AMOTION_EVENT_ACTION_SCROLL {
      handled = 0;
    }
  }

  static REAL_FN: std::sync::OnceLock<
    unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, std::ffi::c_int),
  > = std::sync::OnceLock::new();
  let real = REAL_FN.get_or_init(|| unsafe {
    let ptr = dlsym(RTLD_NEXT, c"AInputQueue_finishEvent".as_ptr());
    std::mem::transmute(ptr)
  });
  unsafe { real(queue, event, handled) };
}
{% endif %}
{% if wear_widget %}
#[cfg(target_os = "android")]
mod widget {
  use std::{
    ffi::CString,
    sync::{Mutex, OnceLock},
  };

  struct WidgetState {
    title: CString,
    value: CString,
    subtitle: CString,
  }

  static STATE: OnceLock<Mutex<WidgetState>> = OnceLock::new();

  fn state() -> &'static Mutex<WidgetState> {
    STATE.get_or_init(|| {
      Mutex::new(WidgetState {
        title: CString::new("{{app_name}}").unwrap(),
        value: CString::new("Ready").unwrap(),
        subtitle: CString::new("Open App").unwrap(),
      })
    })
  }

  pub fn set_data(title: String, value: String, subtitle: String) {
    if let Ok(mut guard) = state().lock() {
      if let Ok(c) = CString::new(title) {
        guard.title = c;
      }
      if let Ok(c) = CString::new(value) {
        guard.value = c;
      }
      if let Ok(c) = CString::new(subtitle) {
        guard.subtitle = c;
      }
    }
  }

  pub unsafe fn new_jni_string(
    env: *mut *const *const std::ffi::c_void,
    s: &std::ffi::CStr,
  ) -> *mut std::ffi::c_void {
    type NewStringUtfFn = unsafe extern "system" fn(
      env: *mut *const *const std::ffi::c_void,
      utf: *const std::ffi::c_char,
    ) -> *mut std::ffi::c_void;
    let table = unsafe { *env };
    let func: NewStringUtfFn = unsafe { std::mem::transmute(*table.add(167)) };
    unsafe { func(env, s.as_ptr()) }
  }

  pub unsafe fn jni_title(env: *mut *const *const std::ffi::c_void) -> *mut std::ffi::c_void {
    let guard = state().lock().unwrap();
    unsafe { new_jni_string(env, &guard.title) }
  }

  pub unsafe fn jni_value(env: *mut *const *const std::ffi::c_void) -> *mut std::ffi::c_void {
    let guard = state().lock().unwrap();
    unsafe { new_jni_string(env, &guard.value) }
  }

  pub unsafe fn jni_subtitle(env: *mut *const *const std::ffi::c_void) -> *mut std::ffi::c_void {
    let guard = state().lock().unwrap();
    unsafe { new_jni_string(env, &guard.subtitle) }
  }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_dev_gpui_mobile_GpuiComplicationService_nativeGetWidgetTitle(
  env: *mut *const *const std::ffi::c_void,
  _class: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
  unsafe { widget::jni_title(env) }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_dev_gpui_mobile_GpuiComplicationService_nativeGetWidgetValue(
  env: *mut *const *const std::ffi::c_void,
  _class: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
  unsafe { widget::jni_value(env) }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_dev_gpui_mobile_GpuiComplicationService_nativeGetWidgetSubtitle(
  env: *mut *const *const std::ffi::c_void,
  _class: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
  unsafe { widget::jni_subtitle(env) }
}
{% endif -%}
