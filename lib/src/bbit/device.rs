use std::vec::Vec;

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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub struct CommandArray {
    cmd_array: [Command],
    cmd_array_size: usize,
}

#[derive(Copy, Clone, Debug)]
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

struct ParameterInfo<'a> {
    parameter: &'a Parameter,
    access: &'a ParamAccess,
}

#[derive(Copy, Clone, Debug)]
struct ParamInfoArray<'a> {
    info_array: [ParameterInfo<'a>],
    info_count: usize,
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
