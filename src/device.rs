// TODO: Better documentation. The documentation is very badly written at the moment because I was
// trying to get the point across. Needs rewording and examples.

use crate::bus::DeviceBus;
use crate::call::Call;
use crate::error::Result;
use crate::types::{Direction, ImportFileInfo, RobotActionResult, RotationDirection};
use erased_serde::Serialize as ErasedSerialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

// TODO: Turn this into a procedural macro
macro_rules! interface {
    (
        $(#[$outer:meta])*
        $vis:vis trait $trait_name:ident {
            $(const $assoc_const:ident : $assoc_ty:ty;)*

            $(
                $(#[$inner:meta])*
                fn $fn_name:ident $(<$($generic:ident),+ $(,)?>)? (&self $(, $($param_name:ident : $param_ty:ty),* $(,)?)?) $(-> $ret_ty:ty)?;
            )*
        }
    ) => {

        $(#[$outer])*
        $vis trait $trait_name: $crate::device::RpcDevice {
            $(
                const $assoc_const: $assoc_ty;
            )*

            $(
                $(#[$inner])*
                #[allow(unused_parens)]
                fn $fn_name$(<$($generic),+>)?(&self, $($($param_name: $param_ty),*)?) -> $crate::error::Result<($($ret_ty)?)>
                    where ($($ret_ty)?): ::serde::de::DeserializeOwned + 'static;
            )*
        }
    };
}

// TODO: Turn this into a procedural macro
#[macro_export]
macro_rules! device {
    (
        #[device(identifier = $identifier:literal)]
        $(#[$doc:meta])*
        $vis:vis struct $device_name:ident;

        $(
            impl $trait_name:ident {
                $(
                    #[device(invoke = $invoke_name:literal)]
                    fn $fn_name:ident $(<$($generic:ident),+ $(,)?>)? (&self $(, $($param_name:ident : $param_ty:ty),* $(,)?)?) $(-> $ret_ty:ty)?;
                )+
            }
        )+

    ) => {
        $(#[$doc])*
        $vis struct $device_name(::uuid::Uuid, $crate::bus::DeviceBus);

        impl $crate::device::RpcDevice for $device_name {
            const IDENTIFIER: &'static ::core::primitive::str = $identifier;

            fn new(id: ::uuid::Uuid, bus: &$crate::bus::DeviceBus) -> Self {
                Self(id, bus.clone())
            }

            fn id(&self) -> ::uuid::Uuid {
                self.0
            }

            fn bus(&self) -> &$crate::bus::DeviceBus {
                &self.1
            }
        }

        $(
            impl $trait_name for $device_name {
                $(
                    #[allow(non_snake_case)]
                    #[allow(unused_parens)]
                    fn $fn_name$(<$($generic),+>)?(&self, $($($param_name: $param_ty),*)?) -> $crate::error::Result<($($ret_ty)?)>
                        where ($($ret_ty)?): ::serde::de::DeserializeOwned + 'static
                    {
                        $crate::device::invoke(self.0, &self.1, $invoke_name, &[$($(&$param_name as &dyn ::erased_serde::Serialize),*)?])
                    }
                )+
            }
        )+
    };
}

// This function is simply here to cut down on possible code duplication, since each
// call which shares the same return type can share the same monomorphized version of this
// function.
#[doc(hidden)]
#[inline(never)]
pub fn invoke<R: DeserializeOwned + 'static>(
    id: Uuid,
    bus: &DeviceBus,
    method_name: &str,
    params: &[&dyn ErasedSerialize],
) -> Result<R> {
    let call = Call::invoke(id, method_name, params);

    bus.call(call).map(|r| r.0)
}

pub trait RpcDevice {
    const IDENTIFIER: &'static str;

    fn new(id: Uuid, bus: &DeviceBus) -> Self;
    fn id(&self) -> Uuid;
    fn bus(&self) -> &DeviceBus;
}

interface! {
    /// An interface between an energy storage device and the HLAPI.
    pub trait EnergyStorageInterface {
        /// Retrieves the current amount of energy stored in FE.
        fn get_energy_stored(&self) -> i32;

        /// Retrieves the maximum possible energy that can be stored in the device in FE.
        fn get_max_energy_stored(&self) -> i32;

        /// Returns a boolean indicating whether the storage device can have energy extracted from
        /// it.
        fn can_extract_energy(&self) -> bool;

        /// Returns a boolean indicating whether the storage device can receive energy.
        fn can_receive_energy(&self) -> bool;
    }
}

interface! {
    /// An interface between item storage devices or blocks and the HLAPI.
    pub trait ItemHandlerInterface {
        /// Returns a signed 32-bit integer that represents the slots in the storage
        /// block.
        fn get_item_slot_count(&self) -> i32;

        /// Returns a signed 32-bit integer that represents how many items can be stored in a single
        /// slot in the storage block.
        fn get_item_slot_limit(&self, slot: i32) -> i32;

        /// Returns a type which can be deserialized from JSON which represents the current
        /// Minecraft IItemStack in the specified slot.
        fn get_item_stack_in_slot<T>(&self, slot: i32) -> T;
    }
}

interface! {
    /// An interface between redstone signal sending and receiving devices and the HLAPI
    pub trait RedstoneInterface {
        /// Returns a signed 32-bit integer that represents the strength of the redstone input on
        /// the provided side. The integer will be in the range \[0, 15\].
        fn get_redstone_input(&self, side: Direction) -> i32;

        /// Returns a signed 32-bit integer that represents the strength of the redstone output on
        /// the provided side. The integer will be in the range \[0, 15\].
        fn get_redstone_output(&self, side: Direction) -> i32;

        /// Sets the redstone output strength on the given side to the provided value. Valid values
        /// for redstone strength are in the range \[0, 15\]
        fn set_redstone_output(&self, side: Direction, val: i32);
    }
}

interface! {
    /// An interface between sound-making devices and the HALPI
    pub trait SoundInterface {
        /// Returns a slice of sound effect names matching the given name. The length of the slice
        /// is limited, so very generic names may result in a truncated list.
        fn find_sound(&self, name: &str) -> Box<[String]>;

        /// Plays the sound effect matching the given name at the given volume and pitch.
        fn play_sound(&self, name: &str, volume: f64, pitch: f64);
    }
}

interface! {
    /// An interface for transferring files between a user's real computer and the HLAPI
    pub trait FileImportExportInterface {
        /// Requests the start of a file import, returning true if a file can be imported.
        fn request_import_file(&self) -> bool;

        /// Prompts the player to select a file to import, returning the information for the file
        /// requested. This method should be called in a loop, as it may return None before
        /// eventually returning the file information.
        fn begin_import_file(&self) -> Option<ImportFileInfo>;

        /// Reads a portion of the file currently being imported. Returns a slice of bytes
        /// containing the portion of the file that has been read. Returns None when the file has
        /// been fully imported. If the byte slice is empty, it means that the device is not ready
        /// to import the file.
        fn read_import_file(&self) -> Option<Vec<u8>>;

        /// Prompts the user to select a path where a file with the given name should be exported.
        fn begin_export_file(&self, name: &str);

        /// Writes some data to the current file being exported.
        fn write_export_file(&self, data: &[u8]);

        /// Finishes the currently running file export operation.
        fn finish_export_file(&self);

        /// Resets the device's state, preparing it for another file import or export operation.
        fn reset(&self);
    }
}

interface! {
    /// An interface between devices which carry out block operations and the HLAPI
    pub trait BlockOperationsInterface {
        /// Mines the adjacent block on the given side. Returns true if the block was able to be
        /// mined.
        fn excavate(&self, side: Direction) -> bool;

        /// Places a block on the given side. Returns true if the block was able to be placed.
        fn place(&self, side: Direction) -> bool;

        /// Returns a 32-bit signed integer that represents the durability of the currently active
        /// tool
        fn durability(&self) -> i32;

        /// Attempts to repair the currently active tool, returning true if the tool was repaired.
        /// If the tool is at full durability, this will always return false.
        fn repair(&self) -> bool;
    }
}

interface! {
    /// An interface between devices which carry out robot inventory operations and the HLAPI
    pub trait InventoryOperationsInterface {
        /// Attempts to move the given number of items from one robot inventory slot into another
        /// slot.
        fn move_item(&self, from: i32, into: i32, count: i32);

        /// Attempts to drop the given number of items in the currently active slot into either the
        /// world or an adjacent inventory on the given side. Returns the amount of items dropped
        fn drop_item(&self, count: i32, side: Direction) -> i32;

        /// Attempts to drop the given number of items in the currently active slot into the given
        /// slot in the adjacent inventory in the given direction. Returns the amount of items
        /// dropped.
        fn drop_item_into(&self, into: i32, count: i32, side: Direction) -> i32;

        /// Attempts to take the given number of items from either the world or an adjacent inventory
        /// on the given side. Returns the amount of items taken.
        fn take_item(&self, count: i32, side: Direction) -> i32;

        /// Attempts to take the given number of items from the given slot in the adjacent inventory
        /// in the given direction. Returns the amount of items taken.
        fn take_item_from(&self, from: i32, count: i32, side: Direction) -> i32;
    }
}

interface! {
    /// An interface between robots and the HLAPI.
    pub trait RobotInterface {
        /// Returns the amount of FE stored in a robot.
        fn get_energy_stored(&self) -> i32;

        /// Returns the maximum possible energy that a robot can store.
        fn get_energy_capacity(&self) -> i32;

        /// Returns the index of the currently active slot.
        fn get_selected_slot(&self) -> i32;

        /// Sets the currently active slot to the given index.
        fn set_selected_slot(&self, slot: i32);

        /// Returns information about the item in the given slot.
        fn get_stack_in_slot<T>(&self, slot: i32) -> T;

        /// Attempts to queue an action which moves the robot in the given direction. Returns true
        /// if the action was successfully queued.
        fn queue_move(&self, direction: Direction) -> bool;

        /// Attempts to queue an action which turns the robot in the given direction. Returns true
        /// if the action was successfully queued.
        fn queue_turn(&self, direction: RotationDirection) -> bool;

        /// Returns the ID of the previously performed action.
        fn get_last_action_id(&self) -> i32;

        /// Returns the currently queued number of actions.
        fn get_queued_action_count(&self) -> i32;

        /// Returns the state of a robot's action with a given ID.
        fn get_action_result(&self, id: i32) -> RobotActionResult;
    }
}

device! {
    #[device(identifier = "redstone")]
    /// A device that can interact with redstone in the world.
    pub struct RedstoneDevice;

    impl RedstoneInterface {
        #[device(invoke = "getRedstoneInput")]
        fn get_redstone_input(&self, side: Direction) -> i32;

        #[device(invoke = "getRedstoneOutput")]
        fn get_redstone_output(&self, side: Direction) -> i32;

        #[device(invoke = "setRedstoneOutput")]
        fn set_redstone_output(&self, side: Direction, val: i32);
    }
}

device! {
    #[device(identifier = "sound")]
    /// A device that allows a computer or robot to play sounds.
    pub struct SoundCard;

    impl SoundInterface {
        #[device(invoke = "findSound")]
        fn find_sound(&self, name: &str) -> Box<[String]>;

        #[device(invoke = "playSound")]
        fn play_sound(&self, name: &str, volume: f64, pitch: f64);
    }
}

device! {
    #[device(identifier = "file_import_export")]
    /// A device that allows importing and exporting of files from the player's computer.
    pub struct FileImportExportCard;

    impl FileImportExportInterface {
        #[device(invoke = "requestImportFile")]
        fn request_import_file(&self) -> bool;

        #[device(invoke = "beginImportFile")]
        fn begin_import_file(&self) -> Option<ImportFileInfo>;

        #[device(invoke = "readImportFile")]
        fn read_import_file(&self) -> Option<Vec<u8>>;

        #[device(invoke = "beginExportFile")]
        fn begin_export_file(&self, name: &str);

        #[device(invoke = "writeExportFile")]
        fn write_export_file(&self, data: &[u8]);

        #[device(invoke = "finishExportFile")]
        fn finish_export_file(&self);

        #[device(invoke = "reset")]
        fn reset(&self);
    }
}

device! {
    #[device(identifier = "inventory_operations")]
    /// A module that allows interaction with inventories in the the world.
    pub struct InventoryOperationsModule;

    impl InventoryOperationsInterface {
        #[device(invoke = "move")]
        fn move_item(&self, from: i32, into: i32, count: i32);

        #[device(invoke = "drop")]
        fn drop_item(&self, count: i32, side: Direction) -> i32;

        #[device(invoke = "dropInto")]
        fn drop_item_into(&self, into: i32, count: i32, side: Direction) -> i32;

        #[device(invoke = "take")]
        fn take_item(&self, count: i32, side: Direction) -> i32;

        #[device(invoke = "takeFrom")]
        fn take_item_from(&self, from: i32, count: i32, side: Direction) -> i32;
    }
}

device! {
    #[device(identifier = "block_operations")]
    /// A module that allows interaction with blocks in the world.
    pub struct BlockOperationsModule;

    impl BlockOperationsInterface {
        #[device(invoke = "excavate")]
        fn excavate(&self, side: Direction) -> bool;

        #[device(invoke = "place")]
        fn place(&self, side: Direction) -> bool;

        #[device(invoke = "durability")]
        fn durability(&self) -> i32;

        #[device(invoke = "repair")]
        fn repair(&self) -> bool;
    }
}

device! {
    #[device(identifier = "robot")]
    pub struct RobotDevice;

    impl RobotInterface {
        #[device(invoke = "getEnergyStored")]
        fn get_energy_stored(&self) -> i32;

        #[device(invoke = "getEnergyCapacity")]
        fn get_energy_capacity(&self) -> i32;

        #[device(invoke = "getSelectedSlot")]
        fn get_selected_slot(&self) -> i32;

        #[device(invoke = "setSelectedSlot")]
        fn set_selected_slot(&self, slot: i32);

        #[device(invoke = "getStackInSlot")]
        fn get_stack_in_slot<T>(&self, slot: i32) -> T;

        #[device(invoke = "move")]
        fn queue_move(&self, direction: Direction) -> bool;

        #[device(invoke = "turn")]
        fn queue_turn(&self, direction: RotationDirection) -> bool;

        #[device(invoke = "getLastActionId")]
        fn get_last_action_id(&self) -> i32;

        #[device(invoke = "getQueuedActionCount")]
        fn get_queued_action_count(&self) -> i32;

        #[device(invoke = "getActionResult")]
        fn get_action_result(&self, id: i32) -> RobotActionResult;
    }
}
