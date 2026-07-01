use crate::{
    components::Component,
    ocs::{Object, ObjectId, World},
};
use std::any::TypeId;

// ─── Bundle trait ──────────────────────────────────────────────

/// A group of components to spawn together.
///
/// Implemented for tuples of 1–16 components via a macro.
///
/// # Example
/// ```ignore
/// world.spawn_bundle((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     Health { max: 100, current: 100 },
/// ));
/// ```
pub trait Bundle {
    /// Consume self and produce (TypeId, boxed component) pairs.
    fn into_components(self) -> Vec<(TypeId, Box<dyn Component>)>;
}

macro_rules! impl_bundle {
    () => {};
    ($($T:ident),+) => {
        impl<$($T: Component),+> Bundle for ($($T,)+) {
            #[allow(non_snake_case)]
            fn into_components(self) -> Vec<(TypeId, Box<dyn Component>)> {
                let ($($T,)+) = self;
                vec![
                    $((TypeId::of::<$T>(), Box::new($T) as Box<dyn Component>),)+
                ]
            }
        }
    };
}

impl_bundle!(A);
impl_bundle!(A, B);
impl_bundle!(A, B, C);
impl_bundle!(A, B, C, D);
impl_bundle!(A, B, C, D, E);
impl_bundle!(A, B, C, D, E, F);
impl_bundle!(A, B, C, D, E, F, G);
impl_bundle!(A, B, C, D, E, F, G, H);
impl_bundle!(A, B, C, D, E, F, G, H, I);
impl_bundle!(A, B, C, D, E, F, G, H, I, J);
impl_bundle!(A, B, C, D, E, F, G, H, I, J, K);
impl_bundle!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_bundle!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_bundle!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_bundle!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_bundle!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

// ─── Inherent methods on World ────────────────────────────────

impl World {
    /// Spawn a bundle of components into the world.
    ///
    /// Returns the new entity's [`ObjectId`].
    ///
    /// # Example
    /// ```ignore
    /// let player = world.spawn_bundle((
    ///     Transform::from_xyz(0.0, 0.0, 0.0),
    ///     Health { max: 100, current: 100 },
    ///     PlayerController { speed: 5.0 },
    /// ));
    /// ```
    pub fn spawn_bundle(&mut self, bundle: impl Bundle) -> ObjectId {
        let mut object = Object::new("");
        // Object::new always adds a default Transform.
        // Code-first users include Transform in the bundle explicitly.
        object.components.clear();
        object.components = bundle.into_components();
        self.spawn_object(object)
    }

    /// Iterate over all entities that have component `T`.
    ///
    /// # Example
    /// ```ignore
    /// for hp in world.query_components::<Health>() {
    ///     println!("hp: {}", hp.current);
    /// }
    /// ```
    pub fn query_components<T: 'static>(&self) -> QueryRef<'_, T> {
        QueryRef {
            objects: &self.objects,
            cursor: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Mutably iterate over all entities that have component `T`.
    ///
    /// # Example
    /// ```ignore
    /// for hp in world.query_components_mut::<Health>() {
    ///     hp.current -= 10.0;
    /// }
    /// ```
    pub fn query_components_mut<T: 'static>(&mut self) -> QueryMut<'_, T> {
        let len = self.objects.len();
        let slice: &mut [Object] = self.objects.as_mut_slice();
        QueryMut {
            objects: slice as *mut [Object],
            len,
            cursor: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

// ─── Component Query iterators ─────────────────────────────────

/// Immutable query over a component type `T`.
pub struct QueryRef<'w, T> {
    objects: &'w [Object],
    cursor: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<'w, T: 'static> Iterator for QueryRef<'w, T> {
    type Item = &'w T;

    fn next(&mut self) -> Option<&'w T> {
        while self.cursor < self.objects.len() {
            let obj = &self.objects[self.cursor];
            self.cursor += 1;
            if let Some(comp) = obj.get_component::<T>() {
                return Some(comp);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.objects.len() - self.cursor))
    }
}

/// Mutable query over a component type `T`.
///
/// Yields one `&mut T` at a time. Standard `Iterator` guarantees no aliasing.
pub struct QueryMut<'w, T> {
    objects: *mut [Object],
    len: usize,
    cursor: usize,
    _phantom: std::marker::PhantomData<&'w mut T>,
}

impl<'w, T: 'static> Iterator for QueryMut<'w, T> {
    type Item = &'w mut T;

    fn next(&mut self) -> Option<&'w mut T> {
        // SAFETY: Each call returns at most one `&mut T` (Iterator contract).
        // The cursor advances monotonically and never revisits an element.
        unsafe {
            while self.cursor < self.len {
                let obj = &mut (*self.objects)[self.cursor];
                self.cursor += 1;
                if let Some(comp) = obj.get_component_mut::<T>() {
                    return Some(comp);
                }
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len - self.cursor))
    }
}
