/*!

This example demonstrates how to manually create a glium context with any backend you want, most
notably without glutin.

There are three concepts in play:

 - The `Backend` trait, which defines how glium interfaces with the OpenGL context
   provider (glutin, SDL, glfw, etc.).

 - The `Context` struct, which is the main object of glium. The context also provides
   OpenGL-related functions like `get_free_video_memory` or `get_supported_glsl_version`.

 - The `Facade` trait, which is the trait required to be implemented on objects that you pass
   to functions like `VertexBuffer::new`. This trait is implemented on `Rc<Context>`, which
   means that you can direct pass the context.

*/
extern crate glium;

use glium::Surface;
#[allow(unused_imports)]
use glium::glutin::{self, ContextTrait};

use std::rc::Rc;
use std::os::raw::c_void;

fn main() {
    // building the glutin window
    // note that it's just `build` and not `build_glium`
    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let gl_window = Rc::new(cb.build_windowed(wb, &events_loop).unwrap());

    // in order to create our context, we will need to provide an object which implements
    // the `Backend` trait
    struct Backend {
        gl_window: Rc<glutin::WindowedContext>,
    }

    unsafe impl glium::backend::Backend for Backend {
        fn swap_buffers(&self) -> Result<(), glium::SwapBuffersError> {
            match self.gl_window.swap_buffers() {
                Ok(()) => Ok(()),
                Err(glutin::ContextError::IoError(_)) => panic!(),
                Err(glutin::ContextError::OsError(_)) => panic!(),
                Err(glutin::ContextError::ContextLost) => Err(glium::SwapBuffersError::ContextLost),
            }
        }

        // this function is called only after the OpenGL context has been made current
        unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
            self.gl_window.get_proc_address(symbol) as *const _
        }

        // this function is used to adjust the viewport when the user wants to draw or blit on
        // the whole window
        fn get_framebuffer_dimensions(&self) -> (u32, u32) {
            // we default to a dummy value is the window no longer exists
            self.gl_window.get_inner_size().map(Into::into).unwrap_or((128, 128))
        }

        fn is_current(&self) -> bool {
            // if you are using a library that doesn't provide an equivalent to `is_current`, you
            // can just put `unimplemented!` and pass `false` when you create
            // the `Context` (see below)
            self.gl_window.is_current()
        }

        unsafe fn make_current(&self) {
            self.gl_window.make_current().unwrap();
        }
    }

    // now building the context
    let context = unsafe {
        // The first parameter is our backend.
        //
        // The second parameter tells glium whether or not it should regularly call `is_current`
        // on the backend to make sure that the OpenGL context is still the current one.
        //
        // It is recommended to pass `true`, but you can pass `false` if you are sure that no
        // other OpenGL context will be made current in this thread.
        let backend = Backend { gl_window: gl_window.clone() };
        glium::backend::Context::new(backend, true, Default::default())
    }.unwrap();

    // drawing a frame to prove that it works
    // note that constructing a `Frame` object manually is a bit hacky and may be changed
    // in the future
    let mut target = glium::Frame::new(context.clone(), context.get_framebuffer_dimensions());
    target.clear_color(0.0, 1.0, 0.0, 1.0);
    target.finish().unwrap();

    // the window is still available
    events_loop.run_forever(|event| {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => return glutin::ControlFlow::Break,
                _ => (),
            },
            _ => (),
        }

        glutin::ControlFlow::Continue
    });
}
