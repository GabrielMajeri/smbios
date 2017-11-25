//! SMBIOS 2.1 or newer structure definitions.
//!
//! ## Entry points
//!
//! The `Smbios2EntryPoint`, despite its name, can be returned by a SMBIOS 3 implementation.
//! The only difference is that it points to an array which is within the first 4 GiBs.
//!
//! ## Structure array
//!
//! The entry point contains the size and address of an array of structures.
//!
//! These structures contain various information about the system and its components.
//! This information is system-specific, and can be used to determine the capabilities
//! and hardware of the running computer.
//!
//! ## Structure format
//!
//! The structures begin with a common header, describing their type and size.
//!
//! The first few fields of a structure contain numbers, either referencing a string
//! or describing some parameter of the computer.
//!
//! After this "formatted" area of the SMBIOS structures come a tightly-packed array
//! of NULL-terminated strings. These strings are referenced by index (for example,
//! if a structure reports the BIOS' name is "3", it means you need to parse the string
//! with index 3, counting from 0).
//!
//! At the end of this string area is a double NULL-terminator.

#![no_std]

#![deny(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", deny(clippy))]

#[macro_use]
extern crate bitflags;

/// Entry point available in SMBIOS 2.1+, only supports 32-bit addresses.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Smbios2EntryPoint {
    /// Must be "_SM_".
    pub anchor0: [u8; 4],
    /// Checksum of the whole structure.
    pub chksum0: u8,
    /// Length of whole header.
    pub length: u8,
    /// Major / minor version.
    pub smbios_version: (u8, u8),
    /// Size of larges SMBIOS structure in the array.
    pub max_size: u16,
    /// Revision of the EPS, should be 0.
    pub revision: u8,
    /// Reserved, should be 0.
    pub _reserved: [u8; 5],
    /// First filed of the intermediate EPS.
    /// Must be "_DMI_".
    pub anchor1: [u8; 5],
    /// Checksum of the intermediate EPS.
    pub chksum1: u8,
    /// Size of table in bytes.
    pub table_size: u16,
    /// 32-bit physical address of table.
    pub table_addr: u32,
    /// Total number of structures.
    pub table_len: u16,
    /// Revision returned as binary-coded hex digits.
    pub bcd_revision: u8,
}

/// Entry point for SMBIOS 3+ structures, supports 64-bit addresses.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Smbios3EntryPoint {
    /// Must be "_SM3_".
    pub anchor: [u8; 5],
    /// Checksum of this structure.
    pub chksum: u8,
    /// Length of this structure.
    pub length: u8,
    /// Version of this structure, in the following order: major / minor / doc revision.
    pub version: (u8, u8, u8),
    /// A value of 1 means SMBIOS 3, anything else is reserved.
    pub revision: u8,
    /// Reserved, must be 0.
    pub _reserved: u8,
    /// Max size of table pointed to by `address`, in bytes.
    pub max_size: u16,
    /// 64-bit physical address of the SMBIOS structures array.
    pub address: u64,
}

/// A type used to index the string table for each structure.
pub type StringRef = u8;

/// Common header for all SMBIOS structures.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Header {
    /// Type of this structure.
    pub ty: Type,
    /// Size of the formatted area of the structure, including the header.
    ///
    /// The length of the strings at the end is not included.
    pub len: u8,
    /// The unique handle of this structure.
    ///
    /// This is meant to be used by programs which need to uniquely identify SMBIOS structures.
    /// However, this handle might change between boots.
    ///
    /// Its value must be between 0 and 0xFFFE (SMBIOS 2) or 0xFEFF (SMBIOS 3).
    pub handle: u16,
}

/// Structure types defined by the specification.
///
/// Values between 0 and 127 are reserved and defined by the specification,
/// all values above are vendor-specific.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Type {
    /// BIOS information.
    BiosInformation,
    /// System information.
    SystemInformation,
    /// System enclosure information.
    SystemEnclosure = 3,
    /// Information about a processor.
    ProcessorInformation = 4,
    /// Information about processor caches.
    CacheInformation = 7,
    /// Description of an upgradeable system slot.
    SystemSlot = 9,
    /// Information about an array of physical memory.
    PhysicalMemoryArray = 16,
    /// Information about a memory device.
    MemoryDevice = 17,
    /// Information about what is a physical memory array mapped to.
    MemoryArrayMappedAddress = 19,
    /// Information about the boot process.
    SystemBootInformation = 32,
}

/// BIOS information structure.
///
/// This structure has a variable size byte array at the end,
/// you should go to the `length - 18 bytes` offset and also parse
/// the `BiosInformationTail` structure.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct BiosInformation {
    /// Common header.
    pub header: Header,
    /// BIOS vendor.
    pub vendor: StringRef,
    /// Free-form BIOS version.
    pub version: StringRef,
    /// The segment of the BIOS's starting address.
    pub starting_address_segment: u16,
    /// Release date of BIOS, in mm/dd/yyyy format.
    pub release_date: StringRef,
    /// Size of BIOS ROM, in multiples of 64 KiB.
    /// Setting to 255 means size is >= 16 MiB.
    pub rom_size: u8,
    /// BIOS characteristics.
    pub characteristics: BiosCharacteristics,
    /// Extended characteristic bytes.
    ///
    /// This has a variable size, and after it comes
    /// the `BiosInformationTail` structure.
    ///
    /// This field and all fields after it are supported by SMBIOS 2.4+ only.
    ///
    /// The current spec only defines the values of the first 2 bytes of this field.
    pub extended_chrs: BiosExtendedCharacteristics,
}

/// Tail of the BIOS information structure.
///
/// Guaranteed to be at the offset `length - 18` bytes
/// from the start of the information structure.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct BiosInformationTail {
    /// BIOS version in major / minor format.
    ///
    /// These values are both 255 if this field is not supported.
    pub version: (u8, u8),
    /// Embedded controller version in major / minor format.
    ///
    /// Both values are 255 if no embedded controller is present.
    pub ec_version: (u8, u8),
    /// Extended BIOS size.
    ///
    /// Only supported by SMBIOS 3.1+.
    ///
    /// Unit is determined by top-two bits: 00 means MiB, 01 means GiB.
    pub size: u16,
}

bitflags! {
    /// BIOS characteristics.
    ///
    /// See the spec for more information.
    pub struct BiosCharacteristics: u64 {
        /// Set if BIOS characteristics are not supported.
        const NOT_SUPPORTED = 1 << 3;
        /// PCI is supported.
        const PCI = 1 << 7;
        /// Plug-and-Play BIOS.
        const PNP = 1 << 9;
    }
}

bitflags! {
    /// Extended BIOS characteristics.
    pub struct BiosExtendedCharacteristics: u16 {
        /// ACPI support.
        const ACPI = 1 << 0;
        /// Legacy USB support (emulate USB keyboard as PS/2 keyboard).
        const USB_LEGACY = 1 << 1;
        /// Smart Battery support.
        const SMART_BATTERY = 1 << 7;
        /// UEFI firmware supported.
        const UEFI = 1 << 11;
        /// Running in a virtual machine.
        const VIRTUAL_MACHINE = 1 << 12;
    }
}
