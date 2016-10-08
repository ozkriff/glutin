#![cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "openbsd"))]

use std::ffi::CString;
use winit;
use {ContextError, CreationError, GlAttributes, GlContext, PixelFormat, PixelFormatRequirements, Event};
use api::dlopen;
use api::egl;
use api::egl::Context as EglContext;
use wayland_client::egl as wegl;
use winit::api::wayland::{ShellWindow, substract_borders};

pub struct Window {
    egl_surface: wegl::WlEglSurface,
    context: EglContext,
}

impl Window {
    pub fn new(
        pf_reqs: &PixelFormatRequirements,
        opengl: &GlAttributes<&Window>,
        winit_window: &winit::Window,
    ) -> Result<Window, CreationError> {
        let winit_wayland: &winit::api::wayland::Window = match winit_window.window {
            winit::platform::Window::X(_) => unimplemented!(),
            winit::platform::Window::Wayland(ref w) => w,
        };
        let (surface, _) = match winit_wayland.wayland_context.new_surface() {
            Some(t) => t,
            None => return Err(CreationError::NotSupported)
        };
        let (w, h) = winit_wayland.get_inner_size().unwrap();
        let egl_surface = wegl::WlEglSurface::new(surface, w, h);
        let context = {
            let libegl = unsafe { dlopen::dlopen(b"libEGL.so\0".as_ptr() as *const _, dlopen::RTLD_NOW) };
            if libegl.is_null() {
                return Err(CreationError::NotSupported);
            }
            let egl = ::api::egl::ffi::egl::Egl::load_with(|sym| {
                let sym = CString::new(sym).unwrap();
                unsafe { dlopen::dlsym(libegl, sym.as_ptr()) }
            });
            try!(EglContext::new(
                egl,
                pf_reqs, &opengl.clone().map_sharing(|_| unimplemented!()),        // TODO: 
                egl::NativeDisplay::Wayland(Some(winit_wayland.wayland_context.display_ptr() as *const _)))
                .and_then(|p| p.finish(unsafe { egl_surface.egl_surfaceptr() } as *const _))
            )
        };
        Ok(Window {
            egl_surface: egl_surface,
            context: context,
        })
    }

    pub fn next_event(&self, wayland_window: &winit::api::wayland::Window) -> Option<Event> {
        use wayland_client::Event as WEvent;
        use wayland_client::wayland::WaylandProtocolEvent;
        use wayland_client::wayland::shell::WlShellSurfaceEvent;

        let mut newsize = None;
        let mut evt_queue_guard = wayland_window.evt_queue.lock().unwrap();

        let mut shell_window_guard = wayland_window.shell_window.lock().unwrap();
        match *shell_window_guard {
            ShellWindow::Decorated(ref mut deco) => {
                for (_, w, h) in deco {
                    newsize = Some((w, h));
                }
            },
            ShellWindow::Plain(ref plain, ref mut evtiter) => {
                for evt in evtiter {
                    if let WEvent::Wayland(WaylandProtocolEvent::WlShellSurface(_, ssevt)) = evt {
                        match ssevt {
                            WlShellSurfaceEvent::Ping(u) => {
                                plain.pong(u);
                            },
                            WlShellSurfaceEvent::Configure(_, w, h) => {
                                newsize = Some((w, h));
                            },
                            _ => {}
                        }
                    }
                }
            }
        }

        if let Some((w, h)) = newsize {
            let (w, h) = substract_borders(w, h);
            *wayland_window.inner_size.lock().unwrap() = (w, h);
            if let ShellWindow::Decorated(ref mut deco) = *shell_window_guard {
                deco.resize(w, h);
            }
            self.egl_surface.resize(w, h, 0, 0);
            if let Some(f) = wayland_window.resize_callback {
                f(w as u32, h as u32);
            }
            Some(Event::Resized(w as u32, h as u32))
        } else {
            evt_queue_guard.pop_front()
        }
    }


    #[inline]
    pub fn poll_events<'a>(&'a self, winit_window: &'a winit::Window) -> PollEventsIterator {
        let winit_wayland: &winit::api::wayland::Window = match winit_window.window {
            winit::platform::Window::X(_) => unimplemented!(),
            winit::platform::Window::Wayland(ref w) => w,
        };
        PollEventsIterator {
            window: self,
            winit_window: winit_wayland,
        }
    }

    #[inline]
    pub fn wait_events<'a>(&'a self, winit_window: &'a winit::Window) -> WaitEventsIterator {
        let winit_wayland: &winit::api::wayland::Window = match winit_window.window {
            winit::platform::Window::X(_) => unimplemented!(),
            winit::platform::Window::Wayland(ref w) => w,
        };
        WaitEventsIterator {
            window: self,
            winit_window: winit_wayland,
        }
    }

    #[inline]
    pub fn set_inner_size<'a>(&'a self, x: u32, y: u32, winit_window: &'a winit::Window) {
        let winit_wayland: &winit::api::wayland::Window = match winit_window.window {
            winit::platform::Window::X(_) => unimplemented!(),
            winit::platform::Window::Wayland(ref w) => w,
        };
        let mut guard = winit_wayland.shell_window.lock().unwrap();
        match *guard {
            ShellWindow::Decorated(ref mut deco) => { deco.resize(x as i32, y as i32); },
            _ => {}
        }
        self.egl_surface.resize(x as i32, y as i32, 0, 0)
    }
}

pub struct PollEventsIterator<'a> {
    window: &'a Window,
    winit_window: &'a winit::api::wayland::Window,
}

impl<'a> Iterator for PollEventsIterator<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        match self.window.next_event(self.winit_window) {
            Some(evt) => return Some(evt),
            None => {}
        }
        // the queue was empty, try a dispatch and see the result
        self.winit_window.wayland_context.dispatch_events();
        return self.window.next_event(self.winit_window);
    }
}

pub struct WaitEventsIterator<'a> {
    window: &'a Window,
    winit_window: &'a winit::api::wayland::Window,
}

impl<'a> Iterator for WaitEventsIterator<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        loop {
            match self.window.next_event(self.winit_window) {
                Some(evt) => return Some(evt),
                None => {}
            }
            // the queue was empty, try a dispatch & read and see the result
            self.winit_window.wayland_context.flush_events().expect("Connexion with the wayland compositor lost.");
            match self.winit_window.wayland_context.read_events() {
                Ok(_) => {
                    // events were read or dispatch is needed, in both cases, we dispatch
                    self.winit_window.wayland_context.dispatch_events()
                }
                Err(_) => panic!("Connexion with the wayland compositor lost.")
            }
        }
    }
}

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
    fn get_api(&self) -> ::Api {
        self.context.get_api()
    }

    #[inline]
    fn get_pixel_format(&self) -> PixelFormat {
        self.context.get_pixel_format().clone()
    }
}
