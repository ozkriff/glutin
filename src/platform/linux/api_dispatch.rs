use winit;

use Event;
use ContextError;
use CreationError;
use GlAttributes;
use GlContext;
use PixelFormat;
use PixelFormatRequirements;
use WindowAttributes;

use api::wayland;
use api::x11;

#[derive(Clone, Default)]
pub struct PlatformSpecificWindowBuilderAttributes;

pub enum Window {
    #[doc(hidden)]
    X(x11::Window),
    #[doc(hidden)]
    Wayland(wayland::Window)
}

pub use winit::platform::{MonitorId, get_available_monitors, get_primary_monitor};

pub enum PollEventsIterator<'a> {
    // #[doc(hidden)]
    // X(x11::PollEventsIterator<'a>),
    #[doc(hidden)]
    Wayland(wayland::PollEventsIterator<'a>)
}

impl<'a> Iterator for PollEventsIterator<'a> {
    type Item = Event;

    #[inline]
    fn next(&mut self) -> Option<Event> {
        match self {
            // &mut PollEventsIterator::X(ref mut it) => it.next(),
            &mut PollEventsIterator::Wayland(ref mut it) => it.next()
        }
    }
}

pub enum WaitEventsIterator<'a> {
    #[doc(hidden)]
    // X(x11::WaitEventsIterator<'a>),
    #[doc(hidden)]
    Wayland(wayland::WaitEventsIterator<'a>)
}

impl<'a> Iterator for WaitEventsIterator<'a> {
    type Item = Event;

    #[inline]
    fn next(&mut self) -> Option<Event> {
        match self {
            // &mut WaitEventsIterator::X(ref mut it) => it.next(),
            &mut WaitEventsIterator::Wayland(ref mut it) => it.next()
        }
    }
}

impl Window {
    #[inline]
    pub fn new(
        _: &WindowAttributes, // вот это надо бы убрать
        pf_reqs: &PixelFormatRequirements,
        opengl: &GlAttributes<&Window>,
        _: &PlatformSpecificWindowBuilderAttributes, // и это, наверное, тоже убрать
        ozkriff_window: &winit::Window,
    ) -> Result<Window, CreationError> {
        match ozkriff_window.window {
            winit::platform::Window::X(_) => {
                let opengl = opengl.clone().map_sharing(|w| match w {
                    &Window::X(ref w) => w,
                    _ => panic!()       // TODO: return an error
                });
                x11::Window::new(
                    pf_reqs,
                    &opengl,
                    ozkriff_window,
                ).map(Window::X)
            },
            winit::platform::Window::Wayland(_) => {
                let opengl = opengl.clone().map_sharing(|w| match w {
                    &Window::Wayland(ref w) => w,
                    _ => panic!()       // TODO: return an error
                });
                wayland::Window::new(
                    pf_reqs,
                    &opengl,
                    ozkriff_window,
                ).map(Window::Wayland)
            },
        }
    }

    #[inline]
    pub fn poll_events<'a>(&'a self, ozkriff_window: &'a winit::Window) -> PollEventsIterator {
        match self {
            // &Window::X(ref w) => PollEventsIterator::X(w.poll_events()),
            &Window::Wayland(ref w) => PollEventsIterator::Wayland(w.poll_events(ozkriff_window)),
            _ => panic!("123"),
        }
    }

    #[inline]
    pub fn wait_events<'a>(&'a self, ozkriff_window: &'a winit::Window) -> WaitEventsIterator {
        match self {
            // &Window::X(ref w) => WaitEventsIterator::X(w.wait_events()),
            &Window::Wayland(ref w) => WaitEventsIterator::Wayland(w.wait_events(ozkriff_window)),
            _ => panic!("123"),
        }
    }

    #[inline]
    pub fn set_inner_size<'a>(&'a self, x: u32, y: u32, ozkriff_window: &'a winit::Window) {
        match self {
            // &Window::X(ref w) => w.set_inner_size(x, y),
            &Window::Wayland(ref w) => w.set_inner_size(x, y, ozkriff_window),
            _ => panic!("123"),
        }
    }

}

impl GlContext for Window {
    #[inline]
    unsafe fn make_current(&self) -> Result<(), ContextError> {
        match self {
            &Window::X(ref w) => w.make_current(),
            &Window::Wayland(ref w) => w.make_current()
        }
    }

    #[inline]
    fn is_current(&self) -> bool {
        match self {
            &Window::X(ref w) => w.is_current(),
            &Window::Wayland(ref w) => w.is_current()
        }
    }

    #[inline]
    fn get_proc_address(&self, addr: &str) -> *const () {
        match self {
            &Window::X(ref w) => w.get_proc_address(addr),
            &Window::Wayland(ref w) => w.get_proc_address(addr)
        }
    }

    #[inline]
    fn swap_buffers(&self) -> Result<(), ContextError> {
        match self {
            &Window::X(ref w) => w.swap_buffers(),
            &Window::Wayland(ref w) => w.swap_buffers()
        }
    }

    #[inline]
    fn get_api(&self) -> ::Api {
        match self {
            &Window::X(ref w) => w.get_api(),
            &Window::Wayland(ref w) => w.get_api()
        }
    }

    #[inline]
    fn get_pixel_format(&self) -> PixelFormat {
        match self {
            &Window::X(ref w) => w.get_pixel_format(),
            &Window::Wayland(ref w) => w.get_pixel_format()
        }
    }
}
