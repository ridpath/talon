// Proto enum conversion helpers for tonic 0.11 compatibility

use super::proto_generated::{EventType, UpdateType};

impl Default for EventType {
    fn default() -> Self {
        EventType::EventStarted
    }
}

impl TryFrom<i32> for EventType {
    type Error = ();
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventType::EventStarted),
            1 => Ok(EventType::EventProgress),
            2 => Ok(EventType::EventOutput),
            3 => Ok(EventType::EventError),
            4 => Ok(EventType::EventCompleted),
            5 => Ok(EventType::EventFailed),
            _ => Err(()),
        }
    }
}

impl From<EventType> for i32 {
    fn from(value: EventType) -> i32 {
        value as i32
    }
}

impl Default for UpdateType {
    fn default() -> Self {
        UpdateType::UpdateGadget
    }
}

impl TryFrom<i32> for UpdateType {
    type Error = ();
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(UpdateType::UpdateGadget),
            1 => Ok(UpdateType::UpdateLibcOffset),
            2 => Ok(UpdateType::UpdateShellcode),
            3 => Ok(UpdateType::UpdateTarget),
            _ => Err(()),
        }
    }
}

impl From<UpdateType> for i32 {
    fn from(value: UpdateType) -> i32 {
        value as i32
    }
}
