#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
mod storage;
#[cfg(test)]
mod tests;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use pallet::*;

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

	#[pallet::storage]
	#[pallet::getter(fn oracle_account)]
	pub type OracleAccount<T: Config> = StorageValue<_, <T>::AccountId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ReceivedEvent(u64),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Not an authorized user.
		WrongOrigin,
		SystemError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn handle_event(origin: OriginFor<T>, event: RawEvent) -> DispatchResult {
			// "Only a single authorised account may post an event". Let's assume it's Root.
			//ensure_root(origin)?;
			let who = ensure_signed(origin)?;
			let authorized_origin = <OracleAccount<T>>::get().unwrap();
			if who != authorized_origin {
				Err(Error::<T>::WrongOrigin)?
			}

			let mut events = <OracleEventStorageStorage<T>>::get();

			let now = <timestamp::Pallet<T>>::get();
			let now = TryInto::<u64>::try_into(now).map_err(|_| Error::<T>::SystemError)?;
			// Finding out a way to create a daemon in Substrate will take too much time,
			// so I decided to keep this synchronous approach.
			events.cleanup(now);

			events.add_event(event, now);
			Self::deposit_event(Event::ReceivedEvent(now));

			<OracleEventStorageStorage<T>>::put(events);
			Ok(())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn authorize(origin: OriginFor<T>, account: <T>::AccountId) -> DispatchResult {
			// "Only a single authorised account may post an event". Let's assume it's Root.
			ensure_root(origin)?;
			<OracleAccount<T>>::put(account);
			Ok(())
		}
	}
}
