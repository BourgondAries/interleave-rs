#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
//! This crate allows you to create an arbitrary interleaving
//! iterator. Each iterator is guaranteed to be behind the
//! most advanced iterator by at max one next() call.
//! The interleave macro is of arbitrary arity.
//!
//! ```
//! #[macro_use]
//! extern crate interleave;
//! fn main() {
//! 	use interleave::{IterList, MultiIter};
//! 	let iter = interleave!(1..5, 9..12, -3..2);
//! 	for i in iter {
//! 		println!("{:?}", i);
//! 	}
//! }
//! ```
//!
//! The types returned by the iterator can also be forced:
//!
//! ```
//! #[macro_use]
//! extern crate interleave;
//! fn main() {
//! 	use interleave::{IterList, MultiIter};
//! 	let iter = interleave!(i8; 1..5, 9..12, -3..2);
//! 	for i in iter {
//! 		println!("{:?}", i);
//! 	}
//! }
//! ```
//!
//! Most information can be found in the examples or the test module.
//!
#[allow(dead_code)]
pub type Iter<T> = Box<Iterator<Item = T>>;

/// Vector of boxed iterator traits
pub type IterList<T> = Vec<Iter<T>>;

/// Holds the state of the interleave iterator
pub struct MultiIter<T> {
	empty: bool,
	index: usize,
	items: IterList<T>,
}

impl<T> MultiIter<T> {
	pub fn new(items: IterList<T>) -> MultiIter<T> {
		MultiIter {
			empty: false,
			index: 0,
			items: items,
		}
	}

	/// Add a new iterator to stack of iterables.
	///
	/// Should only be used when setting up, does not
	/// reset when the other iterators have been exhausted.
	pub fn push(&mut self, item: Iter<T>) {
		self.items.push(item);
	}
}

impl<T> Default for MultiIter<T> {
	fn default() -> MultiIter<T> {
		MultiIter {
			empty: false,
			index: 0,
			items: vec![],
		}
	}
}

impl<T> Iterator for MultiIter<T> {
	type Item = T;
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(iterator) = self.items.get_mut(self.index) {
				if let Some(value) = iterator.next() {
					self.empty = false;
					self.index += 1;
					return Some(value);
				} else {
					self.index += 1;
				}
			} else {
				self.index = 0;
				if self.empty {
					return None;
				} else {
					self.empty = true;
				}
			}
		}
	}
}

/// Main macro for creating a MultiIter
#[macro_export]
macro_rules! interleave {
	($($e:expr),+,) => ({ interleave!($($e),*) });
	($($e:expr),+) => ({
		let mut temporary: IterList<_> = vec![];
		$(
			temporary.push(Box::new($e));
		);*
		MultiIter::new(temporary)
	});
	() => ( MultiIter::new(IterList::<_>::default()) );
	($t:ty;) => ( MultiIter::new(IterList::<$t>::default()) );
	($t:ty; $($e:expr),+,) => ( interleave!($t; $($e),*) );
	($t:ty; $($e:expr),+) => ({
		let mut temporary: IterList<$t> = vec![];
		$(
			temporary.push(Box::new($e));
		)*
		MultiIter::new(temporary)
	});
}

#[cfg(test)]
mod tests {
	use super::{IterList, MultiIter};

	macro_rules! next {
		($e:expr; $($n:expr),*,) => ( next!($e; $($n),*) );
		($e:expr; $($n:expr),*) => ( $(assert_eq!($e.next(), Some($n)));* );
	}

	#[test]
	fn simple() {
		let mut iter = interleave!(i32; 0..10);
		assert_eq!(iter.next(), Some(0));
		assert_eq!(iter.next(), Some(1));
	}

	#[test]
	fn simple_infer() {
		let mut iter = interleave!(0..10);
		assert_eq!(iter.next(), Some(0));
		assert_eq!(iter.next(), Some(1));
	}

	#[test]
	fn di_iter() {
		let mut iter = interleave!(i64; 0..10, 5..15);
		assert_eq!(iter.next(), Some(0));
		assert_eq!(iter.next(), Some(5));
		assert_eq!(iter.next(), Some(1));
		assert_eq!(iter.next(), Some(6));
	}

	#[test]
	fn di_iter_infer() {
		let mut iter = interleave!(0i64..10, 5..15);
		assert_eq!(iter.next(), Some(0));
		assert_eq!(iter.next(), Some(5));
		assert_eq!(iter.next(), Some(1));
		assert_eq!(iter.next(), Some(6));
	}

	#[test]
	fn tri_iter() {
		let mut iter = interleave!(0.., 0.., 0..);
		next!(iter; 0, 0, 0, 1, 1, 1, 2, 2, 2);
	}

	#[test]
	fn quad_iter() {
		let mut iter = interleave!{
			(0..3).map(|x| (0, x)),
			(0..3).map(|x| (x, 0)),
		};
		next!(iter;
			(0, 0),
			(0, 0),
			(0, 1),
			(1, 0),
			(0, 2),
			(2, 0),
		);
	}

	#[test]
	fn diff_len() {
		fn check(mut iter: MultiIter<i32>) {
			next!(iter; 0,0,0,0, 1,1,1,1, 2,2,2, 3,3,3, 4,4,4, 5,5, 6,6, 7, 8, 9);
		}
		check(interleave!((0..10), (0..5), (0..2), (0..7)));
		check(interleave!((0..5), (0..2), (0..7), (0..10)));
		check(interleave!((0..5), (0..7), (0..2), (0..10)));
	}
}
