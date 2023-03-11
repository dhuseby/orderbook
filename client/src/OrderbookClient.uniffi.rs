// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!


// Check for compatibility between `uniffi` and `uniffi_bindgen` versions.
// Note that we have an error message on the same line as the assertion.
// This is important, because if the assertion fails, the compiler only
// seems to show that single line as context for the user.
uniffi::assert_compatible_version!("0.23.0"); // Please check that you depend on version 0.23.0 of the `uniffi` crate.












// Everybody gets basic buffer support, since it's needed for passing complex types over the FFI.
//
// See `uniffi/src/ffi/rustbuffer.rs` for documentation on these functions

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub extern "C" fn ffi_OrderbookClient_1e03_rustbuffer_alloc(size: i32, call_status: &mut uniffi::RustCallStatus) -> uniffi::RustBuffer {
    uniffi::ffi::uniffi_rustbuffer_alloc(size, call_status)
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn ffi_OrderbookClient_1e03_rustbuffer_from_bytes(bytes: uniffi::ForeignBytes, call_status: &mut uniffi::RustCallStatus) -> uniffi::RustBuffer {
    uniffi::ffi::uniffi_rustbuffer_from_bytes(bytes, call_status)
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn ffi_OrderbookClient_1e03_rustbuffer_free(buf: uniffi::RustBuffer, call_status: &mut uniffi::RustCallStatus) {
    uniffi::ffi::uniffi_rustbuffer_free(buf, call_status)
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn ffi_OrderbookClient_1e03_rustbuffer_reserve(buf: uniffi::RustBuffer, additional: i32, call_status: &mut uniffi::RustCallStatus) -> uniffi::RustBuffer {
    uniffi::ffi::uniffi_rustbuffer_reserve(buf, additional, call_status)
}

// Error definitions, corresponding to `error` in the UDL.



#[doc(hidden)]
pub struct FfiConverterTypeOrderbookError;

#[doc(hidden)]
impl uniffi::RustBufferFfiConverter for FfiConverterTypeOrderbookError {
    type RustType = r#OrderbookError;

    

    // For "flat" error enums, we stringify the error on the Rust side and surface that
    // as the error message in the foreign language.


    fn write(obj: r#OrderbookError, buf: &mut std::vec::Vec<u8>) {
        use uniffi::deps::bytes::BufMut;
        let msg = obj.to_string();
        match obj {
            r#OrderbookError::r#InitializeAlreadyCalled{..} => {
                buf.put_i32(1);
                <String as uniffi::FfiConverter>::write(msg, buf);
            },
            r#OrderbookError::r#NotInitialized{..} => {
                buf.put_i32(2);
                <String as uniffi::FfiConverter>::write(msg, buf);
            },
            r#OrderbookError::r#TokenError{..} => {
                buf.put_i32(3);
                <String as uniffi::FfiConverter>::write(msg, buf);
            },
        };
    }
    fn try_read(_buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<r#OrderbookError> {
        panic!("try_read not supported for flat errors");
    }

    
}

impl uniffi::FfiError for FfiConverterTypeOrderbookError { }


// Enum definitions, corresponding to `enum` in UDL.



#[doc(hidden)]
pub struct FfiConverterTypeOrderbookResponse;

#[doc(hidden)]
impl uniffi::RustBufferFfiConverter for FfiConverterTypeOrderbookResponse {
    type RustType = r#OrderbookResponse;

    fn write(obj: Self::RustType, buf: &mut std::vec::Vec<u8>) {
        use uniffi::deps::bytes::BufMut;
        match obj {
            r#OrderbookResponse::r#Initialized {  } => {
                buf.put_i32(1);
                
            },
            r#OrderbookResponse::r#Shutdown {  } => {
                buf.put_i32(2);
                
            },
            r#OrderbookResponse::r#BookSummary { r#summary, } => {
                buf.put_i32(3);
                <FfiConverterTypeSummary as uniffi::FfiConverter>::write(r#summary, buf);
            },
            r#OrderbookResponse::r#Failed { r#reason, } => {
                buf.put_i32(4);
                <String as uniffi::FfiConverter>::write(r#reason, buf);
            },
        };
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<r#OrderbookResponse> {
        use uniffi::deps::bytes::Buf;
        uniffi::check_remaining(buf, 4)?;
        Ok(match buf.get_i32() {
            1 => r#OrderbookResponse::r#Initialized,
            2 => r#OrderbookResponse::r#Shutdown,
            3 => r#OrderbookResponse::r#BookSummary {
                
                r#summary: <FfiConverterTypeSummary as uniffi::FfiConverter>::try_read(buf)?,
            },
            4 => r#OrderbookResponse::r#Failed {
                
                r#reason: <String as uniffi::FfiConverter>::try_read(buf)?,
            },
            v => uniffi::deps::anyhow::bail!("Invalid OrderbookResponse enum value: {}", v),
        })
    }
}


// Record definitions, implemented as method-less structs, corresponding to `dictionary` objects.



#[doc(hidden)]
pub struct FfiConverterTypeLevel;

#[doc(hidden)]
impl uniffi::RustBufferFfiConverter for FfiConverterTypeLevel {
    type RustType = r#Level;

    fn write(obj: r#Level, buf: &mut std::vec::Vec<u8>) {
        // If the provided struct doesn't match the fields declared in the UDL, then
        // the generated code here will fail to compile with somewhat helpful error.
        <String as uniffi::FfiConverter>::write(obj.r#exchange, buf);
        <f64 as uniffi::FfiConverter>::write(obj.r#price, buf);
        <f64 as uniffi::FfiConverter>::write(obj.r#amount, buf);
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<r#Level> {
        Ok(r#Level {
                r#exchange: <String as uniffi::FfiConverter>::try_read(buf)?,
                r#price: <f64 as uniffi::FfiConverter>::try_read(buf)?,
                r#amount: <f64 as uniffi::FfiConverter>::try_read(buf)?,
        })
    }
}



#[doc(hidden)]
pub struct FfiConverterTypeSummary;

#[doc(hidden)]
impl uniffi::RustBufferFfiConverter for FfiConverterTypeSummary {
    type RustType = r#Summary;

    fn write(obj: r#Summary, buf: &mut std::vec::Vec<u8>) {
        // If the provided struct doesn't match the fields declared in the UDL, then
        // the generated code here will fail to compile with somewhat helpful error.
        <f64 as uniffi::FfiConverter>::write(obj.r#spread, buf);
        <std::vec::Vec<FfiConverterTypeLevel> as uniffi::FfiConverter>::write(obj.r#bids, buf);
        <std::vec::Vec<FfiConverterTypeLevel> as uniffi::FfiConverter>::write(obj.r#asks, buf);
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<r#Summary> {
        Ok(r#Summary {
                r#spread: <f64 as uniffi::FfiConverter>::try_read(buf)?,
                r#bids: <std::vec::Vec<FfiConverterTypeLevel> as uniffi::FfiConverter>::try_read(buf)?,
                r#asks: <std::vec::Vec<FfiConverterTypeLevel> as uniffi::FfiConverter>::try_read(buf)?,
        })
    }
}


// Top level functions, corresponding to UDL `namespace` functions.

#[doc(hidden)]
#[no_mangle]
#[allow(clippy::let_unit_value)] // Sometimes we generate code that binds `_retval` to `()`.
pub extern "C" fn r#OrderbookClient_1e03_orderbook_client_initialize(
    
        r#token: uniffi::RustBuffer,
        r#endpoint: uniffi::RustBuffer,
        r#callbacks: u64,
    call_status: &mut uniffi::RustCallStatus
)  {
    // If the provided function does not match the signature specified in the UDL
    // then this attempt to call it will not compile, and will give guidance as to why.
    uniffi::deps::log::debug!("OrderbookClient_1e03_orderbook_client_initialize");
    

uniffi::call_with_result(call_status, || {
    let _retval = r#orderbook_client_initialize(
        match<String as uniffi::FfiConverter>::try_lift(r#token) {
            Ok(val) => val,
            Err(err) => return Err(uniffi::lower_anyhow_error_or_panic::<FfiConverterTypeOrderbookError>(err, "token")),
        },
        match<String as uniffi::FfiConverter>::try_lift(r#endpoint) {
            Ok(val) => val,
            Err(err) => return Err(uniffi::lower_anyhow_error_or_panic::<FfiConverterTypeOrderbookError>(err, "endpoint")),
        },
        match<FfiConverterCallbackInterfaceOnOrderbookClientEvents as uniffi::FfiConverter>::try_lift(r#callbacks) {
            Ok(val) => val,
            Err(err) => return Err(uniffi::lower_anyhow_error_or_panic::<FfiConverterTypeOrderbookError>(err, "callbacks")),
        }).map_err(Into::into).map_err(<FfiConverterTypeOrderbookError as uniffi::FfiConverter>::lower)?;
    Ok(_retval)
})


}


#[doc(hidden)]
#[no_mangle]
#[allow(clippy::let_unit_value)] // Sometimes we generate code that binds `_retval` to `()`.
pub extern "C" fn r#OrderbookClient_1e03_orderbook_client_shutdown(
    
    call_status: &mut uniffi::RustCallStatus
)  {
    // If the provided function does not match the signature specified in the UDL
    // then this attempt to call it will not compile, and will give guidance as to why.
    uniffi::deps::log::debug!("OrderbookClient_1e03_orderbook_client_shutdown");
    

uniffi::call_with_result(call_status, || {
    let _retval = r#orderbook_client_shutdown().map_err(Into::into).map_err(<FfiConverterTypeOrderbookError as uniffi::FfiConverter>::lower)?;
    Ok(_retval)
})


}
// Object definitions, corresponding to UDL `interface` definitions.

// For each Object definition, we assume the caller has provided an appropriately-shaped `struct T`
// with an `impl` for each method on the object. We create an `Arc<T>` for "safely" handing out
// references to these structs to foreign language code, and we provide a `pub extern "C"` function
// corresponding to each method.
//
// (Note that "safely" is in "scare quotes" - that's because we use functions on an `Arc` that
// that are inherently unsafe, but the code we generate is safe in practice.)
//
// If the caller's implementation of the struct does not match with the methods or types specified
// in the UDL, then the rust compiler will complain with a (hopefully at least somewhat helpful!)
// error message when processing this generated code.




// All Object structs must be `Sync + Send`. The generated scaffolding will fail to compile
// if they are not, but unfortunately it fails with an unactionably obscure error message.
// By asserting the requirement explicitly, we help Rust produce a more scrutable error message
// and thus help the user debug why the requirement isn't being met.
uniffi::deps::static_assertions::assert_impl_all!(r#Orderbook: Sync, Send);

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn ffi_OrderbookClient_1e03_Orderbook_object_free(ptr: *const std::os::raw::c_void, call_status: &mut uniffi::RustCallStatus) {
    uniffi::call_with_output(call_status, || {
        assert!(!ptr.is_null());
        drop(unsafe { std::sync::Arc::from_raw(ptr as *const r#Orderbook) })
    })
}
    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn r#OrderbookClient_1e03_Orderbook_new(
    call_status: &mut uniffi::RustCallStatus) -> *const std::os::raw::c_void /* *const Orderbook */ {
        uniffi::deps::log::debug!("OrderbookClient_1e03_Orderbook_new");
        

        // If the constructor does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        

    uniffi::call_with_output(call_status, || {
        let _new = 
    r#Orderbook::r#new();
        let _arc = std::sync::Arc::new(_new);
        <std::sync::Arc<r#Orderbook> as uniffi::FfiConverter>::lower(_arc)
    })


    }
    #[doc(hidden)]
    #[no_mangle]
    #[allow(clippy::let_unit_value)] // Sometimes we generate code that binds `_retval` to `()`.
    pub extern "C" fn r#OrderbookClient_1e03_Orderbook_summary(
        r#ptr: *const std::os::raw::c_void,
    call_status: &mut uniffi::RustCallStatus
    )  {
        uniffi::deps::log::debug!("OrderbookClient_1e03_Orderbook_summary");
        // If the method does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        uniffi::call_with_result(call_status, || {
    let _retval =  r#Orderbook::r#summary(
        match<std::sync::Arc<r#Orderbook> as uniffi::FfiConverter>::try_lift(r#ptr) {
            Ok(ref val) => val,
            Err(err) => return Err(uniffi::lower_anyhow_error_or_panic::<FfiConverterTypeOrderbookError>(err, "ptr")),
        }).map_err(Into::into).map_err(<FfiConverterTypeOrderbookError as uniffi::FfiConverter>::lower)?;
    Ok(_retval)
})

    }



// Callback Interface definitions, corresponding to UDL `callback interface` definitions.


// Register a foreign callback for getting across the FFI.
#[doc(hidden)]
static FOREIGN_CALLBACK_ONORDERBOOKCLIENTEVENTS_INTERNALS: uniffi::ForeignCallbackInternals = uniffi::ForeignCallbackInternals::new();

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn ffi_OrderbookClient_1e03_OnOrderbookClientEvents_init_callback(callback: uniffi::ForeignCallback, _: &mut uniffi::RustCallStatus) {
    FOREIGN_CALLBACK_ONORDERBOOKCLIENTEVENTS_INTERNALS.set_callback(callback);
    // The call status should be initialized to CALL_SUCCESS, so no need to modify it.
}

// Make an implementation which will shell out to the foreign language.
#[doc(hidden)]
#[derive(Debug)]
struct FfiConverterCallbackInterfaceOnOrderbookClientEvents {
  handle: u64
}

impl Drop for FfiConverterCallbackInterfaceOnOrderbookClientEvents {
    fn drop(&mut self) {
        let callback = FOREIGN_CALLBACK_ONORDERBOOKCLIENTEVENTS_INTERNALS.get_callback().unwrap();
        let mut rbuf = uniffi::RustBuffer::new();
        unsafe { callback(self.handle, uniffi::IDX_CALLBACK_FREE, Default::default(), &mut rbuf) };
    }
}

uniffi::deps::static_assertions::assert_impl_all!(FfiConverterCallbackInterfaceOnOrderbookClientEvents: Send);

impl r#OnOrderbookClientEvents for FfiConverterCallbackInterfaceOnOrderbookClientEvents {
    fn r#initialized(&self, 
            r#orderbook: std::sync::Arc<r#Orderbook>){
        let mut args_buf = Vec::new();
        
        <std::sync::Arc<r#Orderbook> as uniffi::FfiConverter>::write(r#orderbook, &mut args_buf);let args_rbuf = uniffi::RustBuffer::from_vec(args_buf);
        let callback = FOREIGN_CALLBACK_ONORDERBOOKCLIENTEVENTS_INTERNALS.get_callback().unwrap();

        unsafe {
            // SAFETY:
            // * We're passing in a pointer to an empty buffer.
            //   * Nothing allocated, so nothing to drop.
            // * We expect the callback to write into that a valid allocated instance of a
            //   RustBuffer.
            let mut ret_rbuf = uniffi::RustBuffer::new();
            let ret = callback(self.handle, 1, args_rbuf, &mut ret_rbuf);
            #[allow(clippy::let_and_return, clippy::let_unit_value)]
            match ret {
                1 => {
                    // 1 indicates success with the return value written to the RustBuffer for
                    //   non-void calls.
                    let result = {
                        
                        uniffi::RustBuffer::destroy(ret_rbuf);
                    };
                    result
                }
                -2 => {
                    // -2 indicates an error written to the RustBuffer
                    panic!("Callback return -2, but throws_type() is None");
                }
                // 0 is a deprecated method to indicates success for void returns
                0 => {
                    uniffi::deps::log::error!("UniFFI: Callback interface returned 0. Please update the bindings code to return 1 for all successful calls");
                    
                    
                }
                // -1 indicates an unexpected error
                
                -1 => {
                    if !ret_rbuf.is_empty() {
                        let reason = match <String as uniffi::FfiConverter>::try_lift(ret_rbuf) {
                            Ok(s) => s,
                            Err(_) => {
                                String::from("[Error reading reason]")
                            }
                        };
                        panic!("callback failed. Reason: {}", reason);
                    } else {
                        panic!("Callback failed")
                    }
                },
                // Other values should never be returned
                _ => panic!("Callback failed with unexpected return code"),
            }
        }
    }
    fn r#shutdown(&self){
        let args_buf = Vec::new();
        let args_rbuf = uniffi::RustBuffer::from_vec(args_buf);
        let callback = FOREIGN_CALLBACK_ONORDERBOOKCLIENTEVENTS_INTERNALS.get_callback().unwrap();

        unsafe {
            // SAFETY:
            // * We're passing in a pointer to an empty buffer.
            //   * Nothing allocated, so nothing to drop.
            // * We expect the callback to write into that a valid allocated instance of a
            //   RustBuffer.
            let mut ret_rbuf = uniffi::RustBuffer::new();
            let ret = callback(self.handle, 2, args_rbuf, &mut ret_rbuf);
            #[allow(clippy::let_and_return, clippy::let_unit_value)]
            match ret {
                1 => {
                    // 1 indicates success with the return value written to the RustBuffer for
                    //   non-void calls.
                    let result = {
                        
                        uniffi::RustBuffer::destroy(ret_rbuf);
                    };
                    result
                }
                -2 => {
                    // -2 indicates an error written to the RustBuffer
                    panic!("Callback return -2, but throws_type() is None");
                }
                // 0 is a deprecated method to indicates success for void returns
                0 => {
                    uniffi::deps::log::error!("UniFFI: Callback interface returned 0. Please update the bindings code to return 1 for all successful calls");
                    
                    
                }
                // -1 indicates an unexpected error
                
                -1 => {
                    if !ret_rbuf.is_empty() {
                        let reason = match <String as uniffi::FfiConverter>::try_lift(ret_rbuf) {
                            Ok(s) => s,
                            Err(_) => {
                                String::from("[Error reading reason]")
                            }
                        };
                        panic!("callback failed. Reason: {}", reason);
                    } else {
                        panic!("Callback failed")
                    }
                },
                // Other values should never be returned
                _ => panic!("Callback failed with unexpected return code"),
            }
        }
    }
    fn r#summary(&self, 
            r#summary: r#Summary){
        let mut args_buf = Vec::new();
        
        <FfiConverterTypeSummary as uniffi::FfiConverter>::write(r#summary, &mut args_buf);let args_rbuf = uniffi::RustBuffer::from_vec(args_buf);
        let callback = FOREIGN_CALLBACK_ONORDERBOOKCLIENTEVENTS_INTERNALS.get_callback().unwrap();

        unsafe {
            // SAFETY:
            // * We're passing in a pointer to an empty buffer.
            //   * Nothing allocated, so nothing to drop.
            // * We expect the callback to write into that a valid allocated instance of a
            //   RustBuffer.
            let mut ret_rbuf = uniffi::RustBuffer::new();
            let ret = callback(self.handle, 3, args_rbuf, &mut ret_rbuf);
            #[allow(clippy::let_and_return, clippy::let_unit_value)]
            match ret {
                1 => {
                    // 1 indicates success with the return value written to the RustBuffer for
                    //   non-void calls.
                    let result = {
                        
                        uniffi::RustBuffer::destroy(ret_rbuf);
                    };
                    result
                }
                -2 => {
                    // -2 indicates an error written to the RustBuffer
                    panic!("Callback return -2, but throws_type() is None");
                }
                // 0 is a deprecated method to indicates success for void returns
                0 => {
                    uniffi::deps::log::error!("UniFFI: Callback interface returned 0. Please update the bindings code to return 1 for all successful calls");
                    
                    
                }
                // -1 indicates an unexpected error
                
                -1 => {
                    if !ret_rbuf.is_empty() {
                        let reason = match <String as uniffi::FfiConverter>::try_lift(ret_rbuf) {
                            Ok(s) => s,
                            Err(_) => {
                                String::from("[Error reading reason]")
                            }
                        };
                        panic!("callback failed. Reason: {}", reason);
                    } else {
                        panic!("Callback failed")
                    }
                },
                // Other values should never be returned
                _ => panic!("Callback failed with unexpected return code"),
            }
        }
    }
    fn r#completed(&self, 
            r#resp: r#OrderbookResponse){
        let mut args_buf = Vec::new();
        
        <FfiConverterTypeOrderbookResponse as uniffi::FfiConverter>::write(r#resp, &mut args_buf);let args_rbuf = uniffi::RustBuffer::from_vec(args_buf);
        let callback = FOREIGN_CALLBACK_ONORDERBOOKCLIENTEVENTS_INTERNALS.get_callback().unwrap();

        unsafe {
            // SAFETY:
            // * We're passing in a pointer to an empty buffer.
            //   * Nothing allocated, so nothing to drop.
            // * We expect the callback to write into that a valid allocated instance of a
            //   RustBuffer.
            let mut ret_rbuf = uniffi::RustBuffer::new();
            let ret = callback(self.handle, 4, args_rbuf, &mut ret_rbuf);
            #[allow(clippy::let_and_return, clippy::let_unit_value)]
            match ret {
                1 => {
                    // 1 indicates success with the return value written to the RustBuffer for
                    //   non-void calls.
                    let result = {
                        
                        uniffi::RustBuffer::destroy(ret_rbuf);
                    };
                    result
                }
                -2 => {
                    // -2 indicates an error written to the RustBuffer
                    panic!("Callback return -2, but throws_type() is None");
                }
                // 0 is a deprecated method to indicates success for void returns
                0 => {
                    uniffi::deps::log::error!("UniFFI: Callback interface returned 0. Please update the bindings code to return 1 for all successful calls");
                    
                    
                }
                // -1 indicates an unexpected error
                
                -1 => {
                    if !ret_rbuf.is_empty() {
                        let reason = match <String as uniffi::FfiConverter>::try_lift(ret_rbuf) {
                            Ok(s) => s,
                            Err(_) => {
                                String::from("[Error reading reason]")
                            }
                        };
                        panic!("callback failed. Reason: {}", reason);
                    } else {
                        panic!("Callback failed")
                    }
                },
                // Other values should never be returned
                _ => panic!("Callback failed with unexpected return code"),
            }
        }
    }
}

unsafe impl uniffi::FfiConverter for FfiConverterCallbackInterfaceOnOrderbookClientEvents {
    // This RustType allows for rust code that inputs this type as a Box<dyn CallbackInterfaceTrait> param
    type RustType = Box<dyn r#OnOrderbookClientEvents>;
    type FfiType = u64;

    // Lower and write are tricky to implement because we have a dyn trait as our type.  There's
    // probably a way to, but this carries lots of thread safety risks, down to impedance
    // mismatches between Rust and foreign languages, and our uncertainty around implementations of
    // concurrent handlemaps.
    //
    // The use case for them is also quite exotic: it's passing a foreign callback back to the foreign
    // language.
    //
    // Until we have some certainty, and use cases, we shouldn't use them.
    fn lower(_obj: Self::RustType) -> Self::FfiType {
        panic!("Lowering CallbackInterface not supported")
    }

    fn write(_obj: Self::RustType, _buf: &mut std::vec::Vec<u8>) {
        panic!("Writing CallbackInterface not supported")
    }

    fn try_lift(v: Self::FfiType) -> uniffi::deps::anyhow::Result<Self::RustType> {
        Ok(Box::new(Self { handle: v }))
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<Self::RustType> {
        use uniffi::deps::bytes::Buf;
        uniffi::check_remaining(buf, 8)?;
        <Self as uniffi::FfiConverter>::try_lift(buf.get_u64())
    }
}


// External and Wrapped types
// Support for external types.

// Types with an external `FfiConverter`...


// For custom scaffolding types we need to generate an FfiConverterType based on the
// UniffiCustomTypeConverter implementation that the library supplies


// The `reexport_uniffi_scaffolding` macro
// Code to re-export the UniFFI scaffolding functions.
//
// Rust won't always re-export the functions from dependencies
// ([rust-lang#50007](https://github.com/rust-lang/rust/issues/50007))
//
// A workaround for this is to have the dependent crate reference a function from its dependency in
// an extern "C" function. This is clearly hacky and brittle, but at least we have some unittests
// that check if this works (fixtures/reexport-scaffolding-macro).
//
// The main way we use this macro is for that contain multiple UniFFI components (libxul,
// megazord).  The combined library has a cargo dependency for each component and calls
// uniffi_reexport_scaffolding!() for each one.

#[doc(hidden)]
pub fn uniffi_reexport_hack() {
}

#[macro_export]
macro_rules! uniffi_reexport_scaffolding {
    () => {
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn OrderbookClient_uniffi_reexport_hack() {
            $crate::uniffi_reexport_hack()
        }
    };
}