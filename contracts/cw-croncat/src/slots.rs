use crate::CwCroncat;
use cosmwasm_std::{BlockInfo, Order, StdResult, Storage};
pub use cw_croncat_core::types::Interval;
use cw_croncat_core::types::SlotType;

impl<'a> CwCroncat<'a> {
    /// Get the slot with lowest height/timestamp
    /// Returns a tuple of optionals: (Option<block height>, Option<timestamp>)
    /// NOTE: This prioritizes blocks over timestamps.
    pub(crate) fn get_current_slot_items(
        &self,
        block: &BlockInfo,
        storage: &dyn Storage,
        limit: Option<usize>,
    ) -> (Option<u64>, Option<u64>) {
        let mut ret: (Option<u64>, Option<u64>) = (None, None);
        let block_height = block.height;

        let block_slot: StdResult<Vec<u64>> = if let Some(l) = limit {
            self.block_slots
                .keys(storage, None, None, Order::Ascending)
                .take(l)
                .collect()
        } else {
            self.block_slots
                .keys(storage, None, None, Order::Ascending)
                .collect()
        };

        if let Ok(Some(block_id)) = block_slot.map(|v| v.first().copied()) {
            if block_height >= block_id {
                ret.0 = Some(block_id);
            }
        }

        let timestamp: u64 = block.time.nanos();
        let time_slot: StdResult<Vec<u64>> = if let Some(l) = limit {
            self.time_slots
                .keys(storage, None, None, Order::Ascending)
                .take(l)
                .collect()
        } else {
            self.time_slots
                .keys(storage, None, None, Order::Ascending)
                .collect()
        };

        if let Ok(Some(time_id)) = time_slot.map(|v| v.first().copied()) {
            if timestamp >= time_id {
                ret.1 = Some(time_id);
            }
        }

        ret
    }

    /// Gets 1 slot hash item, and removes the hash from storage
    /// Cleans up a slot if empty
    pub(crate) fn pop_slot_item(
        &mut self,
        storage: &mut dyn Storage,
        slot: u64,
        kind: SlotType,
    ) -> StdResult<Option<Vec<u8>>> {
        let store = match kind {
            SlotType::Block => &self.block_slots,
            SlotType::Time => &self.time_slots,
        };

        let slot_data = store.may_load(storage, slot)?;

        // Get a single task hash, then retrieve task details
        if let Some(mut slots) = slot_data {
            let hash = slots.pop();

            // Need to remove this slot if no hash's left
            if slots.is_empty() {
                store.remove(storage, slot);
            } else {
                store.save(storage, slot, &slots)?;
            }
            Ok(hash)
        } else {
            Ok(None)
        }
    }
}
