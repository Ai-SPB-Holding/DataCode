// Модуль для работы с файловыми операциями и SMB

use std::sync::{Arc, Mutex};
use crate::websocket::smb::SmbManager;

thread_local! {
    static SMB_MANAGER: std::cell::RefCell<Option<Arc<Mutex<SmbManager>>>> = std::cell::RefCell::new(None);
}

/// Установить SmbManager для текущего потока
pub fn set_smb_manager(manager: Arc<Mutex<SmbManager>>) {
    SMB_MANAGER.with(|m| {
        *m.borrow_mut() = Some(manager);
    });
}

/// Очистить SmbManager для текущего потока
pub fn clear_smb_manager() {
    SMB_MANAGER.with(|m| {
        *m.borrow_mut() = None;
    });
}

/// Получить SmbManager для текущего потока
pub fn get_smb_manager() -> Option<Arc<Mutex<SmbManager>>> {
    SMB_MANAGER.with(|m| {
        m.borrow().clone()
    })
}

