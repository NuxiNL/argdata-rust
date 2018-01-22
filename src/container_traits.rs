use std::borrow::Cow;
use std::cmp::min;
use std::rc::Rc;
use std::sync::Arc;

/// a random-access container.
pub trait Container {
	type Item;
	fn get(&self, index: usize) -> Option<&Self::Item>;
	fn len(&self) -> usize;
}

/// a random-access container containing pairs of keys and values.
///
/// Both a tuple of two Containers and a Container of tuples are considered MapContainers:
/// Examples are `(Vec<Key>, &[Val])` and `Box<[(Key, Val)]>`.
pub trait MapContainer {
	type Key;
	type Value;
	fn get(&self, index: usize) -> Option<(&Self::Key, &Self::Value)>;
	fn len(&self) -> usize;
}

impl<K, V> MapContainer for (K, V) where
	K: Container,
	V: Container,
{
	type Key = <K as Container>::Item;
	type Value = <V as Container>::Item;

	fn get(&self, index: usize) -> Option<(&Self::Key, &Self::Value)> {
		self.0.get(index).and_then(|k| self.1.get(index).map(|v| (k, v)))
	}

	fn len(&self) -> usize {
		min(self.0.len(), self.1.len())
	}
}

impl<T, K, V> MapContainer for T where
	T: Container<Item=(K, V)>,
{
	type Key = K;
	type Value = V;

	fn get(&self, index: usize) -> Option<(&Self::Key, &Self::Value)> {
		Container::get(self, index).map(|&(ref k, ref v)| (k, v))
	}

	fn len(&self) -> usize {
		Container::len(self)
	}
}

macro_rules! impl_container {
	(($($a:tt)*) $t:ty) => {
		impl<$($a)*> Container for $t {
			type Item = T;
			fn get(&self, index: usize) -> Option<&T> { self[..].get(index) }
			fn len(&self) -> usize { self[..].len() }
		}
	};
}

// TODO: Find a way to do this for all C: Deref<Target=[T]>, but not &C.
impl_container!((T) Vec<T>);
impl_container!((T) Box<[T]>);
impl_container!((T) Rc<[T]>);
impl_container!((T) Arc<[T]>);
impl_container!(('a, T: Clone) Cow<'a, [T]>);

impl_container!((T) [T]);
impl_container!((T) [T;  0]);
impl_container!((T) [T;  1]);
impl_container!((T) [T;  2]);
impl_container!((T) [T;  3]);
impl_container!((T) [T;  4]);
impl_container!((T) [T;  5]);
impl_container!((T) [T;  6]);
impl_container!((T) [T;  7]);
impl_container!((T) [T;  8]);
impl_container!((T) [T;  9]);
impl_container!((T) [T; 10]);
impl_container!((T) [T; 11]);
impl_container!((T) [T; 12]);
impl_container!((T) [T; 13]);
impl_container!((T) [T; 14]);
impl_container!((T) [T; 15]);
impl_container!((T) [T; 16]);
impl_container!((T) [T; 17]);
impl_container!((T) [T; 18]);
impl_container!((T) [T; 19]);
impl_container!((T) [T; 20]);
impl_container!((T) [T; 21]);
impl_container!((T) [T; 22]);
impl_container!((T) [T; 23]);
impl_container!((T) [T; 24]);
impl_container!((T) [T; 25]);
impl_container!((T) [T; 26]);
impl_container!((T) [T; 27]);
impl_container!((T) [T; 28]);
impl_container!((T) [T; 29]);
impl_container!((T) [T; 30]);
impl_container!((T) [T; 31]);
impl_container!((T) [T; 32]);
