use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Arc;
use crate::Engine;

pub struct CEngine {
    engine: Arc<Engine>,
}

#[repr(C)]
pub struct CUrlSpecificResources {
    hide_selectors: *mut c_char,
    style_selectors: *mut *mut c_char,
    exceptions: *mut *mut c_char,
    injected_script: *mut c_char,
    generichide: bool,
}

#[no_mangle]
pub extern "C" fn engine_create(rules: *const c_char) -> *mut CEngine {
    if rules.is_null() {
        return ptr::null_mut();
    }
    
    let rules_str = unsafe {
        match CStr::from_ptr(rules).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let mut filter_set = crate::lists::FilterSet::new(false);
    filter_set.add_filter_list(&rules_str, Default::default());
    
    let engine = Engine::from_filter_set(filter_set, true);
    
    Box::into_raw(Box::new(CEngine {
        engine: Arc::new(engine),
    }))
}

#[no_mangle]
pub extern "C" fn engine_destroy(engine: *mut CEngine) {
    if !engine.is_null() {
        unsafe {
            let _ = Box::from_raw(engine);
        }
    }
}

#[no_mangle]
pub extern "C" fn engine_match(
    engine: *const CEngine,
    url: *const c_char,
    host: *const c_char,
    tab_host: *const c_char,
) -> bool {
    if engine.is_null() || url.is_null() || host.is_null() || tab_host.is_null() {
        return false;
    }
    
    let engine = unsafe { &(*engine).engine };
    
    let url_str = unsafe {
        match CStr::from_ptr(url).to_str() {
            Ok(s) => s,
            Err(_) => return false,
        }
    };
    
    let _host_str = unsafe {
        match CStr::from_ptr(host).to_str() {
            Ok(s) => s,
            Err(_) => return false,
        }
    };
    
    let tab_host_str = unsafe {
        match CStr::from_ptr(tab_host).to_str() {
            Ok(s) => s,
            Err(_) => return false,
        }
    };
    
    let source_url = format!("https://{}", tab_host_str);
    let request = match crate::request::Request::new(url_str, &source_url, "document") {
        Ok(r) => r,
        Err(_) => return false,
    };
    
    let blocker_result = engine.check_network_request(&request);
    blocker_result.matched && blocker_result.exception.is_none()
}

#[no_mangle]
pub extern "C" fn engine_url_cosmetic_resources(
    engine: *const CEngine,
    url: *const c_char,
) -> *mut CUrlSpecificResources {
    if engine.is_null() || url.is_null() {
        return ptr::null_mut();
    }
    
    let engine = unsafe { &(*engine).engine };
    
    let url_str = unsafe {
        match CStr::from_ptr(url).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let resources = engine.url_cosmetic_resources(url_str);
    
    let hide_selectors = if resources.hide_selectors.is_empty() {
        ptr::null_mut()
    } else {
        let selectors_string = resources.hide_selectors.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(",");
        match CString::new(selectors_string) {
            Ok(s) => s.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    };
    
    let injected_script = if resources.injected_script.is_empty() {
        ptr::null_mut()
    } else {
        match CString::new(resources.injected_script) {
            Ok(s) => s.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    };
    
    Box::into_raw(Box::new(CUrlSpecificResources {
        hide_selectors,
        style_selectors: ptr::null_mut(), // Not implementing for now
        exceptions: ptr::null_mut(),      // Not implementing for now
        injected_script,
        generichide: resources.generichide,
    }))
}

#[no_mangle]
pub extern "C" fn url_specific_resources_destroy(resources: *mut CUrlSpecificResources) {
    if !resources.is_null() {
        unsafe {
            let resources = Box::from_raw(resources);
            
            if !resources.hide_selectors.is_null() {
                let _ = CString::from_raw(resources.hide_selectors);
            }
            
            if !resources.injected_script.is_null() {
                let _ = CString::from_raw(resources.injected_script);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}