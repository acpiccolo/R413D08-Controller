//! Defines data structures, constants, and protocol logic for interacting
//! with an 8-Channel Multifunction RS485 Module via Modbus RTU, based on
//! the protocol specification document.
//!
//! This module covers:
//! - Representing port states ([`PortState`], [`PortStates`]).
//! - Identifying specific ports ([`Port`]) and the Modbus device address ([`Address`]).
//! - Defining Modbus register addresses and data values for reading states and controlling ports.
//! - Encoding and decoding values from/to Modbus register format ([`Word`]).
//! - Error handling for invalid port or address values.
//!
//! Assumes standard Modbus function codes "Read Holding Registers" (0x03) and
//! "Write Single Register" (0x06) are used externally to interact with the device.

use thiserror::Error;

/// Represents a single 16-bit value stored in a Modbus register.
///
/// Modbus RTU typically operates on 16-bit registers.
pub type Word = u16;

/// The total number of controllable digital I/O ports on the module.
pub const NUMBER_OF_PORTS: usize = 8;

/// Represents the state of a single digital I/O port (e.g., relay).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PortState {
    /// The port is inactive/relay OFF.
    Close,
    /// The port is active/relay ON.
    Open,
}

impl PortState {
    /// Decodes a [`PortState`] from a single Modbus holding register value (`Word`).
    ///
    /// # Arguments
    ///
    /// * `word`: The [`Word`] (u16) value read from the Modbus register for this port.
    ///
    /// # Returns
    ///
    /// The corresponding [`PortState`].
    ///
    /// # Example
    /// ```
    /// # use r413d08_lib::protocol::PortState;
    /// assert_eq!(PortState::decode_from_holding_registers(0x0000), PortState::Close);
    /// assert_eq!(PortState::decode_from_holding_registers(0x0001), PortState::Open);
    /// assert_eq!(PortState::decode_from_holding_registers(0xFFFF), PortState::Open);
    /// ```
    pub fn decode_from_holding_registers(word: Word) -> Self {
        // According to the device protocol document:
        // - `0x0000` represents [`PortState::Close`].
        // - `0x0001` (and likely any non-zero value) represents [`PortState::Open`].
        if word != 0 { Self::Open } else { Self::Close }
    }
}

/// Provides a human-readable string representation ("close" or "open").
impl std::fmt::Display for PortState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Close => write!(f, "close"),
            Self::Open => write!(f, "open"),
        }
    }
}

/// Represents the collective states of all [`NUMBER_OF_PORTS`] ports.
///
/// This struct holds an array of [`PortState`] and provides constants
/// for reading all port states using a single Modbus "Read Holding Registers" (0x03) command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PortStates([PortState; NUMBER_OF_PORTS]);

impl PortStates {
    /// The Modbus function code 0x03 (Read Holding Registers) register address used for reading all port states sequentially.
    pub const ADDRESS: u16 = 0x0001;
    /// The number of consecutive Modbus registers (`Word`s) to read to get all port states.
    ///
    /// This corresponds to [`NUMBER_OF_PORTS`].
    pub const QUANTITY: u16 = NUMBER_OF_PORTS as u16;

    /// Decodes the states of all ports from a slice of Modbus holding register values.
    ///
    /// Expects `words` to contain [`NUMBER_OF_PORTS`] values. Each word in the slice
    /// corresponds to the state of a single port, decoded via [`PortState::decode_from_holding_registers`].
    ///
    /// If `words` contains fewer than [`NUMBER_OF_PORTS`] items, the remaining
    /// port states in the returned struct will retain their default initialized value (`PortState::Close`).
    /// Extra items in `words` beyond [`NUMBER_OF_PORTS`] are ignored.
    ///
    /// # Arguments
    ///
    /// * `words`: A slice of [`Word`] containing the register values read from the device.
    ///
    /// # Returns
    ///
    /// A [`PortStates`] struct containing the decoded state for each port.
    ///
    /// # Example
    /// ```
    /// # use r413d08_lib::protocol::{PortState, PortStates, Word, NUMBER_OF_PORTS};
    /// // Example data mimicking a Modbus read response for 8 registers
    /// let modbus_data: [Word; NUMBER_OF_PORTS] = [0x1, 0x0, 0xFFFF, 0x0, 0x0, 0x0, 0x1234, 0x0];
    /// let decoded_states = PortStates::decode_from_holding_registers(&modbus_data);
    /// assert_eq!(decoded_states.as_array()[0], PortState::Open);
    /// assert_eq!(decoded_states.as_array()[1], PortState::Close);
    /// assert_eq!(decoded_states.as_array()[2], PortState::Open); // Non-zero treated as Open
    /// // ... and so on for all ports
    /// ```
    pub fn decode_from_holding_registers(words: &[Word]) -> Self {
        let mut port_states = [PortState::Close; NUMBER_OF_PORTS];
        // Iterate over the words read, up to the number of ports we have storage for.
        for (i, word) in words.iter().enumerate().take(NUMBER_OF_PORTS) {
            port_states[i] = PortState::decode_from_holding_registers(*word);
        }
        Self(port_states)
    }

    /// Returns an iterator over the individual [`PortState`] values in the order of the ports.
    pub fn iter(&self) -> std::slice::Iter<'_, PortState> {
        self.0.iter()
    }

    /// Provides direct access to the underlying array of port states.
    pub fn as_array(&self) -> &[PortState; NUMBER_OF_PORTS] {
        &self.0
    }
}

/// Provides a comma-separated string representation of all port states (e.g., "close, open, close, ...").
impl std::fmt::Display for PortStates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for state in self.iter() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", state)?;
            first = false;
        }
        Ok(())
    }
}

/// Represents a validated port index, guaranteed to be within the valid range `0..NUMBER_OF_PORTS`.
///
/// Use [`Port::try_from`] to create an instance from a `u8`.
/// This struct also defines constants for the *data values* used when controlling a specific port via
/// Modbus "Write Single Register" (function code 0x06). The register *address*
/// corresponds to the 1-based port index (see [`Port::address_for_write_register`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Port(u8); // Internally stores 0-based index

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Port {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Port::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl Port {
    /// The minimum valid port index (inclusive).
    pub const MIN: u8 = 0;
    /// The maximum valid port index (inclusive).
    pub const MAX: u8 = NUMBER_OF_PORTS as u8 - 1;

    // --- Modbus Register Data Values for Port Control ---
    // These constant `Word` values are written to the specific port's Modbus register address
    // using Modbus function code 0x06 (Write Single Register).

    /// Register data value to **open** the specified port (turn relay ON / activate output).
    pub const REG_DATA_SET_PORT_OPEN: Word = 0x0100;
    /// Register data value to **close** the specified port (turn relay OFF / deactivate output).
    pub const REG_DATA_SET_PORT_CLOSE: Word = 0x0200;
    /// Register data value to **toggle** the state of the specified port (Open <-> Close). Also called "Self-locking".
    pub const REG_DATA_SET_PORT_TOGGLE: Word = 0x0300;
    /// Register data value to **latch** the specified port (set this port Open, set all others Close). Also called "Inter-locking".
    pub const REG_DATA_SET_PORT_LATCH: Word = 0x0400;
    /// Register data value to activate the specified port **momentarily** (Open for ~1 second, then automatically Close). Also called "Non-locking".
    pub const REG_DATA_SET_PORT_MOMENTARY: Word = 0x0500;
    /// Base register data value to initiate a **delayed action** on the specified port.
    /// The actual delay (0-255 seconds) must be added to this value using [`Port::encode_delay_for_write_register`].
    /// The action is typically Open -> wait delay -> Close.
    pub const REG_DATA_SET_PORT_DELAY: Word = 0x0600;

    /// Returns the Modbus register address for controlling this specific port.
    ///
    /// The address is used with Modbus function 0x06 (Write Single Register), where the
    /// *data* written to this address determines the action (e.g., [`Port::REG_DATA_SET_PORT_OPEN`]).
    ///
    /// # Returns
    ///
    /// The `u16` Modbus address for controlling this port.
    ///
    /// # Example
    /// ```
    /// # use r413d08_lib::protocol::Port;
    /// assert_eq!(Port::try_from(0).unwrap().address_for_write_register(), 0x0001);
    /// assert_eq!(Port::try_from(7).unwrap().address_for_write_register(), 0x0008);
    /// ```
    pub fn address_for_write_register(&self) -> u16 {
        // Add 1 to the 0-based port index to get the 1-based Modbus address.
        (self.0 + 1) as u16
    }

    /// Encodes the register data value (`Word`) for setting a delayed action on a port.
    ///
    /// This combines the command code [`Port::REG_DATA_SET_PORT_DELAY`] (in the high byte)
    /// with the desired delay duration (in the low byte). The resulting `Word` should be written
    /// to the port's specific address (see [`Port::address_for_write_register`]) using
    /// Modbus function 0x06.
    ///
    /// # Arguments
    ///
    /// * `delay`: The delay duration in seconds.
    ///
    /// # Returns
    ///
    /// The corresponding `Word` to be written to the register for the delayed action command.
    ///
    /// # Example
    /// ```
    /// # use r413d08_lib::protocol::{Port, Word};
    /// // Command data to trigger a delayed action after 10 seconds:
    /// let delay_command_data = Port::encode_delay_for_write_register(10);
    /// assert_eq!(delay_command_data, 0x060A); // 0x0600 + 10
    ///
    /// // This value (0x060A) would then be written to the target port's Modbus address.
    /// // e.g., for port 3 (address 0x0004): WriteRegister(address=4, value=0x060A)
    /// ```
    pub fn encode_delay_for_write_register(delay: u8) -> Word {
        // Adds the delay (lower byte) to the command code (upper byte).
        Self::REG_DATA_SET_PORT_DELAY + (delay as Word)
    }
}

/// Allows accessing the inner `u8` port index value directly.
impl std::ops::Deref for Port {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Error indicating that a provided port index (`u8`) is outside the valid range
/// defined by [`Port::MIN`] and [`Port::MAX`] (inclusive).
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error(
    "The port value {0} is outside the valid range of {min} to {max}",
    min = Port::MIN,
    max = Port::MAX
)]
pub struct ErrorPortOutOfRange(
    /// The invalid port value that caused the error.
    pub u8,
);

impl TryFrom<u8> for Port {
    type Error = ErrorPortOutOfRange;

    /// Attempts to create a [`Port`] from a `u8` value, validating its 0-based range.
    ///
    /// # Arguments
    ///
    /// * `value`: The port index to validate.
    ///
    /// # Returns
    ///
    /// * `Ok(Port)`: If the `value` is within the valid range [[`Port::MIN`], [`Port::MAX`]].
    /// * `Err(ErrorPortOutOfRange)`: If the `value` is outside the valid range.
    ///
    /// # Example
    /// ```
    /// # use r413d08_lib::protocol::{Port, ErrorPortOutOfRange, NUMBER_OF_PORTS};
    /// let max_port_index = (NUMBER_OF_PORTS - 1) as u8; // Should be 7
    /// assert!(Port::try_from(0).is_ok());
    /// assert!(Port::try_from(max_port_index).is_ok());
    /// let invalid_index = max_port_index + 1; // Should be 8
    /// assert_eq!(Port::try_from(invalid_index).unwrap_err(), ErrorPortOutOfRange(invalid_index));
    /// ```
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // Check if the value as usize is within the valid range constants.
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ErrorPortOutOfRange(value))
        }
    }
}

impl std::fmt::Display for Port {
    /// Formats the port as its number (e.g., "0", "7").
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A zero-sized type providing constants for controlling all ports simultaneously.
pub struct PortsAll;

impl PortsAll {
    /// The address is used with Modbus function 0x06 (Write Single Register) for commands that affect
    /// all ports simultaneously, where the *data* that is written to this address determines the action
    /// (e.g. [`PortsAll::REG_DATA_SET_ALL_OPEN`]).
    pub const ADDRESS: u16 = 0x0000;

    /// Register data value to **open all** ports simultaneously (turn all relays ON).
    /// This value should be written to [`PortsAll::ADDRESS`].
    pub const REG_DATA_SET_ALL_OPEN: Word = 0x0700;

    /// Register data value to **close all** ports simultaneously (turn all relays OFF).
    /// This value should be written to [`PortsAll::ADDRESS`].
    pub const REG_DATA_SET_ALL_CLOSE: Word = 0x0800;
}

/// Represents a validated Modbus device address, used for RTU communication over RS485.
///
/// Valid addresses are in the range 1 to 247 (inclusive).
/// Use [`Address::try_from`] to create an instance from a `u8`.
/// Provides constants and methods for reading/writing the device address itself via Modbus commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Address(u8);

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Address::try_from(value).map_err(serde::de::Error::custom)
    }
}

/// Allows accessing the inner `u8` address value directly.
impl std::ops::Deref for Address {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Address {
    /// Returns the factory default Modbus address.
    fn default() -> Self {
        // Safe because `0x01` is within the valid range 1-247.
        Self(0x01)
    }
}

impl Address {
    /// The Modbus register address used to read or write the device's own Modbus address.
    ///
    /// Use Modbus function 0x03 (Read Holding Registers) to read (requires addressing
    /// the device with its *current* address or the broadcast address).
    /// Use function 0x06 (Write Single Register) to change the address (also requires
    /// addressing the device with its current address).
    pub const ADDRESS: u16 = 0x00FF;

    /// The number of registers to read when reading the device address.
    pub const QUANTITY: u16 = 1;

    /// The minimum valid assignable Modbus device address (inclusive).
    pub const MIN: u8 = 1;
    /// The maximum valid assignable Modbus device address (inclusive).
    pub const MAX: u8 = 247;

    /// The Modbus broadcast address (`0xFF` or 255).
    ///
    /// Can be used for reading the device address when it's unknown.
    /// This address cannot be assigned to a device as its permanent address.
    pub const BROADCAST: Address = Address(0xFF);

    /// Decodes the device [`Address`] from a Modbus holding register value read from the device.
    ///
    /// Expects `words` to contain the single register value read from the device address
    /// configuration register ([`Address::ADDRESS`]).
    /// It validates that the decoded address is within the assignable range ([`Address::MIN`]..=[`Address::MAX`]).
    ///
    /// # Arguments
    ///
    /// * `words`: A slice containing the [`Word`] value read from the device address register. Expected to have length 1.
    ///
    /// # Returns
    ///
    /// An [`Address`] struct containing the decoded and validated value.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// 1.  `words` is empty.
    /// 2.  The address value read from the register is outside the valid range.
    pub fn decode_from_holding_registers(words: &[Word]) -> Self {
        let word_value = *words
            .first()
            .expect("Register data for address must not be empty");
        // Attempt to convert the u8 value, panicking if it's out of range.
        Self::try_from(word_value as u8).expect("Invalid address value read from device register")
    }

    /// Encodes the device [`Address`] into a [`Word`] value suitable for writing to the
    /// device address configuration register ([`Address::ADDRESS`]) using Modbus function 0x06.
    ///
    /// # Returns
    ///
    /// The [`Word`] (u16) representation of the address value.
    pub fn encode_for_write_register(&self) -> Word {
        self.0 as u16
    }
}

/// Error indicating that a provided Modbus device address (`u8`) is outside the valid range
/// for assignable addresses, defined by [`Address::MIN`] and [`Address::MAX`] (inclusive).
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error(
    "The address value {0} is outside the valid assignable range of {min} to {max}",
    min = Address::MIN,
    max = Address::MAX
)]
pub struct ErrorAddressOutOfRange(
    /// The invalid address value that caused the error.
    pub u8,
);

impl TryFrom<u8> for Address {
    type Error = ErrorAddressOutOfRange;

    /// Attempts to create an [`Address`] from a `u8` value, validating its assignable range [[`Address::MIN`], [`Address::MAX`]].
    ///
    /// # Arguments
    ///
    /// * `value`: The Modbus address to validate.
    ///
    /// # Returns
    ///
    /// * `Ok(Address)`: If the `value` is within the valid assignable range [[`Address::MIN`], [`Address::MAX`]].
    /// * `Err(ErrorAddressOutOfRange)`: If the `value` is outside the valid assignable range (e.g., 0 or > 247).
    ///
    /// # Example
    /// ```
    /// # use r413d08_lib::protocol::{Address, ErrorAddressOutOfRange};
    /// assert!(Address::try_from(0).is_err());
    /// assert!(Address::try_from(1).is_ok());
    /// assert!(Address::try_from(247).is_ok());
    /// assert_eq!(Address::try_from(248).unwrap_err(), ErrorAddressOutOfRange(248));
    /// assert!(Address::try_from(255).is_err()); // Broadcast address is not valid for TryFrom
    /// ```
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ErrorAddressOutOfRange(value))
        }
    }
}

/// Provides a hexadecimal string representation (e.g., "0x01", "0xf7").
impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format as 2-digit hexadecimal with "0x" prefix.
        write!(f, "0x{:02x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Address Tests ---
    #[test]
    fn address_try_from_validation() {
        assert!(matches!(
            Address::try_from(0),
            Err(ErrorAddressOutOfRange(0))
        ));
        assert!(matches!(Address::try_from(Address::MIN), Ok(Address(1))));
        assert!(matches!(Address::try_from(Address::MAX), Ok(Address(247))));
        assert!(matches!(
            Address::try_from(Address::MAX + 1),
            Err(ErrorAddressOutOfRange(248))
        ));
        assert!(matches!(
            Address::try_from(255),
            Err(ErrorAddressOutOfRange(255))
        ));
        assert!(matches!(Address::try_from(100), Ok(Address(100))));
    }

    #[test]
    fn address_default() {
        assert_eq!(Address::default(), Address(1));
    }

    #[test]
    fn address_encode_decode() {
        let addr = Address::try_from(42).unwrap();
        let encoded = addr.encode_for_write_register();
        assert_eq!(encoded, 42u16);
        // Test decode (assumes valid input from register)
        let decoded = Address::decode_from_holding_registers(&[encoded]);
        assert_eq!(decoded, addr);
    }

    #[test]
    #[should_panic(expected = "Register data for address must not be empty")]
    fn address_decode_panics_on_empty() {
        let _ = Address::decode_from_holding_registers(&[]);
    }

    #[test]
    #[should_panic(expected = "Invalid address value read from device register")]
    fn address_decode_panics_on_invalid_value_zero() {
        // 0 is outside the valid 1-247 range
        let _ = Address::decode_from_holding_registers(&[0x0000]);
    }

    #[test]
    #[should_panic(expected = "Invalid address value read from device register")]
    fn address_decode_panics_on_invalid_value_high() {
        // 248 is outside the valid 1-247 range
        let _ = Address::decode_from_holding_registers(&[0x00F8]);
    }

    #[test]
    fn address_decode_valid() {
        assert_eq!(
            Address::decode_from_holding_registers(&[0x0001]),
            Address(1)
        );
        assert_eq!(
            Address::decode_from_holding_registers(&[0x00F7]),
            Address(247)
        );
    }

    // --- Port Tests ---
    #[test]
    fn port_try_from_validation() {
        assert!(matches!(Port::try_from(Port::MIN), Ok(Port(0))));
        assert!(matches!(Port::try_from(Port::MAX), Ok(Port(7))));
        assert!(matches!(
            Port::try_from(Port::MAX + 1),
            Err(ErrorPortOutOfRange(8))
        ));
        assert!(matches!(Port::try_from(3), Ok(Port(3))));
    }

    #[test]
    fn port_address_for_write_register_is_one_based() {
        // Check if address is 1-based according to documentation
        assert_eq!(Port::try_from(0).unwrap().address_for_write_register(), 1); // Port 0 -> Address 1
        assert_eq!(Port::try_from(1).unwrap().address_for_write_register(), 2); // Port 1 -> Address 2
        assert_eq!(Port::try_from(7).unwrap().address_for_write_register(), 8); // Port 7 -> Address 8
    }

    #[test]
    fn port_encode_delay() {
        assert_eq!(Port::encode_delay_for_write_register(0), 0x0600);
        assert_eq!(Port::encode_delay_for_write_register(10), 0x060A);
        assert_eq!(Port::encode_delay_for_write_register(255), 0x06FF);
    }

    // --- PortState / PortStates Tests ---
    #[test]
    fn port_state_decode() {
        assert_eq!(
            PortState::decode_from_holding_registers(0x0000),
            PortState::Close
        );
        assert_eq!(
            PortState::decode_from_holding_registers(0x0001),
            PortState::Open
        );
        assert_eq!(
            PortState::decode_from_holding_registers(0xFFFF),
            PortState::Open
        ); // Non-zero
    }

    #[test]
    fn port_states_decode() {
        let words_all_closed = [0x0000; NUMBER_OF_PORTS];
        let words_mixed = [
            0x0001, 0x0000, 0xFFFF, 0x0000, 0x0001, 0x0000, 0x0001, 0x0000,
        ];
        let words_short = [0x0001, 0x0000];
        let words_long = [
            0x0001, 0x0000, 0x0001, 0x0000, 0x0001, 0x0000, 0x0001, 0x0000, 0x9999,
        ];

        let expected_all_closed = PortStates([PortState::Close; NUMBER_OF_PORTS]);
        let expected_mixed = PortStates([
            PortState::Open,
            PortState::Close,
            PortState::Open,
            PortState::Close,
            PortState::Open,
            PortState::Close,
            PortState::Open,
            PortState::Close,
        ]);

        let mut expected_short_arr = [PortState::Close; NUMBER_OF_PORTS];
        expected_short_arr[0] = PortState::Open;
        expected_short_arr[1] = PortState::Close;
        let expected_short = PortStates(expected_short_arr);

        assert_eq!(
            PortStates::decode_from_holding_registers(&words_all_closed),
            expected_all_closed
        );
        assert_eq!(
            PortStates::decode_from_holding_registers(&words_mixed),
            expected_mixed
        );
        assert_eq!(
            PortStates::decode_from_holding_registers(&words_short),
            expected_short
        );
        assert_eq!(
            PortStates::decode_from_holding_registers(&words_long),
            expected_mixed
        ); // Ignores extra
    }

    // --- Display Tests ---
    #[test]
    fn display_formats() {
        assert_eq!(PortState::Open.to_string(), "open");
        assert_eq!(PortState::Close.to_string(), "close");
        assert_eq!(Address(1).to_string(), "0x01");
        assert_eq!(Address(247).to_string(), "0xf7");
        assert_eq!(Address::BROADCAST.to_string(), "0xff"); // Direct creation
        let states = PortStates([
            PortState::Open,
            PortState::Close,
            PortState::Open,
            PortState::Close,
            PortState::Close,
            PortState::Close,
            PortState::Close,
            PortState::Close,
        ]);
        assert_eq!(
            states.to_string(),
            "open, close, open, close, close, close, close, close"
        );
    }
}
