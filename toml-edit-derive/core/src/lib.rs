pub use toml_edit::{
    Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, TableLike, TomlError, Value,
};
pub use toml_edit_derive_macro::TomlEdit;

/// Build an `Item::Value(Value::InlineTable(...))` by inserting each entry from `iter`.
/// Panics if any value is not an `Item::Value(...)` (inline tables only support scalar values).
pub fn inline_item(entries: impl IntoIterator<Item = (String, Item)>) -> Item {
    let mut inline = InlineTable::new();
    for (k, item) in entries {
        match item {
            Item::Value(v) => { inline.insert(&k, v); }
            _ => panic!("toml_edit_derive::inline_item: only Value items are supported in inline tables"),
        }
    }
    Item::Value(Value::InlineTable(inline))
}

pub trait TomlEditValue: Sized {
    fn from_value(v: &Value) -> Option<Self>;
    fn into_value(self) -> Value;
}

impl TomlEditValue for String {
    fn from_value(v: &Value) -> Option<Self> { v.as_str().map(|s| s.to_owned()) }
    fn into_value(self) -> Value { Value::from(self) }
}
impl TomlEditValue for bool {
    fn from_value(v: &Value) -> Option<Self> { v.as_bool() }
    fn into_value(self) -> Value { Value::from(self) }
}
impl TomlEditValue for f64 {
    fn from_value(v: &Value) -> Option<Self> { v.as_float() }
    fn into_value(self) -> Value { Value::from(self) }
}
impl TomlEditValue for f32 {
    fn from_value(v: &Value) -> Option<Self> { v.as_float().map(|n| n as f32) }
    fn into_value(self) -> Value { Value::from(self as f64) }
}
macro_rules! impl_toml_edit_value_int {
    ($($t:ty),*) => { $(impl TomlEditValue for $t {
        fn from_value(v: &Value) -> Option<Self> { v.as_integer().and_then(|n| <$t>::try_from(n).ok()) }
        fn into_value(self) -> Value { Value::from(i64::try_from(self).expect("out of i64 range")) }
    })* };
}
impl_toml_edit_value_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize);

use std::path::PathBuf;
impl TomlEditValue for PathBuf {
    fn from_value(v: &Value) -> Option<Self> { v.as_str().map(PathBuf::from) }
    fn into_value(self) -> Value { Value::from(self.to_string_lossy().into_owned()) }
}
use std::net::SocketAddr;
impl TomlEditValue for SocketAddr {
    fn from_value(v: &Value) -> Option<Self> { v.as_str().and_then(|s| s.parse().ok()) }
    fn into_value(self) -> Value { Value::from(self.to_string()) }
}

fn transfer_value_decor(old: &Item, new: &mut Item) {
    if let (Item::Value(old_val), Item::Value(new_val)) = (old, new) {
        *new_val.decor_mut() = old_val.decor().clone();
    }
}

use std::marker::PhantomData;

pub struct FieldEntry<'a, T: TomlEditable> {
    table: &'a mut dyn TableLike,
    key: &'static str,
    _phantom: PhantomData<fn() -> T>,
}
impl<'a, T: TomlEditable> FieldEntry<'a, T> {
    pub fn new(table: &'a mut dyn TableLike, key: &'static str) -> Self {
        Self { table, key, _phantom: PhantomData }
    }
    pub fn is_some(&self) -> bool { self.table.contains_key(self.key) }
    pub fn is_none(&self) -> bool { !self.is_some() }
    pub fn get(&self) -> Option<T::View<'_>> { self.table.get(self.key).and_then(T::from_item) }
    pub fn get_mut(&mut self) -> Option<T::Mut<'_>> { self.table.get_mut(self.key).and_then(T::from_item_mut) }
    pub fn set(&mut self, v: T) {
        let mut item = T::into_item(v);
        match self.table.get_mut(self.key) {
            Some(slot) => { transfer_value_decor(slot, &mut item); *slot = item; }
            None => { self.table.insert(self.key, item); }
        }
    }
    fn insert_if_absent(mut self, item: Item) -> T::Mut<'a> {
        if self.get_mut().is_none() {
            self.table.insert(self.key, item);
            debug_assert!(self.table.get_mut(self.key).and_then(T::from_item_mut).is_some(),
                "insert_if_absent: empty_item/into_item incompatible with from_item_mut");
        }
        self.get_mut_present()
    }
    pub fn get_or_insert(self, default: T) -> T::Mut<'a> { self.insert_if_absent(T::into_item(default)) }
    pub fn get_or_insert_with(self, f: impl FnOnce() -> T) -> T::Mut<'a> { self.insert_if_absent(T::into_item(f())) }
    pub fn get_or_insert_empty(self) -> T::Mut<'a> { self.insert_if_absent(T::empty_item()) }
    pub fn get_or_insert_default(self) -> T::Mut<'a> where T: Default { self.insert_if_absent(T::into_item(T::default())) }
    fn get_mut_present(self) -> T::Mut<'a> {
        self.table.get_mut(self.key).and_then(T::from_item_mut).expect("entry missing after insert")
    }
    #[track_caller]
    pub fn unwrap(self) -> T::Mut<'a> {
        self.table.get_mut(self.key).and_then(T::from_item_mut)
            .unwrap_or_else(|| panic!("unwrap() on missing key `{}`", self.key))
    }
    pub fn expect(self, msg: &str) -> T::Mut<'a> {
        self.table.get_mut(self.key).and_then(T::from_item_mut).unwrap_or_else(|| panic!("{}", msg))
    }
    pub fn remove(&mut self) -> bool { self.table.remove(self.key).is_some() }
    pub fn replace(self, v: T) -> T::Mut<'a> {
        let mut item = T::into_item(v);
        match self.table.get_mut(self.key) {
            Some(slot) => { transfer_value_decor(slot, &mut item); *slot = item; }
            None => { self.table.insert(self.key, item); }
        }
        self.get_mut_present()
    }
    pub fn and_mutate<F>(mut self, f: F) -> bool where F: FnOnce(&mut T::Mut<'_>) {
        if let Some(mut m) = self.get_mut() { f(&mut m); true } else { false }
    }
}
impl<T: TomlEditable> std::fmt::Debug for FieldEntry<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldEntry").finish()
    }
}

enum ValueOrItem<'a> {
    Value(&'a mut Value),
    Item(&'a mut Item),
}
pub struct TomlEditValueMut<'a, T: TomlEditValue> {
    inner: ValueOrItem<'a>,
    _marker: PhantomData<fn() -> T>,
}
impl<'a, T: TomlEditValue> TomlEditValueMut<'a, T> {
    pub fn new(item: &'a mut Item) -> Self {
        Self { inner: ValueOrItem::Item(item), _marker: PhantomData }
    }
    pub(crate) fn from_value(v: &'a mut Value) -> Self {
        Self { inner: ValueOrItem::Value(v), _marker: PhantomData }
    }
    pub fn get(&self) -> Option<T> {
        match &self.inner {
            ValueOrItem::Value(v) => T::from_value(v),
            ValueOrItem::Item(i) => i.as_value().and_then(|v| T::from_value(v)),
        }
    }
    pub fn set(&mut self, v: T) {
        let mut new_val = T::into_value(v);
        match &mut self.inner {
            ValueOrItem::Value(slot) => { *new_val.decor_mut() = slot.decor().clone(); **slot = new_val; }
            ValueOrItem::Item(slot) => {
                if let Item::Value(ref old_val) = **slot { *new_val.decor_mut() = old_val.decor().clone(); }
                **slot = Item::Value(new_val);
            }
        }
    }
}
impl<T: TomlEditValue> std::fmt::Debug for TomlEditValueMut<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TomlEditValueMut").finish()
    }
}

impl<T: TomlEditValue> TomlEditable for T {
    type View<'a> = T;
    type Mut<'a> = TomlEditValueMut<'a, T>;
    const IS_VALUE_TYPE: bool = true;
    fn from_value(v: &Value) -> Option<T> { <T as TomlEditValue>::from_value(v) }
    fn from_item(item: &Item) -> Option<T> { item.as_value().and_then(|v| <T as TomlEditValue>::from_value(v)) }
    fn from_item_mut(item: &mut Item) -> Option<TomlEditValueMut<'_, T>> { item.is_value().then(|| TomlEditValueMut::new(item)) }
    fn from_value_mut(v: &mut Value) -> Option<TomlEditValueMut<'_, T>> { Some(TomlEditValueMut::from_value(v)) }
    fn into_item(self) -> Item { Item::Value(T::into_value(self)) }
}

pub trait TomlEditable: Sized {
    type View<'a>;
    type Mut<'a>;
    const IS_VALUE_TYPE: bool = false;
    fn from_item(item: &Item) -> Option<Self::View<'_>> { item.as_table_like().and_then(Self::from_table_like) }
    fn from_table_like(table: &dyn TableLike) -> Option<Self::View<'_>> { let _ = table; None }
    fn from_value(_v: &Value) -> Option<Self::View<'_>> { None }
    fn from_item_mut(item: &mut Item) -> Option<Self::Mut<'_>> { item.as_table_like_mut().and_then(Self::from_table_like_mut) }
    fn from_table_like_mut(table: &mut dyn TableLike) -> Option<Self::Mut<'_>> { let _ = table; None }
    fn from_value_mut(_v: &mut Value) -> Option<Self::Mut<'_>> { None }
    fn into_item(self) -> Item;
    fn empty_item() -> Item { Item::Table(Table::new()) }
}

macro_rules! impl_toml_editable_map {
    ($map:ident) => {
        impl<T: TomlEditable + 'static> TomlEditable for std::collections::$map<String, T> {
            type View<'a> = std::collections::$map<String, T::View<'a>>;
            type Mut<'a> = TomlEditMapMut<'a, T>;
            fn from_item(item: &Item) -> Option<Self::View<'_>> { item.as_table_like().and_then(Self::from_table_like) }
            fn from_table_like(table: &dyn TableLike) -> Option<Self::View<'_>> {
                Some(table.iter().filter_map(|(k, i)| Some((k.to_owned(), T::from_item(i)?))).collect())
            }
            fn from_item_mut(item: &mut Item) -> Option<TomlEditMapMut<'_, T>> { item.as_table_like_mut().map(TomlEditMapMut::new) }
            fn into_item(self) -> Item {
                let mut __tbl = Table::new();
                for (k, v) in self { __tbl.insert(&k, T::into_item(v)); }
                Item::Table(__tbl)
            }
        }
    };
}
impl_toml_editable_map!(HashMap);
impl_toml_editable_map!(BTreeMap);

impl<T: TomlEditable + 'static> TomlEditable for Vec<T> {
    type View<'a> = Vec<T::View<'a>>;
    type Mut<'a> = TomlEditArrayMut<'a, T>;
    fn from_item(item: &Item) -> Option<Vec<T::View<'_>>> {
        if let Some(arr) = item.as_array() { Some(arr.iter().filter_map(T::from_value).collect()) }
        else { item.as_array_of_tables().map(|aot| aot.iter().filter_map(|t| T::from_table_like(t)).collect()) }
    }
    fn from_item_mut(item: &mut Item) -> Option<TomlEditArrayMut<'_, T>> {
        match item {
            Item::Value(Value::Array(arr)) => Some(TomlEditArrayMut::new_values(arr)),
            Item::ArrayOfTables(aot) => Some(TomlEditArrayMut::new(aot)),
            _ => None,
        }
    }
    fn empty_item() -> Item {
        if T::IS_VALUE_TYPE { Item::Value(Value::Array(Array::new())) }
        else { Item::ArrayOfTables(ArrayOfTables::new()) }
    }
    fn into_item(self) -> Item {
        let mut arr = Array::new();
        let mut aot = ArrayOfTables::new();
        let mut saw_values = false;
        let mut saw_tables = false;
        for v in self {
            match T::into_item(v) {
                Item::Value(val) => { saw_values = true; arr.push(val); }
                Item::Table(tbl) => { saw_tables = true; aot.push(tbl); }
                _ => {}
            }
        }
        assert!(!(saw_values && saw_tables), "Vec::into_item: mixed Value/Table");
        assert!(!(T::IS_VALUE_TYPE && saw_tables), "Vec::into_item: IS_VALUE_TYPE mismatch");
        if T::IS_VALUE_TYPE || saw_values { Item::Value(Value::Array(arr)) }
        else { Item::ArrayOfTables(aot) }
    }
}

enum ArrayMutStorage<'a> { Values(&'a mut Array), Tables(&'a mut ArrayOfTables) }

enum EitherIter<L: Iterator, R: Iterator<Item = L::Item>> { Left(L), Right(R) }
impl<L: Iterator, R: Iterator<Item = L::Item>> Iterator for EitherIter<L, R> {
    type Item = L::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self { Self::Left(l) => l.next(), Self::Right(r) => r.next() }
    }
}
impl<L, R> std::iter::FusedIterator for EitherIter<L, R>
where L: std::iter::FusedIterator, R: std::iter::FusedIterator<Item = L::Item> {}

pub struct TomlEditArrayMut<'a, T: TomlEditable> {
    storage: ArrayMutStorage<'a>,
    _marker: PhantomData<fn() -> T>,
}
impl<'a, T: TomlEditable> TomlEditArrayMut<'a, T> {
    pub fn new(arr: &'a mut ArrayOfTables) -> Self { Self { storage: ArrayMutStorage::Tables(arr), _marker: PhantomData } }
    fn new_values(arr: &'a mut Array) -> Self { Self { storage: ArrayMutStorage::Values(arr), _marker: PhantomData } }
    pub fn len(&self) -> usize { match &self.storage { ArrayMutStorage::Values(a) => a.len(), ArrayMutStorage::Tables(a) => a.len() } }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn get(&self, index: usize) -> Option<T::View<'_>> {
        match &self.storage {
            ArrayMutStorage::Values(a) => a.get(index).and_then(|v| T::from_value(v)),
            ArrayMutStorage::Tables(a) => a.get(index).and_then(|t| T::from_table_like(t)),
        }
    }
    pub fn get_mut(&mut self, index: usize) -> Option<T::Mut<'_>> {
        match &mut self.storage {
            ArrayMutStorage::Values(a) => a.get_mut(index).and_then(|v| T::from_value_mut(v)),
            ArrayMutStorage::Tables(a) => a.get_mut(index).and_then(|t| T::from_table_like_mut(t)),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = T::View<'_>> + '_ {
        match &self.storage {
            ArrayMutStorage::Values(a) => EitherIter::Left(a.iter().filter_map(|v| T::from_value(v))),
            ArrayMutStorage::Tables(a) => EitherIter::Right(a.iter().filter_map(|t| T::from_table_like(t))),
        }
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = T::Mut<'_>> + '_ {
        match &mut self.storage {
            ArrayMutStorage::Values(a) => EitherIter::Left(a.iter_mut().filter_map(|v| T::from_value_mut(v))),
            ArrayMutStorage::Tables(a) => EitherIter::Right(a.iter_mut().filter_map(|t| T::from_table_like_mut(t))),
        }
    }
    #[must_use]
    pub fn push_empty(&mut self) -> T::Mut<'_> {
        match &mut self.storage {
            ArrayMutStorage::Values(_) => panic!("push_empty on value-backed array"),
            ArrayMutStorage::Tables(a) => {
                a.push(Table::new());
                let last = a.len() - 1;
                T::from_table_like_mut(a.get_mut(last).unwrap()).expect("push_empty requires struct type")
            }
        }
    }
    pub fn remove(&mut self, index: usize) { match &mut self.storage { ArrayMutStorage::Values(a) => { a.remove(index); }, ArrayMutStorage::Tables(a) => { a.remove(index); } } }
    pub fn at(&mut self, index: usize) -> Option<T::Mut<'_>> { self.get_mut(index) }
    pub fn clear(&mut self) { match &mut self.storage { ArrayMutStorage::Values(a) => a.clear(), ArrayMutStorage::Tables(a) => a.clear() } }
}
impl<T: TomlEditable> std::fmt::Debug for TomlEditArrayMut<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("TomlEditArrayMut").finish() }
}

pub struct TomlEditMapMut<'a, T: TomlEditable> {
    table: &'a mut dyn TableLike,
    _marker: PhantomData<fn() -> T>,
}
impl<'a, T: TomlEditable> TomlEditMapMut<'a, T> {
    pub fn new(table: &'a mut dyn TableLike) -> Self { Self { table, _marker: PhantomData } }
    pub fn len(&self) -> usize { self.table.len() }
    pub fn is_empty(&self) -> bool { self.table.is_empty() }
    pub fn contains_key(&self, key: &str) -> bool { self.table.contains_key(key) }
    pub fn get(&self, key: &str) -> Option<T::View<'_>> { self.table.get(key).and_then(T::from_item) }
    pub fn get_mut(&mut self, key: &str) -> Option<T::Mut<'_>> { self.table.get_mut(key).and_then(T::from_item_mut) }
    pub fn keys(&self) -> impl Iterator<Item = &str> + '_ { self.table.iter().map(|(k, _)| k) }
    pub fn iter(&self) -> impl Iterator<Item = (&str, T::View<'_>)> + '_ {
        self.table.iter().filter_map(|(k, i)| Some((k, T::from_item(i)?)))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (String, T::Mut<'_>)> + '_ {
        self.table.iter_mut().filter_map(|(k, i)| Some((k.get().to_owned(), T::from_item_mut(i)?)))
    }
    #[must_use]
    pub fn insert_empty(&mut self, key: &str) -> T::Mut<'_> {
        self.table.insert(key, T::empty_item());
        T::from_item_mut(self.table.get_mut(key).unwrap())
            .expect("insert_empty: T::empty_item() incompatible with T::from_item_mut()")
    }
    pub fn remove(&mut self, key: &str) { self.table.remove(key); }
}
impl<T: TomlEditable> std::fmt::Debug for TomlEditMapMut<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("TomlEditMapMut").finish() }
}
