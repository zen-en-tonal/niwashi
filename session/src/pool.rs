pub trait Store<K, V> {
    fn get(&self, key: &K) -> Option<V>;
}
