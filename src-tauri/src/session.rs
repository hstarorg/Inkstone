use crate::crypto::MK_LEN;
use std::collections::HashMap;
use std::sync::Mutex;
use zeroize::Zeroizing;

/// Master keys for vaults unlocked during this app run, keyed by vault path.
/// Never persisted; cleared on lock or app exit. Assumes a single vault
/// instance per path per FORMAT.md's concurrency assumptions.
#[derive(Default)]
pub struct VaultSessions(Mutex<HashMap<String, Zeroizing<[u8; MK_LEN]>>>);

impl VaultSessions {
    pub fn unlock(&self, vault_path: &str, mk: [u8; MK_LEN]) {
        self.0
            .lock()
            .expect("session mutex poisoned")
            .insert(vault_path.to_string(), Zeroizing::new(mk));
    }

    pub fn lock_vault(&self, vault_path: &str) {
        self.0
            .lock()
            .expect("session mutex poisoned")
            .remove(vault_path);
    }

    pub fn get(&self, vault_path: &str) -> Option<[u8; MK_LEN]> {
        self.0
            .lock()
            .expect("session mutex poisoned")
            .get(vault_path)
            .map(|mk| **mk)
    }

    pub fn is_unlocked(&self, vault_path: &str) -> bool {
        self.0
            .lock()
            .expect("session mutex poisoned")
            .contains_key(vault_path)
    }
}
