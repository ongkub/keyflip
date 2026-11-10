use windows::{core::PCWSTR, Win32::Networking::WinHttp::*};

pub fn post(url: &str, body: &[u8]) -> Result<(u32, Vec<u8>), String> {
    let (host, path) = url
        .strip_prefix("https://")
        .ok_or("รายงานต้องใช้ HTTPS")?
        .split_once('/')
        .map(|(h, p)| (h, format!("/{p}")))
        .unwrap_or((url, "/".into()));
    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    unsafe {
        let agent = wide("KeyFlip");
        let h = wide(host);
        let p = wide(&path);
        let verb = wide("POST");
        let session = WinHttpOpen(
            PCWSTR(agent.as_ptr()),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        );
        if session.is_null() {
            return Err("WinHTTP session failed".into());
        }
        let _s = Handle(session);
        WinHttpSetTimeouts(session, 10_000, 10_000, 10_000, 10_000).map_err(|e| e.to_string())?;
        let connect = WinHttpConnect(session, PCWSTR(h.as_ptr()), 443, 0);
        if connect.is_null() {
            return Err("WinHTTP connect failed".into());
        }
        let _c = Handle(connect);
        let request = WinHttpOpenRequest(
            connect,
            PCWSTR(verb.as_ptr()),
            PCWSTR(p.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            WINHTTP_FLAG_SECURE,
        );
        if request.is_null() {
            return Err("WinHTTP request failed".into());
        }
        let _r = Handle(request);
        let header = wide("Content-Type: application/json\r\n");
        WinHttpSendRequest(
            request,
            Some(&header[..header.len() - 1]),
            Some(body.as_ptr() as *const _),
            body.len() as u32,
            body.len() as u32,
            0,
        )
        .map_err(|e| e.to_string())?;
        WinHttpReceiveResponse(request, std::ptr::null_mut()).map_err(|e| e.to_string())?;
        let mut status = 0u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        WinHttpQueryHeaders(
            request,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some(&mut status as *mut _ as *mut _),
            &mut size,
            std::ptr::null_mut(),
        )
        .map_err(|e| e.to_string())?;
        let mut response = Vec::new();
        loop {
            let mut available = 0u32;
            WinHttpQueryDataAvailable(request, &mut available).map_err(|e| e.to_string())?;
            if available == 0 {
                break;
            }
            let at = response.len();
            response.resize(at + available as usize, 0);
            let mut read = 0u32;
            WinHttpReadData(
                request,
                response[at..].as_mut_ptr() as *mut _,
                available,
                &mut read,
            )
            .map_err(|e| e.to_string())?;
            response.truncate(at + read as usize);
        }
        Ok((status, response))
    }
}
struct Handle(*mut std::ffi::c_void);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = WinHttpCloseHandle(self.0);
        }
    }
}
