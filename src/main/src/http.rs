use anyhow::{bail, Result};
use core::str;
use embedded_svc::http::{client::Client, Method};
use embedded_svc::io::Read;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};

pub fn get(url: impl AsRef<str>) -> Result<String> {
    // 1. Create a new EspHttpClient. (Check documentation)
    // ANCHOR: connection
    let connection = EspHttpConnection::new(&Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        timeout: Some(std::time::Duration::from_secs(30)),
        ..Default::default()
    })?;
    // ANCHOR_END: connection
    let mut client = Client::wrap(connection);

    // 2. Open a GET request to `url`
    let headers = [("content-type", "application/json")];
    let request = client.request(Method::Get, url.as_ref(), &headers)?;

    // 3. Submit write request and check the status code of the response.
    // Successful http status codes are in the 200..=299 range.
    let response = request.submit()?;
    let status = response.status();

    let mut data: String = "".to_owned();
    match status {
        200..=299 => {
            // 4. if the status is OK, read response data chunk by chunk into a buffer and print it until done
            //
            // NB. see http_client.rs for an explanation of the offset mechanism for handling chunks that are
            // split in the middle of valid UTF-8 sequences. This case is encountered a lot with the given
            // example URL.
            let mut buf = [0_u8; 256];
            let mut offset = 0;
            let mut reader = response;
            loop {
                if let Ok(size) = Read::read(&mut reader, &mut buf[offset..]) {
                    if size == 0 {
                        break;
                    }
                    // 5. try converting the bytes into a Rust (UTF-8) string and print it
                    let size_plus_offset = size + offset;
                    match str::from_utf8(&buf[..size_plus_offset]) {
                        Ok(text) => {
                            data.push_str(text);
                            offset = 0;
                        }
                        Err(error) => {
                            let valid_up_to = error.valid_up_to();
                            unsafe {
                                data.push_str(str::from_utf8_unchecked(&buf[..valid_up_to]));
                            }
                            buf.copy_within(valid_up_to.., 0);
                            offset = size_plus_offset - valid_up_to;
                        }
                    }
                }
            }
        }
        _ => bail!("Unexpected response code: {}", status),
    }
    Ok(data)
}
