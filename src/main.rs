use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateFlags, BufferCreateInfo, BufferReadGuard, BufferUsage, Subbuffer}, device::{physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFamilyProperties, QueueFlags}, format::{self, Format}, image::{Image, ImageUsage}, instance::{Instance, InstanceCreateInfo}, library::VulkanLibrary, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::graphics::vertex_input::Vertex, swapchain::{ColorSpace, CompositeAlpha, PresentMode, Surface, SurfaceInfo, Swapchain, SwapchainCreateFlags, SwapchainCreateInfo}
};
use winit::{application::ApplicationHandler, event_loop::{ActiveEventLoop, EventLoop}, window::{self, Window, WindowAttributes}};
use std::sync::Arc;

fn main() {
    //1 Connect to GPU
    

    let event_loop: EventLoop<_> = EventLoop::new().unwrap();



    //2 create window


    //3 create render pass
        //render pass
        //frame buffers

    //4 create vertex buffer


    //5 create pipeline
        //compile shaders
        //subpasses

    //6 create command buffers
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct Vert {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2]
}

struct Application {
    vk_driver : Arc<Instance>,
    vk_GPU : Option<(Arc<PhysicalDevice>, Vec<u32>)>,
    vk_virtual_GPU : Option<(Arc<Device>, Vec<Arc<Queue>>)>,

    window_create_info : WindowAttributes,
    window_main : Option<(Arc<Window>, Arc<Surface>)>,
    swapchain : Option<(Arc<Swapchain>, Vec<Arc<Image>>)>,

    memory_allocator : Option<Arc<dyn MemoryAllocator>>,
    vert_buffer : Option<Subbuffer<[Vert]>>
}

impl Application {
    fn CreateWindow(&mut self, event_loop : &ActiveEventLoop) {
        let new_window = Arc::new(event_loop.create_window(self.window_create_info.clone())
        .unwrap());

        //https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html
        //Says that making a surface is platform specific, so we might need to do conditional compilation for certain platforms
        let window_surface = Surface::from_window(self.vk_driver.clone(), new_window.clone())
                .unwrap();

        self.window_main = Some((new_window, window_surface))
    }

    fn InitGPU(&mut self) {
        let device_extensions = DeviceExtensions{
            khr_swapchain: true,
            ..Default::default()
        };
        let surface = self.window_main.as_ref().unwrap().1.clone();

        let device = LocateDevice(self.vk_driver.clone(), &device_extensions, surface).unwrap();
    }

    fn InitVirtualGPU(&mut self) {
        let (gpu, queue_indices) = self.vk_GPU.as_ref().unwrap().clone();
        let virtual_gpu_params : DeviceCreateInfo;
        
        let vk_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let device_features = Features {
            ..Default::default()
        };

        let queue_params = QueueCreateInfo {
            queue_family_index: queue_indices[0],
            ..Default::default()
        };

        virtual_gpu_params = DeviceCreateInfo {
            enabled_extensions: vk_extensions,
            enabled_features: device_features,
            queue_create_infos: vec![queue_params],
            ..Default::default()
        };

        let (device, queues_iter) = Device::new(gpu, virtual_gpu_params).unwrap();

        let queues: Vec<Arc<Queue>> = queues_iter.collect();

        self.vk_virtual_GPU = Some((device, queues))
    }

    fn BuildSwapchain(&mut self) {
        let (gpu, queues) = self.vk_virtual_GPU.as_ref().unwrap();
        let (window, surface) = self.window_main.as_ref().unwrap();
        ;
        let capabilities = self.vk_virtual_GPU.as_ref().unwrap().0
            .physical_device()
            .surface_capabilities(surface, Default::default())
            .unwrap();

        let swapchain_parameters = SwapchainCreateInfo {
            min_image_count: capabilities.min_image_count + 1,
            image_format: Format::R8G8B8A8_SRGB,
            image_color_space: ColorSpace::SrgbNonLinear,
            image_extent: capabilities.current_extent.unwrap_or([640, 480]),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode : PresentMode::Mailbox,
            ..Default::default()
        };

        let (swapchain, swapchain_images) = Swapchain::new(gpu.clone(), surface.clone(), swapchain_parameters).unwrap();
        self.swapchain = Some((swapchain, swapchain_images));
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_main.is_none() {
            self.CreateWindow(event_loop);

            if self.vk_GPU.is_none() { self.InitGPU(); }
            if self.vk_virtual_GPU.is_none() { self.InitVirtualGPU(); }
            
            self.BuildSwapchain();

            self.memory_allocator = Some(Arc::from(StandardMemoryAllocator::new_default(self.vk_virtual_GPU.as_ref().unwrap().0.clone())));

            self.vert_buffer = Some(DefaultVertexBuffer(self.memory_allocator.as_ref().unwrap().clone()));
        }
        //this is the start of the app
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
        
    }
}




//Sets up connection to GPU
fn InitializeVulkano(event_loop : &EventLoop<()>) -> Arc<Instance> {
    let vk_library = VulkanLibrary::new().expect("failed to make vulkan library"); 
    
    //get gpu driver (instance)
    let vk_instance_parameters: InstanceCreateInfo = InstanceCreateInfo {
        enabled_extensions: Surface::required_extensions(&event_loop), //swapchain extensions
        ..Default::default()
    };
    let vk_driver: Arc<Instance> = Instance::new(
        vk_library.clone(),
        vk_instance_parameters)
        .expect("Failed to create vulkan instance");

    //we can't locate a device until we've made our window, so this is all we can do for now
    return vk_driver;
}

//find an optimal GPU to use and a graphics queue index to use
fn LocateDevice(vk_driver : Arc<Instance>, extensions : &DeviceExtensions, window_surface : Arc<Surface>) 
-> Option<(Arc<PhysicalDevice>, Vec<u32>)> {
    use vulkano::device::physical::PhysicalDeviceType::*;
    
    //get all devices
    //select one to be the logical device
    let mut graphics_processors : Vec<Arc<PhysicalDevice>> = vk_driver.enumerate_physical_devices()
        .unwrap()
        .collect();

    let selected_gpu: Option<(Arc<PhysicalDevice>, Vec<u32>)> = graphics_processors
        .into_iter()
        .filter(|gpu | {
            gpu.supported_extensions().contains(&extensions)
        })
        .map(|gpu| {
            let queue_indices : Vec<u32> = gpu.queue_family_properties().iter().enumerate()
                .filter(|(index, family)| {
                    family.queue_flags.contains(QueueFlags::GRAPHICS) 
                        && gpu.surface_support(*index as u32, &window_surface).unwrap_or(false)
                })
                .map(|(index, family)| {
                index as u32
            }).collect();

            (gpu, queue_indices)
        })
        .filter(|(gpu, queue_indices)| {
            queue_indices.len() > 0
        })
        .min_by_key(|(gpu, queue_indices)| {
            match gpu.properties().device_type{
                DiscreteGpu => 0,
                IntegratedGpu => 1,
                VirtualGpu => 2,
                Cpu => 3,
                _ => 4
            }
        });

    return selected_gpu;
}

fn DefaultVertexBuffer(mem_allocator : Arc<dyn MemoryAllocator>) -> Subbuffer<[Vert]>{
    let vert1 = Vert { position: [-1.0, -1.0] };
    let vert2 = Vert { position: [1.0, -1.0] };
    let vert3 = Vert { position: [-1.0, 1.0] };

    return Buffer::from_iter(
        mem_allocator, 
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        }, 
        AllocationCreateInfo{
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        }, 
        vec![vert1, vert2, vert3])
        .unwrap();
}