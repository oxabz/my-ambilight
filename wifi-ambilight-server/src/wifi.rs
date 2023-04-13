use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    eventloop::{EspEventLoop, System},
    wifi::EspWifi,
};

use crate::constants::{WIFI_PASS, WIFI_SSID};

pub fn setup_wifi(modem: Modem, sysloop: EspEventLoop<System>) -> EspWifi<'static> {
    let mut wifi = EspWifi::new(modem, sysloop, None).unwrap();
    let conf = Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.into(),
        auth_method: embedded_svc::wifi::AuthMethod::WPA2Personal,
        password: WIFI_PASS.into(),
        ..Default::default()
    });
    wifi.set_configuration(&conf);

    wifi.start().unwrap();

    wifi.connect().unwrap();

    wifi
}
