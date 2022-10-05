#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::{inherent::Vec, pallet_prelude::*};
use frame_system::pallet_prelude::*;

use codec::{Encode, Decode};
use pallet_timestamp::{self as timestamp};

/// Timestamp format is intended to be as seconds(s)
const HOUR_DURATION: u64 = 3_600;
/// Let list size be bound by 1000. This will prevent us from putting too much data
/// in blockchain storage.
const LIST_SIZE: u64 = 1000;

#[derive(Clone, Default, Encode, Decode, TypeInfo)]
pub struct OracleEvent {
	pub data: Vec<u8>,
	/// just seconds
	pub timestamp: u64,
}

/// List-like implementation.
#[derive(Encode, Decode, TypeInfo)]
pub struct OracleEventStorage {
	pub events: Vec<OracleEvent>,
	/// Current index of start
	pub start: u64,
	/// Current amount of valid events
	pub size: u64,
	/// Thought of doing some performance enhancement here.
	#[deprecated]
	pub last_timestamp: u64,
}

impl Default for OracleEventStorage {
	fn default() -> Self {
		Self::init_sized(LIST_SIZE)
	}
}
impl OracleEventStorage {
	pub fn init_sized(size: u64) -> Self {
		let mut v = Vec::new();
		v.reserve(size as usize);
		for i in 0..size {
			v.push(OracleEvent::default());
		}
		Self {
			events: v,
			start: 0,
			size: 0,
			last_timestamp: 0,
		}
	}
	pub fn cleanup(&mut self, now: u64) {
		let mut counter = self.get_start();
		let mut cleaned_start = counter;
		let vec_size = self.events.len();

		for i in 0..self.size {
			let ts = self.events[counter].timestamp;
			if now - ts > HOUR_DURATION {
				cleaned_start = (cleaned_start + 1) % vec_size;
				self.size = self.size - 1;
			} else {
				break;
			}
			counter = counter + 1;
		}
		self.start = cleaned_start as u64;
	}
	pub fn add_event(&mut self, event: OracleEvent, now: u64) {
		let put_idx = (self.get_start() + self.get_size()) % self.events.len();
		self.events[put_idx] = event;
		//self.idx = self.idx + 1;
		if self.size < self.events.len() as u64 {
			self.size = self.size + 1;
		}
	}
	pub fn get_start(&self) -> usize {
		self.start as usize
	}
	pub fn get_size(&self) -> usize {
		self.size as usize
	}
}

/// All-in-one to make it easier to run in IDE.
#[test]
fn test_storage() {
	let mut storage = OracleEventStorage::default();
	assert_eq!(1000, storage.events.len());

	let event = OracleEvent {
		data: vec![],
		timestamp: 1000,
	};
	for i in 0..50 {
		storage.add_event(event.clone(), 1000);
	}
	assert_eq!(50, storage.size);
	storage.cleanup(1000 + HOUR_DURATION + 1);
	assert_eq!(0, storage.size);
	assert_eq!(50, storage.start);

	let newer_event = OracleEvent {
		data: vec![],
		timestamp: 1100,
	};

	for i in 0..50 {
		storage.add_event(event.clone(), 1000);
	}
	assert_eq!(50, storage.size);
	for i in 0..50 {
		storage.add_event(newer_event.clone(), 1100);
	}
	assert_eq!(100, storage.size);
	storage.cleanup(1000 + HOUR_DURATION + 1);
	assert_eq!(100, storage.start);
	assert_eq!(50, storage.size);
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub type OracleEvent = Vec<u8>;
	pub type StoredEvents = Vec<Vec<u8>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::storage]
	#[pallet::getter(fn oracle_events)]
	pub type Events<T> = StorageValue<_, StoredEvents, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn foo)]
	pub type OracleEventStorageStorage<T> = StorageValue<_, OracleEventStorage, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn handle_event(
			origin: OriginFor<T>,
			event: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_root(origin)?;
			let mut events = <Events<T>>::get();
			let _now = <timestamp::Pallet<T>>::get(); /// actually duration = {secs, nanosecs}

			events.push(event);

			<Events<T>>::put(events);
			Ok(())
		}
	}
}
