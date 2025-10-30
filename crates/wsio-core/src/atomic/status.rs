use std::{
    marker::PhantomData,
    sync::atomic::{
        AtomicU8,
        Ordering,
    },
};

use anyhow::{
    Result,
    anyhow,
    bail,
};

// Structs
pub struct AtomicStatus<T: Eq + Into<u8> + PartialEq + TryFrom<u8>> {
    _marker: PhantomData<T>,
    inner: AtomicU8,
}

impl<T: Eq + Into<u8> + PartialEq + TryFrom<u8>> AtomicStatus<T> {
    #[inline]
    pub fn new(status: T) -> Self {
        Self {
            _marker: PhantomData,
            inner: AtomicU8::new(status.into()),
        }
    }

    // Public methods
    #[inline]
    pub fn ensure<F: FnOnce(T) -> String>(&self, expected: T, message: F) -> Result<()> {
        let status = self.get();
        if status != expected {
            bail!(message(status));
        }

        Ok(())
    }

    #[inline]
    pub fn get(&self) -> T {
        T::try_from(self.inner.load(Ordering::SeqCst)).ok().unwrap()
    }

    #[inline]
    pub fn is(&self, status: T) -> bool {
        self.inner.load(Ordering::SeqCst) == status.into()
    }

    #[inline]
    pub fn store(&self, status: T) {
        self.inner.store(status.into(), Ordering::SeqCst);
    }

    #[inline]
    pub fn try_transition(&self, from: T, to: T) -> Result<()> {
        self.inner
            .compare_exchange(from.into(), to.into(), Ordering::SeqCst, Ordering::SeqCst)
            .map(|_| ())
            .map_err(|_| anyhow!("Failed to transition status"))
    }
}
