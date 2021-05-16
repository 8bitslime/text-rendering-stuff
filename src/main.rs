mod font;
use font::{cache::GlyphCache, font_from_file};
use fontdue::layout::{LayoutSettings, TextStyle};

mod shader;

use std::{collections::HashMap, marker::PhantomData};
use glutin::{
    Api::OpenGl,
    Context,
    ContextWrapper,
    ContextBuilder,
    CreationError,
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    GlRequest::{self, Specific},
    PossiblyCurrent,
    window::{WindowBuilder, WindowId, Window}
};
use gl::types::*;

const OPENGL_VERSION: GlRequest = Specific(OpenGl, (3, 3));

struct WindowManager<T> {
    shared_context: Context<PossiblyCurrent>,
    windows: HashMap<WindowId, ContextWrapper<PossiblyCurrent, Window>>,
    _phantom: PhantomData<T>
}

impl<T> WindowManager<T> {
    fn new(el: &EventLoopWindowTarget<T>) -> Result<Self, CreationError> {
        let shared_context = ContextBuilder::new()
            .with_depth_buffer(0)
            .with_gl(OPENGL_VERSION)
            .build_headless(el, PhysicalSize::new(0, 0))?;
        
        let shared_context = unsafe { shared_context.make_current().unwrap() };
        gl::load_with(|ptr| shared_context.get_proc_address(ptr));
        
        Ok(Self {
            shared_context,
            windows: HashMap::new(),
            _phantom: PhantomData::default()
        })
    }
    
    fn craete_window(&mut self, wb: WindowBuilder, el: &EventLoopWindowTarget<T>) -> Result<WindowId, CreationError> {
        let context = ContextBuilder::new()
            .with_gl(OPENGL_VERSION)
            .with_shared_lists(&self.shared_context)
            .build_windowed(wb, el)?;
        let id = context.window().id();
        self.windows.insert(context.window().id(), unsafe { context.make_current().unwrap() });
        Ok(id)
    }
    
    fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }
    
    fn take_window(&mut self, id: &WindowId) -> Option<ContextWrapper<PossiblyCurrent, Window>> {
        self.windows.remove(id)
    }
    
    // fn insert_window(&mut self, id: WindowId, context: ContextWrapper<NotCurrent, Window>) {
    //     self.windows.insert(id, context);
    // }
    
    #[inline(always)]
    fn render_with<F>(&mut self, id: &WindowId, callback: F)
        where F: FnOnce(&ContextWrapper<PossiblyCurrent, Window>)
    {
        //TODO: replace with Cell lmao
        if let Some(context) = self.windows.remove(id) {
            let context = unsafe { context.make_current().unwrap() };
            callback(&context);
            self.windows.insert(context.window().id(), context);
        }
    }
    
    fn resize_context(&self, id: &WindowId, size: PhysicalSize<u32>) {
        if let Some(context) = self.windows.get(id) {
            context.resize(size);
        }
    }
}

const SCALE: f32 = 23.4;
fn main() {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aliquam condimentum nibh sit amet metus pulvinar cursus. Vivamus malesuada ipsum quis lectus rutrum aliquam. Ut dignissim venenatis dictum. Suspendisse non felis ac metus pellentesque laoreet eget ac sapien. Fusce at diam et dui semper tincidunt et et justo. Suspendisse sagittis facilisis sollicitudin. In quis placerat metus. Mauris ipsum nunc, tempus id dapibus ut, pharetra vel massa. Integer tristique viverra faucibus. Cras semper sed justo et cursus. Praesent faucibus arcu nec nunc aliquet, quis ultricies mauris tempus. Donec nisi dolor, suscipit sit amet urna id, vulputate faucibus felis. Nunc dictum, metus at accumsan imperdiet, sapien felis aliquam libero, vel auctor libero velit eget orci. Nullam nec nulla et erat lacinia egestas. Nunc tincidunt purus vitae eros pretium, non pulvinar mi cursus.";
    let font = font_from_file("res/Roboto-Regular.ttf", fontdue::FontSettings {
        scale: SCALE,
        .. Default::default()
    }).unwrap();
    let fonts = [font];
    let mut layout = fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
    layout.reset(&fontdue::layout::LayoutSettings {
        max_width: Some(640.0),
        wrap_style: fontdue::layout::WrapStyle::Letter,
        .. Default::default()
    });
    layout.append(&fonts, &fontdue::layout::TextStyle::new(text, SCALE, 0));
    
    let mut cache = GlyphCache::new(1024, 1024, font::cache::AtlasFormat::Subpixel);
    cache.cache(&fonts, layout.glyphs());
    
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Hello World!")
        .with_inner_size(LogicalSize::new(640, 480));
    
    let mut window_manager = WindowManager::new(&el).expect("Failed to create window manager");
    window_manager.craete_window(wb, &el).expect("Failed to create window");
    
    let (vao, program, uv_rect, location, window_size) = unsafe {
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
        
        let data: &[f32; 12] = &[
            0.0, 1.0, // TL
            1.0, 1.0, // TR
            1.0, 0.0, // BR
            0.0, 1.0, // TL
            1.0, 0.0, // BR
            0.0, 0.0, // BL
        ];
        
        let program = {
            let vertex = shader::Shader::from_file("res/vertex.glsl", shader::Stage::Vertex).unwrap();
            let fragment = shader::Shader::from_file("res/fragment.glsl", shader::Stage::Fragment).unwrap();
            shader::Program::from_shaders(&[vertex, fragment]).unwrap()
        };
        gl::UseProgram(program.0);
        let uv_rect = gl::GetUniformLocation(program.0, "uv_rect\0".as_ptr() as _);
        let location = gl::GetUniformLocation(program.0, "location\0".as_ptr() as _);
        let window_size = gl::GetUniformLocation(program.0, "window_size\0".as_ptr() as _);
        
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        
        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(data) as GLsizeiptr,
            data.as_ptr() as _,
            gl::STATIC_DRAW
        );
        
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, (std::mem::size_of::<f32>() * 2) as GLsizei, 0 as _);
        
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, 1024, 1024, 0, gl::RGB, gl::UNSIGNED_BYTE, cache.get_image().as_ptr() as _);
        
        (vao, program, uv_rect, location, window_size)
    };
    
    el.run(move |event, _el, control_flow| {
        *control_flow = ControlFlow::Wait;
        if window_manager.is_empty() {
            *control_flow = ControlFlow::Exit;
        }
        
        match event {
            Event::WindowEvent { event, window_id } => {
                match event {
                    WindowEvent::Resized(size) => {
                        window_manager.resize_context(&window_id, size);
                        layout.reset(&LayoutSettings {
                            max_width: Some(size.width as f32),
                            wrap_style: fontdue::layout::WrapStyle::Letter,
                            .. Default::default()
                        });
                        layout.append(&fonts, &TextStyle::new(text, SCALE, 0));
                    }
                    
                    WindowEvent::CloseRequested => {
                        window_manager.take_window(&window_id);
                    },
                    
                    _ => {},
                }
            },
            
            Event::RedrawRequested(window_id) => {
                window_manager.render_with(&window_id, |context| unsafe {
                    let size = context.window().inner_size();
                    gl::Viewport(
                        0, 0,
                        size.width as i32,
                        size.height as i32
                    );
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    
                    gl::UseProgram(program.0);
                    gl::BindVertexArray(vao);
                    gl::Uniform2f(window_size, size.width as f32, size.height as f32);
                    
                    for glyph in layout.glyphs() {
                        if let Some(uv) = cache.get_uv(&glyph.key) {
                            gl::Uniform4f(uv_rect, uv.width, uv.height, uv.x, uv.y);
                            gl::Uniform4f(location, glyph.width as f32, glyph.height as f32, glyph.x, glyph.y);
                            gl::DrawArrays(gl::TRIANGLES, 0, 6);
                        }
                    }
                    
                    context.swap_buffers().unwrap();
                });
            },
            
            _ => {},
        }
    });
}
