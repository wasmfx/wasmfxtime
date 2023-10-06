use std::any::Any;
use std::collections::{BTreeSet, HashMap};
use wasmtime::component::Resource;

#[derive(thiserror::Error, Debug)]
pub enum TableError {
    #[error("table has no free keys")]
    Full,
    #[error("value not present")]
    NotPresent,
    #[error("value is of another type")]
    WrongType,
    #[error("entry still has children")]
    HasChildren,
}

/// The `Table` type is designed to map u32 handles to resources. The table is now part of the
/// public interface to a `WasiCtx` - it is reference counted so that it can be shared beyond a
/// `WasiCtx` with other WASI proposals (e.g. `wasi-crypto` and `wasi-nn`) to manage their
/// resources. Elements in the `Table` are `Any` typed.
///
/// The `Table` type is intended to model how the Interface Types concept of Resources is shaping
/// up. Right now it is just an approximation.
#[derive(Debug)]
pub struct Table {
    map: HashMap<u32, TableEntry>,
    next_key: u32,
}

/// This structure tracks parent and child relationships for a given table entry.
///
/// Parents and children are referred to by table index. We maintain the
/// following invariants to prevent orphans and cycles:
/// * parent can only be assigned on creating the entry.
/// * parent, if some, must exist when creating the entry.
/// * whenever a child is created, its index is added to children.
/// * whenever a child is deleted, its index is removed from children.
/// * an entry with children may not be deleted.
#[derive(Debug)]
struct TableEntry {
    /// The entry in the table, as a boxed dynamically-typed object
    entry: Box<dyn Any + Send + Sync>,
    /// The index of the parent of this entry, if it has one.
    parent: Option<u32>,
    /// The indicies of any children of this entry.
    children: BTreeSet<u32>,
}

impl TableEntry {
    fn new(entry: Box<dyn Any + Send + Sync>, parent: Option<u32>) -> Self {
        Self {
            entry,
            parent,
            children: BTreeSet::new(),
        }
    }
    fn add_child(&mut self, child: u32) {
        debug_assert!(!self.children.contains(&child));
        self.children.insert(child);
    }
    fn remove_child(&mut self, child: u32) {
        let was_removed = self.children.remove(&child);
        debug_assert!(was_removed);
    }
}

impl Table {
    /// Create an empty table
    pub fn new() -> Self {
        Table {
            map: HashMap::new(),
            // 0, 1 and 2 are formerly (preview 1) for stdio. To prevent users from assuming these
            // indicies are still valid ways to access stdio, they are deliberately left empty.
            // Once we have a full implementation of resources, this confusion should hopefully be
            // impossible :)
            next_key: 3,
        }
    }

    /// Insert a resource at the next available index.
    pub fn push(&mut self, entry: Box<dyn Any + Send + Sync>) -> Result<u32, TableError> {
        self.push_(TableEntry::new(entry, None))
    }

    /// Same as `push`, but typed.
    pub fn push_resource<T>(&mut self, entry: T) -> Result<Resource<T>, TableError>
    where
        T: Send + Sync + 'static,
    {
        let idx = self.push(Box::new(entry))?;
        Ok(Resource::new_own(idx))
    }

    /// Insert a resource at the next available index, and track that it has a
    /// parent resource.
    ///
    /// The parent must exist to create a child. All children resources must
    /// be destroyed before a parent can be destroyed - otherwise [`Table::delete`]
    /// will fail with [`TableError::HasChildren`].
    ///
    /// Parent-child relationships are tracked inside the table to ensure that
    /// a parent resource is not deleted while it has live children. This
    /// allows child resources to hold "references" to a parent by table
    /// index, to avoid needing e.g. an `Arc<Mutex<parent>>` and the associated
    /// locking overhead and design issues, such as child existence extending
    /// lifetime of parent referent even after parent resource is destroyed,
    /// possibility for deadlocks.
    ///
    /// Parent-child relationships may not be modified once created. There
    /// is no way to observe these relationships through the [`Table`] methods
    /// except for erroring on deletion, or the [`std::fmt::Debug`] impl.
    pub fn push_child(
        &mut self,
        entry: Box<dyn Any + Send + Sync>,
        parent: u32,
    ) -> Result<u32, TableError> {
        if !self.contains_key(parent) {
            return Err(TableError::NotPresent);
        }
        let child = self.push_(TableEntry::new(entry, Some(parent)))?;
        self.map
            .get_mut(&parent)
            .expect("parent existence assured above")
            .add_child(child);
        Ok(child)
    }

    /// Same as `push_child`, but typed.
    pub fn push_child_resource<T, U>(
        &mut self,
        entry: T,
        parent: &Resource<U>,
    ) -> Result<Resource<T>, TableError>
    where
        T: Send + Sync + 'static,
        U: 'static,
    {
        let idx = self.push_child(Box::new(entry), parent.rep())?;
        Ok(Resource::new_own(idx))
    }

    fn push_(&mut self, e: TableEntry) -> Result<u32, TableError> {
        // NOTE: The performance of this new key calculation could be very bad once keys wrap
        // around.
        if self.map.len() == u32::MAX as usize {
            return Err(TableError::Full);
        }
        loop {
            let key = self.next_key;
            self.next_key = self.next_key.wrapping_add(1);
            if self.map.contains_key(&key) {
                continue;
            }
            self.map.insert(key, e);
            return Ok(key);
        }
    }

    /// Check if the table has a resource at the given index.
    pub fn contains_key(&self, key: u32) -> bool {
        self.map.contains_key(&key)
    }

    /// Check if the resource at a given index can be downcast to a given type.
    /// Note: this will always fail if the resource is already borrowed.
    pub fn is<T: Any + Sized>(&self, key: u32) -> bool {
        if let Some(r) = self.map.get(&key) {
            r.entry.is::<T>()
        } else {
            false
        }
    }

    /// Get a mutable reference to the underlying untyped cell for an entry in the table.
    pub fn get_any_mut(&mut self, key: u32) -> Result<&mut dyn Any, TableError> {
        if let Some(r) = self.map.get_mut(&key) {
            Ok(&mut *r.entry)
        } else {
            Err(TableError::NotPresent)
        }
    }

    /// Get an immutable reference to a resource of a given type at a given index. Multiple
    /// immutable references can be borrowed at any given time. Borrow failure
    /// results in a trapping error.
    pub fn get<T: Any + Sized>(&self, key: u32) -> Result<&T, TableError> {
        if let Some(r) = self.map.get(&key) {
            r.entry
                .downcast_ref::<T>()
                .ok_or_else(|| TableError::WrongType)
        } else {
            Err(TableError::NotPresent)
        }
    }

    /// Get a mutable reference to a resource of a given type at a given index.
    pub fn get_mut<T: Any + Sized>(&mut self, key: u32) -> Result<&mut T, TableError> {
        if let Some(r) = self.map.get_mut(&key) {
            r.entry
                .downcast_mut::<T>()
                .ok_or_else(|| TableError::WrongType)
        } else {
            Err(TableError::NotPresent)
        }
    }

    /// Get a mutable reference to a resource a a `&mut dyn Any`.
    pub fn get_as_any_mut(&mut self, key: u32) -> Result<&mut dyn Any, TableError> {
        if let Some(r) = self.map.get_mut(&key) {
            Ok(&mut *r.entry)
        } else {
            Err(TableError::NotPresent)
        }
    }

    /// Same as `get`, but typed
    pub fn get_resource<T: Any + Sized>(&self, key: &Resource<T>) -> Result<&T, TableError> {
        self.get(key.rep())
    }

    /// Same as `get_mut`, but typed
    pub fn get_resource_mut<T: Any + Sized>(
        &mut self,
        key: &Resource<T>,
    ) -> Result<&mut T, TableError> {
        self.get_mut(key.rep())
    }

    fn delete_entry(&mut self, key: u32) -> Result<TableEntry, TableError> {
        if !self
            .map
            .get(&key)
            .ok_or(TableError::NotPresent)?
            .children
            .is_empty()
        {
            return Err(TableError::HasChildren);
        }
        let e = self.map.remove(&key).unwrap();
        if let Some(parent) = e.parent {
            // Remove deleted resource from parent's child list.
            // Parent must still be present because it cant be deleted while still having
            // children:
            self.map
                .get_mut(&parent)
                .expect("missing parent")
                .remove_child(key);
        }
        Ok(e)
    }

    /// Remove a resource at a given index from the table.
    ///
    /// If this method fails, the resource remains in the table.
    ///
    /// May fail with [`TableError::HasChildren`] if the resource has any live
    /// children.
    pub fn delete<T: Any + Sized>(&mut self, key: u32) -> Result<T, TableError> {
        let e = self.delete_entry(key)?;
        match e.entry.downcast::<T>() {
            Ok(v) => Ok(*v),
            Err(entry) => {
                // Re-insert into parent list
                if let Some(parent) = e.parent {
                    self.map
                        .get_mut(&parent)
                        .expect("already checked parent exists")
                        .add_child(key);
                }
                // Insert the value back
                self.map.insert(
                    key,
                    TableEntry {
                        entry,
                        children: e.children,
                        parent: e.parent,
                    },
                );
                Err(TableError::WrongType)
            }
        }
    }

    /// Same as `delete`, but typed
    pub fn delete_resource<T>(&mut self, resource: Resource<T>) -> Result<T, TableError>
    where
        T: Any,
    {
        debug_assert!(resource.owned());
        self.delete(resource.rep())
    }

    /// Zip the values of the map with mutable references to table entries corresponding to each
    /// key. As the keys in the [HashMap] are unique, this iterator can give mutable references
    /// with the same lifetime as the mutable reference to the [Table].
    pub fn iter_entries<'a, T>(
        &'a mut self,
        map: HashMap<u32, T>,
    ) -> impl Iterator<Item = (Result<&'a mut dyn Any, TableError>, T)> {
        map.into_iter().map(move |(k, v)| {
            let item = self
                .map
                .get_mut(&k)
                .map(|e| Box::as_mut(&mut e.entry))
                // Safety: extending the lifetime of the mutable reference.
                .map(|item| unsafe { &mut *(item as *mut dyn Any) })
                .ok_or(TableError::NotPresent);
            (item, v)
        })
    }
}

impl Default for Table {
    fn default() -> Self {
        Table::new()
    }
}
