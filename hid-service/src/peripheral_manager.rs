// use embedded_services::{comms::{self, Internal}, hid, keyboard};

use embedded_services::{comms::{self, Internal}, hid, keyboard};

// TODO(melvin): remove hack re-export
pub use embedded_services::keyboard::KeyboardMessage;

pub struct Config {
    pub peripherals: &'static[PeripheralMapping],
}

pub enum PeripheralId {
    /// Keyboard device
    Keyboard(keyboard::DeviceId),
}

/// Mapping from a peripheral device to a physical HID device. Optionally associate a report ID with the peripheral
pub struct PeripheralMapping {
    pub peripheral_device_id: PeripheralId,
    pub hid_device_id: hid::DeviceId,
    pub report_id: Option<hid::ReportId>,
}

pub struct PeripheralManager {
    /// Comms endpoint
    tp: comms::Endpoint,
    /// Config
    config: Config,
}

impl PeripheralManager {
    pub fn new(
        config: Config,
    ) -> Self {
        Self {
            tp: comms::Endpoint::uninit(
                comms::EndpointID::Internal(Internal::Hid)
            ),
            config,
            // cache, // TODO(melvin): cache hid lookups
        }
    }

    pub async fn process(&self) {
        todo!()
        // match select(self.device.wait_request(), self.kb_msg.wait()).await {
        //     Either::Left(hid_msg) => self.process_hid_request(hid_msg).await,
        //     Either::Right(kb_msg) => self.process_kb_msg(kb_msg).await
        // }
    }

    async fn process_hid_request(&self, request: hid::Request<'_>) {
        todo!()
    //     let response = match msg {
    //         Request::Description => self.generate_descriptor_response(),
    //         // ...
    //     }

    //     self.device.send_response(response).await;
    }

    async fn process_kb_msg(&self, msg: KeyboardMessage) {
        todo!()
        // Assert interrupt
    }
}