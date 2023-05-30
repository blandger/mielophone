use crate::EventHandler;
use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral},
};
use std::sync::Arc;

/// Structure to contain EEG data and interval.
#[derive(Debug, Clone)]
pub struct EegData {
    data: i16,
    index: u8,
}
/// Structure to contain EEG data and interval.
#[derive(Debug, Clone)]
pub struct CommandData {
    data: i16,
    cmd_type: CommandType,
}

/// The core sensor manager
pub struct BleSensor {
    /// BLE connection manager
    ble_manager: Manager,
    /// Connected and controlled device
    ble_device: Option<Peripheral>,
    /// Handler for callback events
    event_handler: Option<Arc<dyn EventHandler>>,
    /// Device manage and send commands
    control_point: Option<ControlPoint>,
}

/// Struct that has access to command point.
#[derive(Debug, PartialEq, Eq)]
pub struct ControlPoint {
    control_point: Characteristic,
}

#[derive(Clone, Debug)]
pub enum CommandType {
    CommandStartSignal,
    CommandStopSignal,
    CommandStartResist,
    CommandStopResist,
    CommandStartMEMS,
    CommandStopMEMS,
    CommandStartRespiration,
    CommandStopRespiration,
    CommandStartStimulation,
    CommandStopStimulation,
    CommandEnableMotionAssistant,
    CommandDisableMotionAssistant,
    CommandFindMe,
}

#[derive(Clone, Debug)]
pub struct CommandArray<'a> {
    cmd_array: &'a [CommandData],
    cmd_array_size: usize,
}

#[derive(Clone, Debug)]
pub enum Parameter {
    ParameterName,
    ParameterState,
    ParameterAddress,
    ParameterSerialNumber,
    ParameterHardwareFilterState,
    ParameterFirmwareMode,
    ParameterSamplingFrequency,
    ParameterGain,
    ParameterOffset,
    ParameterExternalSwitchState,
    ParameterADCInputState,
    ParameterAccelerometerSens,
    ParameterGyroscopeSens,
    ParameterStimulatorAndMAState,
    ParameterStimulatorParamPack,
    ParameterMotionAssistantParamPack,
    ParameterFirmwareVersion,
}

#[derive(Copy, Clone, Debug)]
pub enum ParamAccess {
    Read,
    ReadWrite,
    ReadNotify,
}

#[derive(Copy, Clone, Debug)]
pub enum ChannelType {
    ChannelTypeSignal,
    ChannelTypeBattery,
    ChannelTypeElectrodesState,
    ChannelTypeRespiration,
    ChannelTypeMEMS,
    ChannelTypeOrientation,
    ChannelTypeConnectionStats,
    ChannelTypeResistance,
    ChannelTypePedometer,
    ChannelTypeCustom,
}

/*
pub struct ChannelInfo {
    name: char[],
    type: ChannelType,
    index: usize,
}*/
