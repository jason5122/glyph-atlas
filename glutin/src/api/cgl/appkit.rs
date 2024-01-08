//! The parts of AppKit related to OpenGL.
//!
//! TODO: Move this to another crate.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::ops::Deref;

use dispatch::Queue;
use objc2::encode::{Encoding, RefEncode};
use objc2::foundation::{is_main_thread, NSInteger, NSObject};
use objc2::rc::{Id, Shared};
use objc2::{extern_class, extern_methods, msg_send_id, ClassType};

pub type GLint = i32;

pub enum CGLContextObj {}

// XXX borrowed from winit.

// Unsafe wrapper type that allows us to dispatch things that aren't Send.
// This should *only* be used to dispatch to the main queue.
// While it is indeed not guaranteed that these types can safely be sent to
// other threads, we know that they're safe to use on the main thread.
pub(crate) struct MainThreadSafe<T>(pub(crate) T);

unsafe impl<T> Send for MainThreadSafe<T> {}

impl<T> Deref for MainThreadSafe<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

/// Run closure on the main thread.
pub(crate) fn run_on_main<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    if is_main_thread() {
        f()
    } else {
        Queue::main().exec_sync(f)
    }
}

unsafe impl RefEncode for CGLContextObj {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("_CGLContextObject", &[]));
}

extern_class!(
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub(crate) struct NSOpenGLContext;

    unsafe impl ClassType for NSOpenGLContext {
        type Super = NSObject;
    }
);

unsafe impl Send for NSOpenGLContext {}
unsafe impl Sync for NSOpenGLContext {}

extern_methods!(
    unsafe impl NSOpenGLContext {
        pub(crate) fn currentContext() -> Option<Id<Self, Shared>> {
            unsafe { msg_send_id![Self::class(), currentContext] }
        }

        pub(crate) fn newWithFormat_shareContext(
            format: &NSOpenGLPixelFormat,
            share: Option<&NSOpenGLContext>,
        ) -> Option<Id<Self, Shared>> {
            unsafe {
                msg_send_id![
                    msg_send_id![Self::class(), alloc],
                    initWithFormat: format,
                    shareContext: share,
                ]
            }
        }

        #[sel(clearCurrentContext)]
        pub(crate) fn clearCurrentContext();

        #[sel(makeCurrentContext)]
        pub(crate) fn makeCurrentContext(&self);

        #[sel(update)]
        pub(crate) fn update(&self);

        #[sel(flushBuffer)]
        pub(crate) fn flushBuffer(&self);

        pub(crate) fn view(&self) -> Option<Id<NSObject, Shared>> {
            unsafe { msg_send_id![self, view] }
        }

        #[sel(setView:)]
        pub(crate) unsafe fn setView(&self, view: Option<&NSObject>);

        #[sel(setValues:forParameter:)]
        pub(crate) unsafe fn setValues_forParameter(
            &self,
            vals: *const GLint,
            param: NSOpenGLContextParameter,
        );

        #[sel(CGLContextObj)]
        pub(crate) fn CGLContextObj(&self) -> *mut CGLContextObj;
    }
);

extern_class!(
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct NSOpenGLPixelFormat;

    unsafe impl ClassType for NSOpenGLPixelFormat {
        type Super = NSObject;
    }
);

unsafe impl Send for NSOpenGLPixelFormat {}
unsafe impl Sync for NSOpenGLPixelFormat {}

extern_methods!(
    unsafe impl NSOpenGLPixelFormat {
        pub unsafe fn newWithAttributes(
            attrs: &[NSOpenGLPixelFormatAttribute],
        ) -> Option<Id<Self, Shared>> {
            unsafe {
                msg_send_id![
                    msg_send_id![Self::class(), alloc],
                    initWithAttributes: attrs.as_ptr(),
                ]
            }
        }

        #[sel(getValues:forAttribute:forVirtualScreen:)]
        pub(crate) unsafe fn getValues_forAttribute_forVirtualScreen(
            &self,
            vals: *mut GLint,
            param: NSOpenGLPixelFormatAttribute,
            screen: GLint,
        );
    }
);

type NSOpenGLContextParameter = NSInteger;
pub(crate) const NSOpenGLCPSwapInterval: NSOpenGLContextParameter = 222;
pub(crate) const NSOpenGLCPSurfaceOrder: NSOpenGLContextParameter = 235;
pub(crate) const NSOpenGLCPSurfaceOpacity: NSOpenGLContextParameter = 236;
pub(crate) const NSOpenGLCPSurfaceBackingSize: NSOpenGLContextParameter = 304;
pub(crate) const NSOpenGLCPReclaimResources: NSOpenGLContextParameter = 308;
pub(crate) const NSOpenGLCPCurrentRendererID: NSOpenGLContextParameter = 309;
pub(crate) const NSOpenGLCPGPUVertexProcessing: NSOpenGLContextParameter = 310;
pub(crate) const NSOpenGLCPGPUFragmentProcessing: NSOpenGLContextParameter = 311;
pub(crate) const NSOpenGLCPHasDrawable: NSOpenGLContextParameter = 314;
pub(crate) const NSOpenGLCPMPSwapsInFlight: NSOpenGLContextParameter = 315;

pub type NSOpenGLPixelFormatAttribute = u32;
pub const NSOpenGLPFAAllRenderers: NSOpenGLPixelFormatAttribute = 1;
pub const NSOpenGLPFATripleBuffer: NSOpenGLPixelFormatAttribute = 3;
pub const NSOpenGLPFADoubleBuffer: NSOpenGLPixelFormatAttribute = 5;
pub const NSOpenGLPFAStereo: NSOpenGLPixelFormatAttribute = 6;
pub const NSOpenGLPFAAuxBuffers: NSOpenGLPixelFormatAttribute = 7;
pub const NSOpenGLPFAColorSize: NSOpenGLPixelFormatAttribute = 8;
pub const NSOpenGLPFAAlphaSize: NSOpenGLPixelFormatAttribute = 11;
pub const NSOpenGLPFADepthSize: NSOpenGLPixelFormatAttribute = 12;
pub const NSOpenGLPFAStencilSize: NSOpenGLPixelFormatAttribute = 13;
pub const NSOpenGLPFAAccumSize: NSOpenGLPixelFormatAttribute = 14;
pub const NSOpenGLPFAMinimumPolicy: NSOpenGLPixelFormatAttribute = 51;
pub const NSOpenGLPFAMaximumPolicy: NSOpenGLPixelFormatAttribute = 52;
pub const NSOpenGLPFAOffScreen: NSOpenGLPixelFormatAttribute = 53;
pub const NSOpenGLPFAFullScreen: NSOpenGLPixelFormatAttribute = 54;
pub const NSOpenGLPFASampleBuffers: NSOpenGLPixelFormatAttribute = 55;
pub const NSOpenGLPFASamples: NSOpenGLPixelFormatAttribute = 56;
pub const NSOpenGLPFAAuxDepthStencil: NSOpenGLPixelFormatAttribute = 57;
pub const NSOpenGLPFAColorFloat: NSOpenGLPixelFormatAttribute = 58;
pub const NSOpenGLPFAMultisample: NSOpenGLPixelFormatAttribute = 59;
pub const NSOpenGLPFASupersample: NSOpenGLPixelFormatAttribute = 60;
pub const NSOpenGLPFASampleAlpha: NSOpenGLPixelFormatAttribute = 61;
pub const NSOpenGLPFARendererID: NSOpenGLPixelFormatAttribute = 70;
pub const NSOpenGLPFASingleRenderer: NSOpenGLPixelFormatAttribute = 71;
pub const NSOpenGLPFANoRecovery: NSOpenGLPixelFormatAttribute = 72;
pub const NSOpenGLPFAAccelerated: NSOpenGLPixelFormatAttribute = 73;
pub const NSOpenGLPFAClosestPolicy: NSOpenGLPixelFormatAttribute = 74;
pub const NSOpenGLPFARobust: NSOpenGLPixelFormatAttribute = 75;
pub const NSOpenGLPFABackingStore: NSOpenGLPixelFormatAttribute = 76;
pub const NSOpenGLPFAMPSafe: NSOpenGLPixelFormatAttribute = 78;
pub const NSOpenGLPFAWindow: NSOpenGLPixelFormatAttribute = 80;
pub const NSOpenGLPFAMultiScreen: NSOpenGLPixelFormatAttribute = 81;
pub const NSOpenGLPFACompliant: NSOpenGLPixelFormatAttribute = 83;
pub const NSOpenGLPFAScreenMask: NSOpenGLPixelFormatAttribute = 84;
pub const NSOpenGLPFAPixelBuffer: NSOpenGLPixelFormatAttribute = 90;
pub const NSOpenGLPFARemotePixelBuffer: NSOpenGLPixelFormatAttribute = 91;
pub const NSOpenGLPFAAllowOfflineRenderers: NSOpenGLPixelFormatAttribute = 96;
pub const NSOpenGLPFAAcceleratedCompute: NSOpenGLPixelFormatAttribute = 97;
pub const NSOpenGLPFAOpenGLProfile: NSOpenGLPixelFormatAttribute = 99;
pub const NSOpenGLPFAVirtualScreenCount: NSOpenGLPixelFormatAttribute = 128;
pub const NSOpenGLProfileVersionLegacy: NSOpenGLPixelFormatAttribute = 0x1000;
pub const NSOpenGLProfileVersion3_2Core: NSOpenGLPixelFormatAttribute = 0x3200;
pub const NSOpenGLProfileVersion4_1Core: NSOpenGLPixelFormatAttribute = 0x4100;
