#![cfg(target_os = "android")]

extern crate android_glue;

use CreationError::{self, OsError};

use winit;

use Api;
use ContextError;
use GlAttributes;
use GlContext;
use PixelFormat;
use PixelFormatRequirements;
use WindowAttributes;

use api::egl;
use api::egl::Context as EglContext;

mod ffi;

pub struct Window {
    context: EglContext,
}

#[derive(Clone, Default)]
pub struct PlatformSpecificWindowBuilderAttributes;

#[derive(Clone, Default)]
pub struct PlatformSpecificHeadlessBuilderAttributes;

impl Window {
    pub fn new(
        _: &WindowAttributes,
        pf_reqs: &PixelFormatRequirements,
        opengl: &GlAttributes<&Window>,
        _: &PlatformSpecificWindowBuilderAttributes,
        ozkriff_window: &winit::Window,
    ) -> Result<Window, CreationError> {
        let opengl = opengl.clone().map_sharing(|w| &w.context);
        let native_window = unsafe { android_glue::get_native_window() };
        if native_window.is_null() {
            return Err(OsError(format!("Android's native window is null")));
        }
        let context = try!(EglContext::new(
            egl::ffi::egl::Egl,
            pf_reqs,
            &opengl,
            egl::NativeDisplay::Android
        ).and_then(|p| p.finish(native_window as *const _)));
        Ok(Window {
            context: context,
        })
    }
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl GlContext for Window {
    #[inline]
    unsafe fn make_current(&self) -> Result<(), ContextError> {
        self.context.make_current()
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.context.is_current()
    }

    #[inline]
    fn get_proc_address(&self, addr: &str) -> *const () {
        self.context.get_proc_address(addr)
    }

    #[inline]
    fn swap_buffers(&self) -> Result<(), ContextError> {
        self.context.swap_buffers()
    }

    #[inline]
    fn get_api(&self) -> Api {
        self.context.get_api()
    }

    #[inline]
    fn get_pixel_format(&self) -> PixelFormat {
        self.context.get_pixel_format()
    }
}

#[derive(Clone)]
pub struct WindowProxy;

impl WindowProxy {
    #[inline]
    pub fn wakeup_event_loop(&self) {
        unimplemented!()
    }
}

pub struct HeadlessContext(EglContext);

impl HeadlessContext {
    /// See the docs in the crate root file.
    pub fn new(dimensions: (u32, u32), pf_reqs: &PixelFormatRequirements,
               opengl: &GlAttributes<&HeadlessContext>,
               _: &PlatformSpecificHeadlessBuilderAttributes)
               -> Result<HeadlessContext, CreationError>
    {
        let opengl = opengl.clone().map_sharing(|c| &c.0);
        let context = try!(EglContext::new(egl::ffi::egl::Egl, pf_reqs, &opengl,
                           egl::NativeDisplay::Android));
        let context = try!(context.finish_pbuffer(dimensions));     // TODO: 
        Ok(HeadlessContext(context))
    }
}

unsafe impl Send for HeadlessContext {}
unsafe impl Sync for HeadlessContext {}

impl GlContext for HeadlessContext {
    #[inline]
    unsafe fn make_current(&self) -> Result<(), ContextError> {
        self.0.make_current()
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.0.is_current()
    }

    #[inline]
    fn get_proc_address(&self, addr: &str) -> *const () {
        self.0.get_proc_address(addr)
    }

    #[inline]
    fn swap_buffers(&self) -> Result<(), ContextError> {
        self.0.swap_buffers()
    }

    #[inline]
    fn get_api(&self) -> Api {
        self.0.get_api()
    }

    #[inline]
    fn get_pixel_format(&self) -> PixelFormat {
        self.0.get_pixel_format()
    }
}
