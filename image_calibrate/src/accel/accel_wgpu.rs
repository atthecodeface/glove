//a Imports
use std::borrow::Cow;
use std::str::FromStr;

use crate::utils::rtc::run_to_completion as rtc;
use crate::{Accelerate, AccelerateArgs};

//a Support types
//tp Pipeline
#[derive(Debug, Clone, Copy)]
pub struct Pipeline(usize);

//ip Pipeline
impl Pipeline {
    fn as_usize(&self) -> usize {
        self.0
    }
}

//tp From<usize> for Pipeline
impl From<usize> for Pipeline {
    fn from(n: usize) -> Pipeline {
        Pipeline(n)
    }
}

//tp Shader
#[derive(Debug, Clone, Copy)]
pub struct Shader(usize);

//ip Shader
impl Shader {
    fn as_usize(&self) -> usize {
        self.0
    }
}

//ip From<usize> for Shader
impl From<usize> for Shader {
    fn from(n: usize) -> Shader {
        Shader(n)
    }
}

//tp BufferType
pub enum BufferType {
    HostSrc,
    Gpu,
    HostDst,
}

//a AccelWgpu
//tp AccelWgpu
#[derive(Debug)]
pub struct AccelWgpu {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shaders: Vec<wgpu::ShaderModule>,
    pipelines: Vec<(wgpu::ComputePipeline, Vec<wgpu::BindGroupLayout>)>,
    buffers: Vec<wgpu::Buffer>,
}

//ip AccelWgpu
impl AccelWgpu {
    //cp new
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
        let pipelines = vec![];
        let buffers = vec![];
        Self {
            instance,
            adapter,
            device,
            queue,
            shaders,
            pipelines,
            buffers,
        }
    }

    //mp add_shader
    pub fn add_shader(
        &mut self,
        code: &str,
        opt_label: Option<&'static str>,
    ) -> Result<Shader, String> {
        let shader_desc = wgpu::ShaderModuleDescriptor {
            label: opt_label,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
        };

        // Loads the shader from WGSL
        let sm = self.device.create_shader_module(shader_desc);
        let n = self.shaders.len();
        self.shaders.push(sm);
        Ok(n.into())
    }

    //ap device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    //ap queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    //ap shader
    pub fn shader(&self, shader: Shader) -> Option<&wgpu::ShaderModule> {
        self.shaders.get(shader.as_usize())
    }

    //ap shader_err
    pub fn shader_err(&self, shader: Shader) -> Result<&wgpu::ShaderModule, String> {
        self.shader(shader).ok_or("Bad shader index".into())
    }

    //ap pipeline
    pub fn pipeline(&self, pipeline: Pipeline) -> Option<&wgpu::ComputePipeline> {
        self.pipelines.get(pipeline.as_usize()).map(|(p, _bg)| p)
    }

    //ap pipeline_err
    pub fn pipeline_err(&self, pipeline: Pipeline) -> Result<&wgpu::ComputePipeline, String> {
        self.pipeline(pipeline).ok_or("Bad pipeline index".into())
    }

    //mp create_pipeline
    pub fn create_pipeline(
        &mut self,
        shader: Shader,
        entry_point: &str,
        num_bind_groups: usize,
    ) -> Result<Pipeline, String> {
        let pipeline_desc = wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: self.shader_err(shader)?,
            entry_point,
        };
        let pipeline = self.device().create_compute_pipeline(&pipeline_desc);
        let n = self.pipelines.len();
        let mut bgl = vec![];
        for i in 0..num_bind_groups {
            bgl.push(pipeline.get_bind_group_layout(i as u32));
        }
        self.pipelines.push((pipeline, bgl));
        Ok(n.into())
    }

    //ap bind_group_layout
    pub fn bind_group_layout(
        &self,
        pipeline: Pipeline,
        index: usize,
    ) -> Option<&wgpu::BindGroupLayout> {
        self.pipelines
            .get(pipeline.as_usize())
            .map(|(_, bgl)| bgl.get(index))
            .flatten()
    }

    //mp create_buffer
    pub fn create_buffer(
        &self,
        buffer_type: BufferType,
        buffer_size: usize,
        opt_label: Option<&'static str>,
    ) -> wgpu::Buffer {
        let usage = match buffer_type {
            BufferType::HostDst => wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            BufferType::HostSrc => wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            BufferType::Gpu => {
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
            }
        };
        let bd = wgpu::BufferDescriptor {
            label: opt_label,
            size: buffer_size as u64,
            usage,
            mapped_at_creation: false,
        };
        self.device.create_buffer(&bd)
    }

    //mp block
    pub fn block(&self) {
        self.device()
            .poll(wgpu::Maintain::wait())
            .panic_on_timeout();
    }

    //zz All done
}

//a ImageAccelerator
//tp ImageAccelerator
#[derive(Debug)]
pub struct ImageAccelerator {
    accelerator: AccelWgpu,
    buffer_size: usize,
    compute_pipeline: Pipeline,
    input_buffer: wgpu::Buffer,
    storage_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

//ip ImageAccelerator
impl ImageAccelerator {
    fn new(mut accelerator: AccelWgpu, buffer_size: usize) -> Result<Self, String> {
        let cs_module = accelerator.add_shader(include_str!("shader.wgsl"), None)?;
        let compute_pipeline = accelerator.create_pipeline(cs_module, "main", 1)?;
        let (input_buffer, storage_buffer, output_buffer) =
            Self::create_buffers(&mut accelerator, buffer_size);
        let bind_group = accelerator
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: accelerator.bind_group_layout(compute_pipeline, 0).unwrap(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: storage_buffer.as_entire_binding(),
                }],
            });

        Ok(Self {
            accelerator,
            buffer_size,
            compute_pipeline,
            input_buffer,
            storage_buffer,
            output_buffer,
            bind_group,
        })
    }

    //mp cmd_buffer
    fn cmd_buffer(&self, number_of_ops: usize) -> Result<wgpu::CommandBuffer, String> {
        let mut encoder = self
            .accelerator
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(
            &self.input_buffer,
            0,
            &self.storage_buffer,
            0,
            self.buffer_size as u64,
        );
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(self.accelerator.pipeline_err(self.compute_pipeline)?);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.insert_debug_marker("image accelerator slice");
            // Number of cells to run, the (x,y,z) size of item being processed
            cpass.dispatch_workgroups(number_of_ops as u32, 1, 1);
        }
        encoder.copy_buffer_to_buffer(
            &self.storage_buffer,
            0,
            &self.output_buffer,
            0,
            self.buffer_size as u64,
        );

        Ok(encoder.finish())
    }

    //fi create_buffers
    fn create_buffers(
        accelerator: &mut AccelWgpu,
        buffer_size: usize,
    ) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
        (
            accelerator.create_buffer(BufferType::HostSrc, buffer_size, Some("Input")),
            accelerator.create_buffer(BufferType::Gpu, buffer_size, Some("Storage")),
            accelerator.create_buffer(BufferType::HostDst, buffer_size, Some("Output")),
        )
    }

    //mp copy_to_input
    fn copy_to_input(&self, src_data: &[u8]) -> Result<(), String> {
        let byte_size = src_data.len() as u64;
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.input_buffer.slice(0..byte_size);
        buffer_slice.map_async(wgpu::MapMode::Write, move |v| sender.send(v).unwrap());
        self.accelerator.block();
        let Ok(Ok(())) = receiver.recv() else {
            return Err("Failed to run compute on gpu!".into());
        };
        buffer_slice
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(src_data));
        self.input_buffer.unmap();
        Ok(())
    }

    //mp copy_output
    fn copy_output(&self, byte_size: usize) -> Result<wgpu::BufferView, String> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.output_buffer.slice(0..byte_size as u64);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        self.accelerator.block();
        let Ok(Ok(())) = receiver.recv() else {
            return Err("Failed to run compute on gpu!".into());
        };
        Ok(buffer_slice.get_mapped_range())
    }
    //mp run
    fn run<F: FnOnce(&[u8]) -> Result<(), String>>(
        &self,
        src_data: &[u8],
        cmd_buffer: wgpu::CommandBuffer,
        callback: F,
    ) -> Result<(), String> {
        self.copy_to_input(src_data)?;
        self.accelerator.queue().submit(Some(cmd_buffer));
        let data = self.copy_output(src_data.len())?;
        callback(&data)?;
        // Must drop data so that self.output_buffer has no BufferView's
        drop(data);
        // Must unmap self.output_buffer so it can be used in the future
        self.output_buffer.unmap();
        Ok(())
    }

    //zz All done
}

//ip Accelerate for ImageAccelerator
impl Accelerate for ImageAccelerator {
    //mp run_shader
    fn run_shader<F: FnOnce(&[u8]) -> Result<(), String>>(
        &self,
        shader: &str,
        src_data: &[u8],
        args: &AccelerateArgs,
        callback: F,
    ) -> Result<bool, String> {
        if shader == "collatz" {
            let cmd_buffer = self.cmd_buffer(args.width)?;
            self.run(src_data, cmd_buffer, callback)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

//a Test
//fi execute_gpu
fn execute_gpu(img_acc: &ImageAccelerator, numbers: &[u32]) -> Result<Vec<u32>, String> {
    let args = AccelerateArgs {
        width: numbers.len(),
        ..std::default::Default::default()
    };
    let mut result: Vec<u32> = vec![];
    img_acc.run_shader("collatz", bytemuck::cast_slice(numbers), &args, |data| {
        result = bytemuck::cast_slice(data).to_vec();
        Ok(())
    })?;
    Ok(result)
}

//fp run
const OVERFLOW: u32 = 0xffffffff;
pub fn run() -> Result<(), String> {
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
    let accelerator = AccelWgpu::new();
    let img_acc = ImageAccelerator::new(accelerator, 1024)?;
    for _ in 0..3 {
        let steps = execute_gpu(&img_acc, &numbers)?;
        let disp_steps: Vec<String> = steps
            .iter()
            .map(|&n| match n {
                OVERFLOW => "OVERFLOW".to_string(),
                _ => n.to_string(),
            })
            .collect();
        println!("Steps: [{}]", disp_steps.join(", "));
    }

    Ok(())
}
