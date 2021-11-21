use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Blackboard is a generalized data storage - properties of any type are assigned to keys/names.
///
/// Blackboards can be used as memory type for decision makers to generalize and simplify access to
/// memory, consider it as good entry level memory type - later when your AI will need more fine
/// tuning and optimizations, consider switching to plain struct data types for memory storage.
/// Blackboards gives slower access to the actual data comparing to plain struct types but makes
/// your life easier when just starting with AI development and you just want some general memory
/// type that can be shared between different agents expecting access to named properties of
/// any data type.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let mut memory = Blackboard::default();
///
/// memory.set("bool".to_owned(), true);
/// memory.set::<f32>("float".to_owned(), 42.0);
/// memory.set::<(usize, usize)>("tuple".to_owned(), (4, 2));
/// assert_eq!(memory.len(), 3);
/// assert_eq!(memory.has_property("bool"), true);
/// assert_eq!(memory.has_property_of_type::<(usize, usize)>("tuple"), true);
/// assert_eq!(memory.has_property("int"), false);
/// assert_eq!(memory.get::<bool>("bool"), Some(&true));
/// assert_eq!(memory.get::<f32>("float"), Some(&42.0));
/// assert_eq!(memory.get::<(usize, usize)>("tuple"), Some(&(4, 2)));
/// assert_eq!(
///     memory
///         .with("bool", |v| std::mem::replace(v, false))
///         .unwrap(),
///     true
/// );
/// assert_eq!(memory.get::<bool>("bool"), Some(&false));
/// assert!(memory.remove("bool"));
/// assert!(memory.remove("float"));
/// assert!(memory.remove("tuple"));
/// assert_eq!(memory.len(), 0);
/// ```
pub struct Blackboard {
    properties: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl Default for Blackboard {
    fn default() -> Self {
        Self {
            properties: Default::default(),
        }
    }
}

impl std::fmt::Debug for Blackboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Blackboard")
            .field("properties", &self.properties.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Blackboard {
    /// Returns number of properties stored in blackboard.
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Tells if there are no properties stored in blackboard.
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    /// Tells if there is a property under given name stored in blackboard.
    pub fn has_property(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }

    /// Tells if there is a property of given type under given name stored in blackboard.
    pub fn has_property_of_type<T>(&self, name: &str) -> bool
    where
        T: 'static,
    {
        if let Some(value) = self.properties.get(name) {
            return value.is::<T>();
        }
        false
    }

    /// Returns type ID of a property data stored in blackboard.
    pub fn type_id(&self, name: &str) -> Option<TypeId> {
        if let Some(value) = self.properties.get(name) {
            let v: &dyn Any = &*value;
            return Some(v.type_id());
        }
        None
    }

    /// Returns reference to property data.
    pub fn get<T>(&self, name: &str) -> Option<&T>
    where
        T: 'static,
    {
        if let Some(value) = self.properties.get(name) {
            return value.downcast_ref();
        }
        None
    }

    /// Returns mutable reference to property data.
    pub fn get_mut<T>(&mut self, name: &str) -> Option<&mut T>
    where
        T: 'static,
    {
        if let Some(value) = self.properties.get_mut(name) {
            return value.downcast_mut();
        }
        None
    }

    /// Returns reference to property data as [`Any`].
    pub fn raw(&self, name: &str) -> Option<&dyn Any> {
        if let Some(value) = self.properties.get(name) {
            return Some(&*value);
        }
        None
    }

    /// Returns mutable reference to property data as [`Any`].
    pub fn raw_mut(&mut self, name: &str) -> Option<&mut dyn Any> {
        if let Some(value) = self.properties.get_mut(name) {
            return Some(&mut *value);
        }
        None
    }

    /// Put value to property under given name.
    pub fn set<T>(&mut self, name: String, value: T)
    where
        T: Any + 'static + Send + Sync,
    {
        self.properties.insert(name, Box::new(value));
    }

    /// Put value to property under given name.
    pub fn set_raw(&mut self, name: String, value: Box<dyn Any + Send + Sync>) {
        self.properties.insert(name, value);
    }

    /// Mutate property data in-place with closure.
    pub fn with<T, R, F>(&mut self, name: &str, mut f: F) -> Option<R>
    where
        F: FnMut(&mut T) -> R,
        T: 'static,
    {
        if let Some(value) = self.properties.get_mut(name) {
            if let Some(value) = value.downcast_mut() {
                return Some(f(value));
            }
        }
        None
    }

    /// Remove property under given name.
    pub fn remove(&mut self, name: &str) -> bool {
        self.properties.remove(name).is_some()
    }

    /// Remove all properties in blackboard.
    pub fn clear(&mut self) {
        self.properties.clear();
    }

    /// Return iterator over properties keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.properties.keys().map(|k| k.as_str())
    }

    /// Return iterator over properties keys and type IDs.
    pub fn key_types(&self) -> impl Iterator<Item = (&str, TypeId)> {
        self.properties.iter().map(|(k, v)| {
            let v: &dyn Any = &*v;
            (k.as_str(), v.type_id())
        })
    }

    /// Return iterator over properties keys and their data as [`Any`].
    pub fn iter(&self) -> impl Iterator<Item = (&str, &dyn Any)> {
        self.properties.iter().map(|(k, v)| {
            let v: &dyn Any = &*v;
            (k.as_str(), v)
        })
    }
}
