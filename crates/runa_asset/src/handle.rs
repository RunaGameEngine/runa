use std::sync::Arc;

pub struct Handle<T> {
    pub inner: Arc<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
