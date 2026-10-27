//! Nonblocking platform file/folder chooser. GTK's native chooser uses the
//! desktop portal where available; its event queue is pumped from the UI loop.
use anyhow::{ensure, Result};
use libloading::Library;
use std::{
    ffi::{c_char, c_int, c_ulong, c_void, CStr, CString},
    path::{Path, PathBuf},
    sync::{mpsc, Arc, OnceLock},
};
type Ptr = *mut c_void;
#[derive(Clone)]
pub enum Purpose {
    Journal,
    Theme,
    Export(String),
}
struct Api {
    _gtk: Library,
    _gobject: Library,
    _glib: Library,
    create: unsafe extern "C" fn(*const c_char, Ptr, c_int, *const c_char, *const c_char) -> Ptr,
    show: unsafe extern "C" fn(Ptr),
    destroy: unsafe extern "C" fn(Ptr),
    unref: unsafe extern "C" fn(Ptr),
    folder: unsafe extern "C" fn(Ptr, *const c_char) -> c_int,
    name: unsafe extern "C" fn(Ptr, *const c_char),
    filename: unsafe extern "C" fn(Ptr) -> *mut c_char,
    filter_new: unsafe extern "C" fn() -> Ptr,
    filter_name: unsafe extern "C" fn(Ptr, *const c_char),
    filter_pattern: unsafe extern "C" fn(Ptr, *const c_char),
    add_filter: unsafe extern "C" fn(Ptr, Ptr),
    free: unsafe extern "C" fn(Ptr),
    connect: unsafe extern "C" fn(
        Ptr,
        *const c_char,
        unsafe extern "C" fn(),
        Ptr,
        Option<unsafe extern "C" fn(Ptr, Ptr)>,
        u32,
    ) -> c_ulong,
    disconnect: unsafe extern "C" fn(Ptr, c_ulong),
    pending: unsafe extern "C" fn() -> c_int,
    iteration: unsafe extern "C" fn(c_int) -> c_int,
}
struct Response {
    api: *const Api,
    tx: mpsc::Sender<Option<PathBuf>>,
}
unsafe extern "C" fn response(dialog: Ptr, id: c_int, data: Ptr) {
    unsafe {
        let state = &*(data.cast::<Response>());
        let api = &*state.api;
        let selected = if id == -3 {
            let raw = (api.filename)(dialog);
            if raw.is_null() {
                None
            } else {
                let path = PathBuf::from(CStr::from_ptr(raw).to_string_lossy().into_owned());
                (api.free)(raw.cast());
                Some(path)
            }
        } else {
            None
        };
        let _ = state.tx.send(selected);
    }
}
pub struct FileDialog {
    api: Arc<Api>,
    dialog: Ptr,
    _response: Box<Response>,
    handler: c_ulong,
    rx: mpsc::Receiver<Option<PathBuf>>,
    pub purpose: Purpose,
}
impl FileDialog {
    pub fn new(purpose: Purpose, initial: &Path) -> Result<Self> {
        unsafe {
            static API: OnceLock<std::result::Result<Arc<Api>, String>> = OnceLock::new();
            let api = API
                .get_or_init(|| {
                    (|| -> Result<Arc<Api>> {
                        let gtk = Library::new("libgtk-3.so.0")?;
                        let gobject = Library::new("libgobject-2.0.so.0")?;
                        let glib = Library::new("libglib-2.0.so.0")?;
                        macro_rules! sym {
                            ($lib:ident,$name:literal) => {
                                *$lib.get(concat!($name, "\0").as_bytes())?
                            };
                        }
                        let init: unsafe extern "C" fn(*mut c_int, *mut *mut *mut c_char) -> c_int =
                            sym!(gtk, "gtk_init_check");
                        ensure!(
                            init(std::ptr::null_mut(), std::ptr::null_mut()) != 0,
                            "Cannot connect the native file chooser to the display"
                        );
                        let api = Arc::new(Api {
                            create: sym!(gtk, "gtk_file_chooser_native_new"),
                            show: sym!(gtk, "gtk_native_dialog_show"),
                            destroy: sym!(gtk, "gtk_native_dialog_destroy"),
                            folder: sym!(gtk, "gtk_file_chooser_set_current_folder"),
                            name: sym!(gtk, "gtk_file_chooser_set_current_name"),
                            filename: sym!(gtk, "gtk_file_chooser_get_filename"),
                            filter_new: sym!(gtk, "gtk_file_filter_new"),
                            filter_name: sym!(gtk, "gtk_file_filter_set_name"),
                            filter_pattern: sym!(gtk, "gtk_file_filter_add_pattern"),
                            add_filter: sym!(gtk, "gtk_file_chooser_add_filter"),
                            free: sym!(glib, "g_free"),
                            unref: sym!(gobject, "g_object_unref"),
                            connect: sym!(gobject, "g_signal_connect_data"),
                            disconnect: sym!(gobject, "g_signal_handler_disconnect"),
                            pending: sym!(gtk, "gtk_events_pending"),
                            iteration: sym!(gtk, "gtk_main_iteration_do"),
                            _gtk: gtk,
                            _gobject: gobject,
                            _glib: glib,
                        });
                        Ok(api)
                    })()
                    .map_err(|error| format!("{error:#}"))
                })
                .as_ref()
                .map_err(|error| anyhow::anyhow!(error.clone()))?
                .clone();
            let export_title = match &purpose {
                Purpose::Export(name) => format!(
                    "{} Export",
                    if matches!(name.as_str(), "Explorer" | "Botanist" | "Communicator") {
                        format!("Observatory {name}")
                    } else {
                        name.clone()
                    }
                ),
                _ => String::new(),
            };
            let export_title = CString::new(export_title)?;
            let (title, action, accept) = match purpose {
                Purpose::Journal => (c"Select Journal Folder", 2, c"Select"),
                Purpose::Theme => (c"Elite Observatory Theme (*.eot)", 0, c"Open"),
                Purpose::Export(_) => (export_title.as_c_str(), 1, c"Save"),
            };
            let dialog = (api.create)(
                title.as_ptr(),
                std::ptr::null_mut(),
                action,
                accept.as_ptr(),
                c"Cancel".as_ptr(),
            );
            ensure!(!dialog.is_null(), "Cannot create native file chooser");
            let filter = match &purpose {
                Purpose::Export(_) => {
                    let ext = initial
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .unwrap_or("csv");
                    let label = if ext.eq_ignore_ascii_case("xlsx") {
                        format!("Office Open XML Spreadsheet (*.{ext})")
                    } else if ext.eq_ignore_ascii_case("csv") {
                        "Tab-separated values (*.csv)".into()
                    } else {
                        format!("Plugin-specified file type (*.{ext})")
                    };
                    Some((label, format!("*.{ext}")))
                }
                Purpose::Theme => Some(("Elite Observatory Theme (*.eot)".into(), "*.eot".into())),
                Purpose::Journal => None,
            };
            if let Some((label, pattern)) = filter {
                let filter = (api.filter_new)();
                let label = CString::new(label)?;
                let pattern = CString::new(pattern)?;
                (api.filter_name)(filter, label.as_ptr());
                (api.filter_pattern)(filter, pattern.as_ptr());
                (api.add_filter)(dialog, filter);
            }

            if let Some(folder) = if action == 2 {
                Some(initial)
            } else {
                initial.parent()
            } {
                let path = CString::new(folder.to_string_lossy().as_bytes())?;
                (api.folder)(dialog, path.as_ptr());
            }
            if action == 1 {
                if let Some(name) = initial.file_name() {
                    let name = CString::new(name.to_string_lossy().as_bytes())?;
                    (api.name)(dialog, name.as_ptr());
                }
            }
            let (tx, rx) = mpsc::channel();
            let state = Box::new(Response {
                api: Arc::as_ptr(&api),
                tx,
            });
            let callback = std::mem::transmute::<
                unsafe extern "C" fn(Ptr, c_int, Ptr),
                unsafe extern "C" fn(),
            >(response);
            let handler = (api.connect)(
                dialog,
                c"response".as_ptr(),
                callback,
                (&*state as *const Response).cast_mut().cast(),
                None,
                0,
            );
            (api.show)(dialog);
            Ok(Self {
                api,
                dialog,
                _response: state,
                handler,
                rx,
                purpose,
            })
        }
    }
    pub fn poll(&self) -> Option<Option<PathBuf>> {
        for _ in 0..32 {
            unsafe {
                if (self.api.pending)() == 0 {
                    break;
                }
                (self.api.iteration)(0);
            }
        }
        self.rx.try_recv().ok()
    }
}
impl Drop for FileDialog {
    fn drop(&mut self) {
        unsafe {
            (self.api.disconnect)(self.dialog, self.handler);
            (self.api.destroy)(self.dialog);
            (self.api.unref)(self.dialog);
        }
    }
}
