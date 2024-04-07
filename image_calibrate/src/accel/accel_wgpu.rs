use wgpu::util::DeviceExt;

use std::borrow::Cow;
use std::str::FromStr;

use crate::utils::rtc::run_to_completion as rtc;
pub struct AccelWgpu {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shaders: Vec<wgpu::ShaderModule>,
}

impl AccelWgpu {
    pub fn new() -> Self {
        // Instantiates instance of WebGPU
        let instance = wgpu::Instance::default();

        // `request_adapter` instantiates the general connection to the GPU
        let adapter =
            rtc(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();

        // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
        //  `features` being the available features.
        let (device, queue) = rtc(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        ))
        .unwrap();
        let shaders = vec![];
        Self {
            instance,
            adapter,
            device,
            queue,
            shaders,
        }
    }
    pub fn add_shader(&mut self, code: &str, opt_label: Option<&'static str>) -> usize {
        let shader_desc = wgpu::ShaderModuleDescriptor {
            label: opt_label,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
        };

        // Loads the shader from WGSL
        let sm = self.device.create_shader_module(shader_desc);
        let n = self.shaders.len();
        self.shaders.push(sm);
        n
    }
    pub fn shader(&self, n: usize) -> Option<&wgpu::ShaderModule> {
        self.shaders.get(n)
    }
    /*
        pub fn create_buffer() -> {
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        }
    */
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

fn execute_gpu(numbers: &[u32]) -> Option<Vec<u32>> {
    let mut accelerator = AccelWgpu::new();

    // Gets the size in bytes of the buffer.
    let size = std::mem::size_of_val(numbers) as wgpu::BufferAddress;

    let cs_module = accelerator.add_shader(include_str!("shader.wgsl"), None);

    // Instantiates buffer without data.
    // `usage` of buffer specifies how it can be used:
    //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
    //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
    let staging_buffer = accelerator.device().create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Instantiates buffer with data (`numbers`).
    // Usage allowing the buffer to be:
    //   A storage buffer (can be bound within a bind group and thus available to a shader).
    //   The destination of a copy.
    //   The source of a copy.
    let storage_buffer =
        accelerator
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Storage Buffer"),
                contents: bytemuck::cast_slice(numbers),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            });

    // A bind group defines how buffers are accessed by shaders.
    // It is to WebGPU what a descriptor set is to Vulkan.
    // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).

    // A pipeline specifies the operation of a shader

    // Instantiates the pipeline.
    let pipeline_desc = wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: accelerator.shader(cs_module).unwrap(),
        entry_point: "main",
    };
    let compute_pipeline = accelerator.device().create_compute_pipeline(&pipeline_desc);

    // Instantiates the bind group, once again specifying the binding of buffers.
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = accelerator
        .device()
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
        });

    // A command encoder executes one or many pipelines.
    // It is to WebGPU what a command buffer is to Vulkan.
    let mut encoder = accelerator
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("compute collatz iterations");
        cpass.dispatch_workgroups(numbers.len() as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
    }
    // Sets adds copy operation to command encoder.
    // Will copy data from storage buffer on GPU to staging buffer on CPU.
    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);

    // Submits command encoder for processing
    accelerator.queue().submit(Some(encoder.finish()));

    // Note that we're not calling `.await` here.
    let buffer_slice = staging_buffer.slice(..);
    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    accelerator
        .device()
        .poll(wgpu::Maintain::wait())
        .panic_on_timeout();

    // Awaits until `buffer_future` can be read from
    if let Ok(Ok(())) = receiver.recv() {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        dbg!(&data);
        // Since contents are got in bytes, this converts these bytes back to u32
        let result = bytemuck::cast_slice(&data).to_vec();
        dbg!(&result);

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap(); // Unmaps buffer from memory
                                // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                //   delete myPointer;
                                //   myPointer = NULL;
                                // It effectively frees the memory

        // Returns data from buffer
        Some(result)
    } else {
        panic!("failed to run compute on gpu!")
    }
}

const OVERFLOW: u32 = 0xffffffff;
pub fn run() {
    let numbers = if std::env::args().len() <= 2 {
        let default = vec![1, 2, 3, 4];
        println!("No numbers were provided, defaulting to {default:?}");
        default
    } else {
        std::env::args()
            .skip(2)
            .map(|s| u32::from_str(&s).expect("You must pass a list of positive integers!"))
            .collect()
    };
    let steps = execute_gpu(&numbers).unwrap();
    let disp_steps: Vec<String> = steps
        .iter()
        .map(|&n| match n {
            OVERFLOW => "OVERFLOW".to_string(),
            _ => n.to_string(),
        })
        .collect();

    println!("Steps: [{}]", disp_steps.join(", "));
}
