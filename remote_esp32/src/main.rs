use std::sync::Arc;
use anyhow::Result;

use embedded_svc::http::Method;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfiguration};
use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::{EspSystemEventLoop, SystemEventLoop};
use esp_idf_svc::http::server::{EspHttpServer, Request};
use esp_idf_svc::netif::{EspNetif, EspNetifWait};
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::wifi::EspWifi;
use log::*;

const CONTROL_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Pontiac GTO</title>
</head>
<body>

  <h1>Pontiac GTO 1967 control center</h1>

  <!-- Speed -->
  <div>
    <label for="speed">Speed:</label>
    <input class="slider" type="range" id="speed" name="speed" min="0" max="100" value="50" endpoint="updatespeed">
    <span id="speedValue">50</span>
  </div>

  <!-- Steering -->
  <div>
    <label for="steering">Steering:</label>
    <input class="slider" type="range" id="steering" name="steering" min="-45" max="45" value="0" endpoint="updateSteering">
    <span id="steeringValue">0</span>
  </div>

  <!-- Headlight -->
  <div>
    <label for="Headlight">LHeadlight:</label>
    <input class="checkbox" type="checkbox" id="Headlight" name="Headlight" endpoint="toggleHeadlight">
  </div>

  <script>
    // Handle all sliders
    const sliders = document.querySelectorAll('.slider');
    sliders.forEach(slider => {
      slider.addEventListener('input', event => {
        const value = event.target.value;
        const id = event.target.id;
        const endpoint = event.target.getAttribute('endpoint');

        // Update displayed value
        const valueDisplay = document.getElementById(id + 'Value');
        if (valueDisplay) {
          valueDisplay.textContent = value;
        }

        // Send the slider value via fetch in the URL
        fetch(`/${endpoint}/${value}`)
          .catch(error => console.error(`Error updating ${id}:`, error));
      });
    });

    // Handle all checkboxes
    const checkboxes = document.querySelectorAll('.checkbox');
    checkboxes.forEach(checkbox => {
      checkbox.addEventListener('change', event => {
        const isChecked = event.target.checked;
        const endpoint = event.target.getAttribute('endpoint');
        const id = event.target.id;

        // Send the checkbox state via fetch in the URL
        fetch(`/${endpoint}/${isChecked}`)
          .catch(error => console.error(`Error toggling ${id}:`, error));
      });
    });
  </script>

</body>
</html>
"#;

fn main() -> Result<()> {
    // Initialize logging
    esp_idf_sys::link_patches();
    esp_idf_hal::prelude:: Peripherals::take().unwrap();
    esp_idf_hal::prelude:: Rtc::new(peripherals.RTC_CNTL);
    esp_idf_hal::prelude:: ClockControl::boot_defaults(peripherals.DPORT).freeze();

    esp_idf_svc::log::EspLogger::initialize_default();

    // Create a system event loop
    let sysloop = EspSystemEventLoop::take()?;

    // Initialize Wi-Fi
    let wifi = initialize_wifi(sysloop.clone())?;

    // Start HTTP server
    let mut httpd = EspHttpServer::new(&Default::default())?;

    // Serve the main HTML page at "/"
    httpd.fn_handler("/", Method::Get, |request| {
        let response = request.into_ok_response()?;
        response.write_all(CONTROL_HTML.as_bytes())?;
        Ok(())
    })?;

    // Example endpoints:
    // 1) "/updateSpeed1/<value>"
    // 2) "/updateSteering1/<value>"
    // 3) "/toggleLED1/<true|false>"
    // 4) "/toggleLED2/<true|false>"

    httpd.fn_handler("/updateSpeed1/:value", Method::Get, handle_path_params)?;
    httpd.fn_handler("/updateSteering1/:value", Method::Get, handle_path_params)?;
    httpd.fn_handler("/toggleLED1/:state", Method::Get, handle_path_params)?;
    httpd.fn_handler("/toggleLED2/:state", Method::Get, handle_path_params)?;

    info!("HTTP server started. Go to http://<device_ip>/ to see the page.");

    // The main thread can do other work, or just sleep
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

/// Simple path-parameter handler that prints the endpoint and value/state.
fn handle_path_params(request: Request) -> Result<()> {
    // Extract "value" or "state" from the path
    let params = request.params();
    // E.g., "value" if the route is "/updateSpeed1/:value"
    // or "state" if the route is "/toggleLED1/:state"

    // For demonstration, we simply log them
    for (param, val) in params {
        info!("Got param: {} = {}", param, val);
        // Here, you'd actually do something with 'val',
        // e.g., parse it as an integer for speed/steering
        // or parse it as bool for LED on/off.
    }

    // Return a 200 OK
    let mut response = request.into_ok_response()?;
    response.write_all(b"OK")?;
    Ok(())
}

/// Initialize Wi-Fi in station mode.
///
/// Modify the SSID and PASSWORD to match your network.
fn initialize_wifi(sysloop: Arc<SystemEventLoop>) -> Result<EspWifi<'static>> {
    let mut wifi = EspWifi::new(
        Arc::new(EspNetif::new()?),
        sysloop,
        Arc::new(EspDefaultNvs::new()?),
    )?;

    let ap_ssid = "Pontiac";
    let ap_password = "GTO";

    // Configure as a Wi-Fi station
    wifi.set_configuration(&WifiConfiguration::Client(ClientConfiguration {
        ssid: ap_ssid.into(),
        password: ap_password.into(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    info!("Starting Wi-Fi...");
    wifi.start()?;

    info!("Connecting to Wi-Fi...");
    wifi.connect()?;

    // Wait for a lease on an IP
    EspNetifWait::new::<EspNetif>(wifi.netif(), None).wait_with_timeout(
        std::time::Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap_or(false) && wifi.ip_info().unwrap().ip != std::net::Ipv4Addr::new(0, 0, 0, 0)
        },
    )?;

    let ip_info = wifi.ip_info()?;
    info!("Wi-Fi connected: {:?}", ip_info);

    Ok(wifi)
}
