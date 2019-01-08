use num::{FromPrimitive, ToPrimitive};
use std::fmt;
use std::marker::PhantomData;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawId<Id>(pub u8, PhantomData<Id>);

impl<Id: FromPrimitive> RawId<Id> {
    pub fn try_into(self) -> Option<Id> {
        Id::from_u8(self.0)
    }
}

impl<Id: ToPrimitive> From<Id> for RawId<Id> {
    fn from(id: Id) -> Self {
        RawId(id.to_u8().expect("always one byte"), PhantomData)
    }
}

impl<Id: fmt::Debug + FromPrimitive + Copy> fmt::Debug for RawId<Id> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(id) = self.try_into() {
            write!(f, "{:?}", id)
        } else {
            f.debug_tuple("RawId").field(&self.0).finish()
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum OutputReportId {
    RumbleSubcmd = 0x01,
    MCUFwUpdate = 0x03,
    RumbleOnly = 0x10,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum InputReportId {
    // 0x3F not used
    Standard = 0x21,
    MCUFwUpdate = 0x23,
    StandardFull = 0x30,
    StandardFullMCU = 0x31,
    // 0x32 not used
    // 0x33 not used
}

// All unused values are a Nop
#[repr(u8)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum SubcommandId {
    GetOnlyControllerState = 0x00,
    BluetoothManualPairing = 0x01,
    RequestDeviceInfo = 0x02,
    SetInputReportMode = 0x03,
}

// Console -> Joy-Con
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct OutputReport {
    pub report_id: RawId<OutputReportId>,
    pub packet_counter: u8,
    pub rumble_data: RumbleData,
    pub subcmd: SubcommandRequest,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SubcommandRequest {
    pub subcommand_id: RawId<SubcommandId>,
    pub u: SubcommandRequestData,
}

impl fmt::Debug for SubcommandRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = f.debug_struct("SubcommandRequest");
        match self.subcommand_id.try_into() {
            Some(SubcommandId::SetInputReportMode) => {
                out.field("subcommand", unsafe { &self.u.input_report_mode })
            }
            Some(subcmd) => out.field("subcommand", &subcmd),
            None => out.field("subcommand_id", &self.subcommand_id),
        };
        out.finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RumbleData {
    pub raw: [u8; 8],
}

impl Default for RumbleData {
    fn default() -> Self {
        RumbleData {
            raw: [0x00, 0x01, 0x40, 0x40, 0x00, 0x01, 0x40, 0x40],
        }
    }
}

// Joy-Con -> Console
#[repr(C)]
#[derive(Copy, Clone)]
pub struct InputReport {
    pub report_id: RawId<InputReportId>,
    pub timer: u8,
    pub info: u8,
    pub buttons: ButtonsStatus,
    pub left_stick: StickStatus,
    pub right_stick: StickStatus,
    pub vibrator: u8,
    pub u: ExtraData,
}

impl fmt::Debug for InputReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = f.debug_struct("InputReport");
        out.field("report_id", &self.report_id)
            .field("timer", &self.timer)
            .field("info", &self.info)
            .field("buttons", &self.buttons)
            .field("left_stick", &self.left_stick)
            .field("right_stick", &self.right_stick)
            .field("vibrator", &self.vibrator);
        match self.report_id.try_into() {
            Some(InputReportId::Standard) => {
                out.field("subcommand_reply", unsafe { &self.u.subcmd_reply })
            }
            Some(InputReportId::MCUFwUpdate) => out.field("mcu_fw_update", &"[data]"),
            // TODO: mask MCU
            Some(InputReportId::StandardFull) => {
                out.field("subcommand_reply", unsafe { &self.u.gyro_acc_nfc_ir })
            }
            Some(InputReportId::StandardFullMCU) => {
                out.field("subcommand_reply", unsafe { &self.u.gyro_acc_nfc_ir })
            }
            None => &mut out,
        };
        out.finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ButtonsStatus {
    pub data: [u8; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StickStatus {
    pub data: [u8; 3],
}

impl StickStatus {
    pub fn h(self) -> u16 {
        u16::from(self.data[0]) | u16::from(self.data[1] & 0xf) << 8
    }

    pub fn v(self) -> u16 {
        u16::from(self.data[1]) >> 4 | u16::from(self.data[2]) << 4
    }
}

impl fmt::Debug for StickStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("StickStatus")
            .field(&self.h())
            .field(&self.v())
            .finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ExtraData {
    pub subcmd_reply: SubcommandReply,
    pub mcu_fw_update: [u8; 37],
    pub gyro_acc_nfc_ir: GyroAccNFCIR,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SubcommandReply {
    pub ack: u8,
    pub subcommand_id: RawId<SubcommandId>,
    pub u: SubcommandReplyData,
}

impl fmt::Debug for SubcommandReply {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = f.debug_struct("SubcommandReply");
        out.field("ack", &self.ack);
        match self.subcommand_id.try_into() {
            Some(SubcommandId::RequestDeviceInfo) => {
                out.field("device_info", unsafe { &self.u.device_info })
            }
            Some(subcmd) => out.field("subcommand", &subcmd),
            None => out.field("subcommand_id", &self.subcommand_id),
        };
        out.finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union SubcommandRequestData {
    pub input_report_mode: RawInputReportMode,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RawInputReportMode(u8);

#[repr(C)]
#[derive(Copy, Clone)]
pub union SubcommandReplyData {
    pub device_info: DeviceInfo,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DeviceInfo {
    pub firmware_version: [u8; 2],
    // 1=Left Joy-Con, 2=Right Joy-Con, 3=Pro Controller
    pub which_controller: u8,
    // Unknown. Seems to be always 02
    _something: u8,
    // Big endian
    pub mac_address: [u8; 6],
    // Unknown. Seems to be always 01
    _somethingelse: u8,
    // bool
    pub use_spi_colors: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GyroAccNFCIR {
    pub gyro_acc_frames: [[u8; 12]; 3],
    pub nfc_ir_data: [u8; 313],
}

impl fmt::Debug for GyroAccNFCIR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GyroAccNFCIR")
            .field("gyro_acc_frames", &self.gyro_acc_frames)
            .field("nfc_ir_data", &"[data]")
            .finish()
    }
}
