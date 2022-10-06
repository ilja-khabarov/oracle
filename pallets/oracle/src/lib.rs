#![cfg_attr(not(feature = "std"), no_std)]

mod storage;

pub use pallet::*;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

use pallet_timestamp::{self as timestamp};

use storage::{OracleEventStorage, RawEvent};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

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
	#[pallet::getter(fn event_storage)]
	pub type OracleEventStorageStorage<T> = StorageValue<_, OracleEventStorage, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		ReceivedEvent(u64),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// WIP: temporary solution
		SomeError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn handle_event(
			origin: OriginFor<T>,
			event: RawEvent,
		) -> DispatchResult {
			ensure_root(origin)?;
			let mut events = <OracleEventStorageStorage<T>>::get();

			let now = <timestamp::Pallet<T>>::get();
			let now = TryInto::<u64>::try_into(now)
				.map_err(|_| Error::<T>::SomeError)?;
			// Finding out a way to create a daemon in Substrate will take too much time,
			// so I decided to keep this synchronous approach.
			events.cleanup(now);

			events.add_event(event, now);
			Self::deposit_event(Event::ReceivedEvent(now));

			<OracleEventStorageStorage<T>>::put(events);
			Ok(())
		}
	}
}
