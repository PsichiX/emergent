use std::collections::HashMap;

/// DataTable holds set of rows of given type assigned to keys/names.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// #[derive(Debug, Copy, Clone, PartialEq)]
/// enum Row {
///     Bool(bool),
///     Float(f32),
///     Tuple(usize, usize),
/// }
///
/// let mut memory = DataTable::<Row>::default();
///
/// memory.set("bool".to_owned(), Row::Bool(true));
/// memory.set("float".to_owned(), Row::Float(42.0));
/// memory.set("tuple".to_owned(), Row::Tuple(4, 2));
/// assert_eq!(memory.len(), 3);
/// assert_eq!(memory.has_row("bool"), true);
/// assert_eq!(memory.has_row("int"), false);
/// assert_eq!(memory.get("bool"), Some(&Row::Bool(true)));
/// assert_eq!(memory.get("float"), Some(&Row::Float(42.0)));
/// assert_eq!(memory.get("tuple"), Some(&Row::Tuple(4, 2)));
/// assert_eq!(
///     memory
///         .with("bool", |v| std::mem::replace(v, Row::Bool(false)))
///         .unwrap(),
///     Row::Bool(true),
/// );
/// assert_eq!(memory.get("bool"), Some(&Row::Bool(false)));
/// assert!(memory.remove("bool"));
/// assert!(memory.remove("float"));
/// assert!(memory.remove("tuple"));
/// assert_eq!(memory.len(), 0);
/// ```
pub struct DataTable<T> {
    rows: HashMap<String, T>,
}

impl<T> Default for DataTable<T> {
    fn default() -> Self {
        Self {
            rows: Default::default(),
        }
    }
}

impl<T> std::fmt::Debug for DataTable<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTable")
            .field("rows", &self.rows)
            .finish()
    }
}

impl<T> Clone for DataTable<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            rows: self.rows.clone(),
        }
    }
}

impl<T> DataTable<T> {
    /// Returns number of rows stored in datatable.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Tells if there are no rows stored in datatable.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Tells if there is a row under given name stored in datatable.
    pub fn has_row(&self, name: &str) -> bool {
        self.rows.contains_key(name)
    }

    /// Returns reference to row data.
    pub fn get(&self, name: &str) -> Option<&T> {
        self.rows.get(name)
    }

    /// Returns mutable reference to row data.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut T> {
        self.rows.get_mut(name)
    }

    /// Put value to row under given name.
    pub fn set(&mut self, name: String, value: T) {
        self.rows.insert(name, value);
    }

    /// Mutate row data in-place with closure.
    pub fn with<R, F>(&mut self, name: &str, mut f: F) -> Option<R>
    where
        F: FnMut(&mut T) -> R,
    {
        if let Some(value) = self.rows.get_mut(name) {
            return Some(f(value));
        }
        None
    }

    /// Remove row under given name.
    pub fn remove(&mut self, name: &str) -> bool {
        self.rows.remove(name).is_some()
    }

    /// Remove all rows in datatable.
    pub fn clear(&mut self) {
        self.rows.clear();
    }

    /// Return iterator over rows keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.rows.keys().map(|k| k.as_str())
    }

    /// Return iterator over rows values.
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.rows.values()
    }

    /// Return iterator over rows keys and their values.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.rows.iter().map(|(k, v)| (k.as_str(), v))
    }
}
