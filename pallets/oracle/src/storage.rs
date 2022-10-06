use codec::{Decode, Encode};
use frame_support::{inherent::Vec, pallet_prelude::TypeInfo};
/// Timestamp format is intended to be as milliseconds(ms)
const HOUR_DURATION: u64 = 3_600_000;
/// Let list size be bound by 1000. This will prevent us from putting too much data
/// in blockchain storage.
const LIST_SIZE: u64 = 1000;

pub type RawEvent = Vec<u8>;

#[derive(Clone, Debug, PartialEq, PartialOrd, Default, Encode, Decode, TypeInfo)]
pub struct OracleEvent {
	/// just seconds
	pub timestamp: u64,
	pub data: RawEvent,
}

/// List-like implementation.
#[derive(Encode, Decode, TypeInfo)]
pub struct OracleEventStorage {
	pub events: Vec<OracleEvent>,
	/// Current index of start
	pub start: u64,
	/// Current amount of valid events
	pub size: u64,
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
		for _i in 0..size {
			v.push(OracleEvent::default());
		}
		Self { events: v, start: 0, size: 0 }
	}
	/// Remove all the outdated events.
	pub fn cleanup(&mut self, now: u64) {
		let mut counter = self.get_start();
		let mut cleaned_start = counter;
		let vec_size = self.events.len();

		for _i in 0..self.size {
			let ts = self.events[counter].timestamp;
			if now - ts > HOUR_DURATION {
				cleaned_start = (cleaned_start + 1) % vec_size;
				self.size = self.size - 1;
			} else {
				break
			}
			counter = counter + 1;
		}
		self.start = cleaned_start as u64;
	}
	/// Add a new event. Will rewrite the oldest one if there were more than 1000 events
	/// in the last hour.
	pub fn add_event(&mut self, event: Vec<u8>, now: u64) {
		let event = OracleEvent { timestamp: now, data: event };
		let put_idx = (self.get_start() + self.get_size()) % self.events.len();
		self.events[put_idx] = event;
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

	let event = vec![];
	for _i in 0..50 {
		storage.add_event(event.clone(), 1000);
	}
	assert_eq!(50, storage.size);
	storage.cleanup(1000 + HOUR_DURATION + 1);
	assert_eq!(0, storage.size);
	assert_eq!(50, storage.start);

	for _i in 0..50 {
		storage.add_event(event.clone(), 1000);
	}
	assert_eq!(50, storage.size);
	for _i in 0..50 {
		storage.add_event(event.clone(), 1100);
	}
	assert_eq!(100, storage.size);
	storage.cleanup(1000 + HOUR_DURATION + 1);
	assert_eq!(100, storage.start);
	assert_eq!(50, storage.size);
}
