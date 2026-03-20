#![cfg(target_os = "macos")]

use std::ffi::CStr;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Mutex, OnceLock};

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{class, declare_class, msg_send, msg_send_id, mutability, sel, ClassType, DeclaredClass};

// Apple Event four-char codes
const KEY_DIRECT_OBJECT: u32 = 0x2D2D2D2D; // '----'
const AE_CORE_EVENT_CLASS: u32 = 0x61657674; // 'aevt'
const AE_OPEN_DOCUMENTS: u32 = 0x6F646F63; // 'odoc'

static FILE_SENDER: OnceLock<Mutex<Sender<PathBuf>>> = OnceLock::new();

/// Called from the ObjC handler; safe to call from any thread.
fn process_open_event(event: &AnyObject) {
    unsafe {
        let list: Option<Retained<AnyObject>> =
            msg_send_id![event, paramDescriptorForKeyword: KEY_DIRECT_OBJECT];
        let Some(list) = list else { return };

        // Apple Event lists are 1-indexed
        let count: i32 = msg_send![&*list, numberOfItems];
        for i in 1..=count {
            let item: Option<Retained<AnyObject>> =
                msg_send_id![&*list, descriptorAtIndex: i];
            let Some(item) = item else { continue };

            let url: Option<Retained<AnyObject>> = msg_send_id![&*item, fileURLValue];
            let Some(url) = url else { continue };

            let path_ns: Option<Retained<AnyObject>> = msg_send_id![&*url, path];
            let Some(path_ns) = path_ns else { continue };

            let c_str: *const std::ffi::c_char = msg_send![&*path_ns, UTF8String];
            if c_str.is_null() {
                continue;
            }
            let path = PathBuf::from(CStr::from_ptr(c_str).to_string_lossy().as_ref());

            if let Some(lock) = FILE_SENDER.get() {
                if let Ok(tx) = lock.lock() {
                    let _ = tx.send(path);
                }
            }
        }
    }
}

declare_class!(
    struct FileOpenHandler;

    unsafe impl ClassType for FileOpenHandler {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "MReaderFileOpenHandler";
    }

    impl DeclaredClass for FileOpenHandler {
        type Ivars = ();
    }

    unsafe impl FileOpenHandler {
        #[method(handleOpenDocumentEvent:withReplyEvent:)]
        fn handle_open_document(&self, event: &AnyObject, _reply: &AnyObject) {
            process_open_event(event);
        }
    }
);

pub fn register_open_document_handler(tx: Sender<PathBuf>) {
    FILE_SENDER
        .set(Mutex::new(tx))
        .expect("register_open_document_handler must only be called once");

    unsafe {
        // Intentional leak: NSAppleEventManager holds an unretained reference;
        // the handler object must live for the entire app lifetime.
        let handler: Retained<FileOpenHandler> =
            msg_send_id![FileOpenHandler::class(), new];
        let raw: *mut FileOpenHandler = Retained::into_raw(handler);
        let handler_ptr: *mut AnyObject = raw.cast();

        let manager: Retained<AnyObject> =
            msg_send_id![class!(NSAppleEventManager), sharedAppleEventManager];
        let () = msg_send![
            &*manager,
            setEventHandler: handler_ptr,
            andSelector: sel!(handleOpenDocumentEvent:withReplyEvent:),
            forEventClass: AE_CORE_EVENT_CLASS,
            andEventID: AE_OPEN_DOCUMENTS
        ];
    }
}
