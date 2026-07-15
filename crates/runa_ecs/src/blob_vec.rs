use std::alloc::Layout;
use std::any::TypeId;
use std::mem::needs_drop;

// ─── ComponentInfo ────────────────────────────────────────────

pub type DropFn = unsafe fn(*mut u8);

#[derive(Clone)]
pub struct ComponentInfo {
    pub type_id: TypeId,
    pub layout: Layout,
    pub drop_fn: Option<DropFn>,
}

impl ComponentInfo {
    pub fn of<T: 'static>() -> Self {
        let drop_fn = if needs_drop::<T>() {
            Some(drop_as::<T> as DropFn)
        } else {
            None
        };
        Self {
            type_id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            drop_fn,
        }
    }
}

unsafe fn drop_as<T>(ptr: *mut u8) {
    std::ptr::drop_in_place(ptr as *mut T)
}

// ─── BlobVec ──────────────────────────────────────────────────

pub struct BlobVec {
    info: ComponentInfo,
    data: Vec<u8>,
}

impl BlobVec {
    pub unsafe fn new<T: 'static>() -> Self {
        Self {
            info: ComponentInfo::of::<T>(),
            data: Vec::new(),
        }
    }

    pub fn new_with_info(info: ComponentInfo) -> Self {
        Self {
            info,
            data: Vec::new(),
        }
    }

    pub fn info(&self) -> &ComponentInfo {
        &self.info
    }

    pub fn get(&self, index: usize) -> *mut u8 {
        unsafe { self.data.as_ptr().add(index * self.info.layout.size()) as *mut u8 }
    }

    pub fn len(&self) -> usize {
        let size = self.info.layout.size();
        if size == 0 { self.data.len() } else { self.data.len() / size }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    pub unsafe fn push(&mut self, value: *mut u8) {
        let size = self.info.layout.size();
        if size == 0 {
            self.data.push(0);
            return;
        }
        let offset = self.data.len();
        self.data.reserve(size);
        unsafe {
            self.data.set_len(offset + size);
            std::ptr::copy_nonoverlapping(value, self.data.as_mut_ptr().add(offset), size);
        }
    }

    pub unsafe fn swap_remove(&mut self, index: usize) {
        let size = self.info.layout.size();
        if size == 0 {
            self.data.swap_remove(index);
            return;
        }
        let len = self.data.len() / size;

        if let Some(drop) = self.info.drop_fn {
            unsafe { drop(self.data.as_mut_ptr().add(index * size)) }
        }

        if index != len - 1 {
            let src = self.data.as_ptr().add((len - 1) * size);
            let dst = self.data.as_mut_ptr().add(index * size);
            unsafe { std::ptr::copy_nonoverlapping(src, dst, size) }
        }

        unsafe { self.data.set_len((len - 1) * size) }
    }

    pub fn clear(&mut self) {
        let size = self.info.layout.size();
        if size > 0 {
            if let Some(drop) = self.info.drop_fn {
                let len = self.data.len() / size;
                for i in 0..len {
                    unsafe { drop(self.data.as_mut_ptr().add(i * size)) }
                }
            }
        }
        self.data.clear();
    }
}
