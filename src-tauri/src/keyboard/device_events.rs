//! Windows tells Keystra when any USB input device (HID interface) arrives or leaves, so the
//! worker can look for the keyboard right away instead of checking every second.

use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;

use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
    CM_NOTIFY_ACTION, CM_NOTIFY_ACTION_DEVICEINTERFACEARRIVAL, CM_NOTIFY_ACTION_DEVICEINTERFACEREMOVAL,
    CM_NOTIFY_EVENT_DATA, CM_NOTIFY_FILTER, CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE, CM_Register_Notification,
    CM_Unregister_Notification, CR_SUCCESS, HCMNOTIFICATION,
};
use windows_sys::Win32::Devices::HumanInterfaceDevice::GUID_DEVINTERFACE_HID;

type OnChange = Box<dyn Fn() + Send + Sync>;

/// A live device notification. Dropping it stops the notifications.
pub struct DeviceWatch {
    handle: HCMNOTIFICATION,
    on_change: *mut OnChange,
}

// The handle is an opaque token and the callback is Send + Sync, so the watch can move threads.
unsafe impl Send for DeviceWatch {}

impl DeviceWatch {
    /// Calls `on_change` from a Windows thread whenever a HID interface arrives or leaves.
    /// Returns `None` if Windows refuses, and the worker then falls back to checking on a timer.
    pub fn hid_changes(on_change: impl Fn() + Send + Sync + 'static) -> Option<Self> {
        let on_change: *mut OnChange = Box::into_raw(Box::new(Box::new(on_change)));
        let mut filter = CM_NOTIFY_FILTER {
            cbSize: size_of::<CM_NOTIFY_FILTER>() as u32,
            FilterType: CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE,
            ..Default::default()
        };
        filter.u.DeviceInterface.ClassGuid = GUID_DEVINTERFACE_HID;
        let mut handle: HCMNOTIFICATION = ptr::null_mut();
        // SAFETY: the filter and handle outlive the call, and `on_change` stays alive until Drop,
        // which unregisters (waiting for running callbacks) before freeing it.
        let result = unsafe { CM_Register_Notification(&filter, on_change as *const c_void, Some(notify), &mut handle) };
        if result != CR_SUCCESS {
            // SAFETY: registration failed, so Windows holds no copy of the pointer.
            drop(unsafe { Box::from_raw(on_change) });
            log::warn!("device notifications unavailable (error {result}), checking for the keyboard on a timer");
            return None;
        }
        Some(Self { handle, on_change })
    }
}

impl Drop for DeviceWatch {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful registration. Unregistering waits for callbacks
        // in progress, so nothing uses `on_change` once it's freed.
        unsafe {
            CM_Unregister_Notification(self.handle);
            drop(Box::from_raw(self.on_change));
        }
    }
}

unsafe extern "system" fn notify(
    _handle: HCMNOTIFICATION,
    context: *const c_void,
    action: CM_NOTIFY_ACTION,
    _data: *const CM_NOTIFY_EVENT_DATA,
    _size: u32,
) -> u32 {
    if matches!(action, CM_NOTIFY_ACTION_DEVICEINTERFACEARRIVAL | CM_NOTIFY_ACTION_DEVICEINTERFACEREMOVAL) {
        // SAFETY: `context` is the `OnChange` registered above, alive until the watch is dropped.
        let on_change = unsafe { &*(context as *const OnChange) };
        on_change();
    }
    0
}
